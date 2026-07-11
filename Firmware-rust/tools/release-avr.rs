use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

#[derive(Debug, PartialEq, Eq)]
struct Options
{
    mcu: String,
    elf: PathBuf,
    manifest: PathBuf,
    budget: Option<u64>,
    baseline: Option<u64>,
    allowed_regression: u64,
    cargo_args: Vec<String>,
}

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

fn run() -> Result<(), String>
{
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments == ["--self-test"]
    {
        return self_test();
    }
    let options = parse_arguments(&arguments)?;
    build_release(&options)?;
    check_size(&options)
}

fn parse_arguments(arguments: &[String]) -> Result<Options, String>
{
    let mut mcu = None;
    let mut elf = None;
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
        manifest,
        budget,
        baseline,
        allowed_regression,
        cargo_args,
    })
}

fn parse_number(name: &str, value: &str) -> Result<u64, String>
{
    value.parse().map_err(|_| format!("invalid {name}: {value}"))
}

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

fn build_release(options: &Options) -> Result<(), String>
{
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(&options.manifest)
        .args(&options.cargo_args)
        .status()
        .map_err(|error| format!("failed to start cargo: {error}"))?;
    if !status.success()
    {
        return Err(format!("release build failed with {status}"));
    }
    Ok(())
}

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

fn self_test() -> Result<(), String>
{
    let sections = parse_sections(".text 1000 0\n.data 64 1000\n.bss 10 1064\n");
    if sections.get(".text") != Some(&1000) || sections.get(".data") != Some(&64)
    {
        return Err("section parser self-test failed".to_string());
    }
    let arguments = [
        "--mcu", "atmega32", "--elf", "firmware.elf", "--budget", "30000", "--baseline", "29000",
        "--allowed-regression", "256", "--", "--target", "avr-atmega32.json", "--bin", "div",
    ].map(String::from);
    let options = parse_arguments(&arguments)?;
    if options.mcu != "atmega32" || options.budget != Some(30000) || options.cargo_args.len() != 4
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
