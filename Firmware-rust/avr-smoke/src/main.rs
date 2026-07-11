//! Minimal `no_std` AVR executable used to verify the release toolchain.
//!
//! It intentionally performs no board I/O: its purpose is to prove that Rust,
//! `core`, the AVR linker, size checks, and HEX conversion work before a concrete
//! firmware entry point is wired to interrupt vectors and startup hardware.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// Keeps the smoke image alive without assuming any connected peripherals.
///
/// The diverging C entry point satisfies the AVR linker while avoiding accidental
/// register writes on whichever supported MCU is selected for the smoke build.
#[no_mangle]
pub extern "C" fn main() -> !
{
    loop
    {
        core::hint::spin_loop();
    }
}

/// Halts after a panic because a `no_std` microcontroller has no unwinder or OS.
///
/// Spinning preserves the failing state for a debugger and adds no board-specific
/// reset behavior to what is intended to remain a target-neutral smoke image.
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> !
{
    loop
    {
        core::hint::spin_loop();
    }
}
