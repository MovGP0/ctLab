# c't-Lab Firmware Rust Port

This directory contains a best-effort Rust port of the original Pascal firmware sources from `../Firmware/`.

Rules for this port:

- Preserve each Pascal source as a Rust module root so provenance remains traceable.
- Put every production struct, enum, and trait in its own snake_case `.rs` file. Module roots retain constants, free functions, tests, wiring, and selective re-exports.
- Keep positional protocol and calibration tables one entry per line so humans can compare them directly with the Pascal arrays.
- Keep at most one empty line between Rust items. `build.rs` enforces this during every Cargo build and reports the offending file and line.
- Prefer explicit Rust types, enums, structs, and constants over macro-heavy transliteration.
- Preserve hardware-specific assumptions in comments when no direct Rust equivalent is implemented.
- Use `todo!()` sparingly; prefer placeholder functions and data structures that keep the code readable.
- Do not modify the original Pascal sources in `../Firmware/`.

Project status:

- `Cargo.toml` now defines a real Rust project rooted in this directory.
- `src/lib.rs` exposes every translated module, the low-level hardware modules, and the shared `avrd` support layer so host checks and tests cannot silently omit a port.
- `src/avrd_support.rs` provides thin register/MMIO helpers on top of the `avrd` crate for `ATmega32` and `ATmega644`.
- The `*-HW.rs` files now include concrete `avrd` backends alongside the structural source ports.
- The release profile uses `opt-level = "z"`, LTO, one codegen unit, aborting panics, and stripped symbols to minimize deployable artifacts.

## AVR flash budgets

The AVR build uses Rust's built-in Tier 3 `avr-none` target. It requires the pinned nightly toolchain with `rust-src` because Cargo must build `core`, plus `avr-gcc`, `avr-objcopy`, and `avr-size`. On Windows, install the tested AVR GCC package with:

```powershell
winget install --exact --id ZakKemble.avr-gcc
```

`rust-toolchain.toml` installs the required nightly and `rust-src` automatically. Stable Rust alone cannot use Cargo's `build-std` feature required by this target.

The independent `avr-smoke` package proves that Windows can compile `core`, link a genuine `no_std` AVR executable through `avr-gcc`, convert it to Intel HEX, and enforce flash limits without compiling the host-only behavioral library. Build a device-specific smoke artifact from this directory with:

```text
rust-script ./tools/release-avr.rs \
  --mcu atmega32 \
  --manifest-path ./avr-smoke/Cargo.toml \
  --elf ./avr-smoke/target/atmega32/avr-none/release/ctlab-avr-smoke.elf \
  --hex ./avr-smoke/target/atmega32/avr-none/release/ctlab-avr-smoke.hex \
  --baseline 114 \
  -- --target-dir target/atmega32
```

Replace both `atmega32` occurrences and the baseline with `atmega168`/`134` or `atmega644`/`142` for the other supported devices. These smoke baselines were produced by `nightly-2026-07-11` and AVR GCC 14.1.0. Program flash is `.text + .data`; the wrapper rejects physical-limit and baseline regressions and only writes HEX after a successful build and size check.

The target CPU and matching AVR GCC `-mmcu` linker argument are selected from `--mcu`. Do not replace the built-in `avr-none` target with an ad-hoc target JSON.

Current limitation:

- The higher-level firmware program ports are host-side behavioral translations and still use heap-backed standard-library types such as `String` and `Vec`. They compile and test as a library, but are not yet deployable `no_std` AVR binaries.
- The smoke binary uses AVR GCC's device CRT and a minimal Rust `main`/panic loop, but the translated firmware still lacks per-device entry points and interrupt-vector integration. Consequently, the smoke sizes prove the toolchain only; they are not firmware flash measurements.
