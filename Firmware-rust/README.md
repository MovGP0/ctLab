# c't-Lab Firmware Rust Port

This directory contains a best-effort Rust port of the original Pascal firmware sources from `../Firmware/`.

Rules for this port:

- Preserve the original module and file structure where practical.
- Keep one Rust file per Pascal source file, using the same basename and a `.rs` extension.
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

Program flash is the ELF `.text` plus initialized `.data` copied from flash at startup. Use the Rust release wrapper so a successful AVR release always includes the footprint check:

```text
rust-script ./tools/release-avr.rs \
  --mcu atmega32 \
  --elf ./target/avr-atmega32/release/div.elf \
  -- --target ./targets/avr-atmega32.json --bin div
```

The wrapper first runs `cargo build --release` with the arguments after `--`, then invokes `avr-size`. It rejects a missing ELF, a failed build, an image larger than the selected MCU's physical flash, or an optional tighter budget/baseline regression:

```text
rust-script ./tools/release-avr.rs \
  --mcu atmega644 \
  --elf ./target/avr-atmega644/release/fpga.elf \
  --budget 60000 \
  --baseline 54000 \
  --allowed-regression 256 \
  -- --target ./targets/avr-atmega644.json --bin fpga
```

Supported physical limits are ATmega168 (16 KiB), ATmega32 (32 KiB), and ATmega644 (64 KiB). CI should retain the checker's output with each deployable target and pass explicit baseline values from version-controlled target-specific policy once those budgets are agreed.

Current limitation:

- The higher-level firmware program ports are host-side behavioral translations and still use heap-backed standard-library types such as `String` and `Vec`. They compile and test as a library, but are not yet deployable `no_std` AVR binaries.
- No AVR linker configuration, startup/runtime, interrupt vector table, or per-device binary entry points exist yet. Consequently, a host release artifact is not a meaningful AVR flash measurement; the wrapper requires the linked AVR ELF produced by the target-specific build.
