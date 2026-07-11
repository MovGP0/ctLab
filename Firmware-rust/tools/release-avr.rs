//! Builds one AVR firmware release and enforces its flash-size limits.
//!
//! The wrapper keeps compilation, size accounting, regression checks, and HEX
//! generation in one Rust-based release path. This matters because a successful
//! link alone does not guarantee that the image fits the selected ATmega.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[path = "release_avr/options.rs"]
mod options;
use options::Options;

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Runs the release workflow and maps its diagnostic result to a process status.
///
/// A conventional non-zero exit code lets local scripts and CI treat either a
/// compiler failure or an exceeded flash budget as a failed release.
fn main() -> ExitCode
{
    match run()
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) =>
        {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Executes argument parsing, compilation, size validation, and optional HEX output.
///
/// The order is intentional: no image is emitted until the linked ELF has passed
/// both the physical flash limit and any project-specific regression threshold.
///
/// # Errors
///
/// Returns a diagnostic if arguments are invalid, a tool cannot run, the build
/// fails, the ELF exceeds its limits, or HEX generation fails.
fn run() -> Result<(), String>
{
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments == ["--self-test"]
    {
        return self_test();
    }
    let options = parse_arguments(&arguments)?;
    build_release(&options)?;
    check_size(&options)?;
    write_hex(&options)
}

/// Converts command-line tokens into validated release options.
///
/// Arguments after `--` are forwarded untouched to Cargo so the wrapper can
/// select bins or other build options without duplicating Cargo's interface.
///
/// # Errors
///
/// Returns a diagnostic for missing values, unknown options, malformed numbers,
/// omitted required options, or an unsupported MCU.
fn parse_arguments(arguments: &[String]) -> Result<Options, String>
{
    let mut mcu = None;
    let mut elf = None;
    let mut hex = None;
    let mut manifest = PathBuf::from("Cargo.toml");
    let mut budget = None;
    let mut baseline = None;
    let mut allowed_regression = 0;
    let mut cargo_args = Vec::new();
    let mut index = 0;
    while index < arguments.len()
    {
        if arguments[index] == "--"
        {
            cargo_args.extend_from_slice(&arguments[index + 1..]);
            break;
        }
        let value = arguments.get(index + 1).ok_or_else(|| format!("{} requires a value", arguments[index]))?;
        match arguments[index].as_str()
        {
            "--mcu" => mcu = Some(value.clone()),
            "--elf" => elf = Some(PathBuf::from(value)),
            "--hex" => hex = Some(PathBuf::from(value)),
            "--manifest-path" => manifest = PathBuf::from(value),
            "--budget" => budget = Some(parse_number("budget", value)?),
            "--baseline" => baseline = Some(parse_number("baseline", value)?),
            "--allowed-regression" => allowed_regression = parse_number("allowed regression", value)?,
            unknown => return Err(format!("unknown argument {unknown}")),
        }
        index += 2;
    }
    let mcu = mcu.ok_or("--mcu is required")?;
    flash_limit(&mcu)?;
    Ok(Options
    {
        mcu,
        elf: elf.ok_or("--elf is required")?,
        hex,
        manifest,
        budget,
        baseline,
        allowed_regression,
        cargo_args,
    })
}

/// Parses a decimal byte count and names the failing option in diagnostics.
///
/// # Errors
///
/// Returns an error when `value` is not a valid unsigned decimal integer.
fn parse_number(name: &str, value: &str) -> Result<u64, String>
{
    value.parse().map_err(|_| format!("invalid {name}: {value}"))
}

/// Returns the physical flash capacity of a supported firmware MCU.
///
/// Centralizing the capacities makes the same device constraint govern argument
/// validation and the final size check instead of relying on caller-supplied data.
///
/// # Errors
///
/// Returns an error when `mcu` is not one of the ATmega targets supported by the
/// translated firmware.
fn flash_limit(mcu: &str) -> Result<u64, String>
{
    match mcu.to_ascii_lowercase().as_str()
    {
        "atmega168" => Ok(16 * 1024),
        "atmega32" => Ok(32 * 1024),
        "atmega644" => Ok(64 * 1024),
        _ => Err(format!("unsupported MCU {mcu}; expected atmega168, atmega32, or atmega644")),
    }
}

/// Invokes Cargo's optimized AVR build for the selected MCU.
///
/// MCU-specific compiler and linker flags are appended to existing target flags
/// so specialization is preserved without discarding the caller's environment.
/// Building `core` is required because the `avr-none` target has no prebuilt
/// standard library.
///
/// # Errors
///
/// Returns a diagnostic when the manifest cannot be resolved, has no parent,
/// Cargo cannot start, or Cargo reports an unsuccessful build.
fn build_release(options: &Options) -> Result<(), String>
{
    let manifest = options.manifest.canonicalize().map_err(|error|
    {
        format!("failed to resolve manifest {}: {error}", options.manifest.display())
    })?;
    let working_directory = manifest.parent().ok_or_else(||
    {
        format!("manifest {} has no parent directory", manifest.display())
    })?;
    let required_rustflags = format!(
        "-Ctarget-cpu={} -Clink-arg=-mmcu={}",
        options.mcu.to_ascii_lowercase(),
        options.mcu.to_ascii_lowercase(),
    );
    let rustflags = match env::var("CARGO_TARGET_AVR_NONE_RUSTFLAGS")
    {
        Ok(existing) if !existing.trim().is_empty() => format!("{existing} {required_rustflags}"),
        _ => required_rustflags,
    };
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("avr-none")
        .arg("-Zbuild-std=core")
        .args(&options.cargo_args)
        .env("CARGO_TARGET_AVR_NONE_RUSTFLAGS", rustflags)
        .current_dir(working_directory)
        .status()
        .map_err(|error| format!("failed to start cargo: {error}"))?;
    if !status.success()
    {
        return Err(format!("release build failed with {status}"));
    }
    Ok(())
}

/// Converts the validated ELF into Intel HEX when an output path was requested.
///
/// EEPROM is deliberately excluded because flashing program memory must not
/// overwrite calibration or configuration retained in the controller EEPROM.
///
/// # Errors
///
/// Returns a diagnostic if the output directory cannot be created, `avr-objcopy`
/// cannot start, or conversion fails.
fn write_hex(options: &Options) -> Result<(), String>
{
    let Some(hex) = &options.hex else
    {
        return Ok(());
    };
    ensure_parent_directory(hex)?;
    let output = Command::new("avr-objcopy")
        .args(["-O", "ihex", "-R", ".eeprom"])
        .arg(&options.elf)
        .arg(hex)
        .output()
        .map_err(|error| format!("failed to start avr-objcopy: {error}"))?;
    if !output.status.success()
    {
        return Err(format!(
            "avr-objcopy failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    println!("wrote {}", hex.display());
    Ok(())
}

/// Creates an output file's parent directory when the path contains one.
///
/// Bare filenames need no directory operation; accepting both forms keeps the
/// release command convenient locally and predictable in CI artifact folders.
///
/// # Errors
///
/// Returns a diagnostic when the parent directory cannot be created.
fn ensure_parent_directory(path: &Path) -> Result<(), String>
{
    let Some(parent) = path.parent() else
    {
        return Ok(());
    };
    if parent.as_os_str().is_empty()
    {
        return Ok(());
    }
    std::fs::create_dir_all(parent).map_err(|error|
    {
        format!("failed to create output directory {}: {error}", parent.display())
    })
}

/// Measures linked flash consumption and enforces physical and regression limits.
///
/// AVR initializes `.data` from a copy stored in flash, so both `.text` and
/// `.data` count toward the image even though `.data` later resides in SRAM.
/// `.bss` is excluded because zero-initialized storage consumes no flash payload.
///
/// # Errors
///
/// Returns a diagnostic if the ELF is missing, `avr-size` fails or produces no
/// `.text`, arithmetic overflows, a requested budget exceeds physical flash, or
/// the measured image exceeds its budget or allowed baseline regression.
fn check_size(options: &Options) -> Result<(), String>
{
    if !options.elf.is_file()
    {
        return Err(format!("release build did not produce {}", options.elf.display()));
    }
    let output = Command::new("avr-size")
        .args(["--format=sysv", "--radix=10"])
        .arg(&options.elf)
        .output()
        .map_err(|error| format!("failed to start avr-size: {error}"))?;
    if !output.status.success()
    {
        return Err(format!("avr-size failed with {}: {}", output.status, String::from_utf8_lossy(&output.stderr)));
    }
    let sections = parse_sections(&String::from_utf8_lossy(&output.stdout));
    let text = *sections.get(".text").ok_or("avr-size output has no .text section")?;
    let data = sections.get(".data").copied().unwrap_or(0);
    let used = text.checked_add(data).ok_or("flash size overflow")?;
    let physical_limit = flash_limit(&options.mcu)?;
    let budget = options.budget.unwrap_or(physical_limit);
    if budget > physical_limit
    {
        return Err(format!("budget {budget} exceeds {} physical limit {physical_limit}", options.mcu));
    }
    println!("{}: .text {text} + .data {data} = {used} bytes; budget {budget}; remaining {}", options.mcu, budget.saturating_sub(used));
    if used > budget
    {
        return Err(format!("flash budget exceeded: {used} used, {budget} allowed"));
    }
    if let Some(baseline) = options.baseline
    {
        let regression_limit = baseline.checked_add(options.allowed_regression).ok_or("regression limit overflow")?;
        if used > regression_limit
        {
            return Err(format!("flash regression exceeded: {used} used, baseline {baseline} plus {} allowed", options.allowed_regression));
        }
    }
    Ok(())
}

/// Extracts section sizes from the System V output emitted by `avr-size`.
///
/// Non-section header and summary lines are ignored so callers can address the
/// relevant sections by name without depending on their printed order.
fn parse_sections(output: &str) -> BTreeMap<String, u64>
{
    output
        .lines()
        .filter_map(|line|
        {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            let size = fields.next()?.parse().ok()?;
            name.starts_with('.').then(|| (name.to_string(), size))
        })
        .collect()
}

/// Exercises size parsing, argument forwarding, and MCU capacities without tools.
///
/// This lightweight mode catches wrapper regressions on hosts that do not have
/// the AVR compiler installed, while real release builds remain the end-to-end
/// validation of external tool behavior.
///
/// # Errors
///
/// Returns a diagnostic when any built-in expectation fails.
fn self_test() -> Result<(), String>
{
    let sections = parse_sections(".text 1000 0\n.data 64 1000\n.bss 10 1064\n");
    if sections.get(".text") != Some(&1000) || sections.get(".data") != Some(&64)
    {
        return Err("section parser self-test failed".to_string());
    }
    let arguments = [
        "--mcu", "atmega32", "--elf", "firmware.elf", "--hex", "firmware.hex",
        "--budget", "30000", "--baseline", "29000",
        "--allowed-regression", "256", "--", "--target", "avr-atmega32.json", "--bin", "div",
    ].map(String::from);
    let options = parse_arguments(&arguments)?;
    if options.mcu != "atmega32"
        || options.hex != Some(PathBuf::from("firmware.hex"))
        || options.budget != Some(30000)
        || options.cargo_args.len() != 4
    {
        return Err("argument parser self-test failed".to_string());
    }
    if flash_limit("atmega168")? != 16_384 || flash_limit("atmega644")? != 65_536
    {
        return Err("MCU limit self-test failed".to_string());
    }
    println!("release-avr self-test passed");
    Ok(())
}
