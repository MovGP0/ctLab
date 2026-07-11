//! Shared AVR register access used by the firmware-specific hardware adapters.
//!
//! The generic [`crate::avrd_support::AvrdPortIo`] keeps register selection at
//! compile time through [`crate::avrd_support::Mcu`]. Firmware families can
//! therefore share pin, SPI, and ADC operations
//! without paying for a run-time MCU abstraction on space-constrained targets.

mod register_port;
pub use register_port::RegisterPort;
mod mcu;
pub use mcu::Mcu;
mod atmega32;
pub use atmega32::Atmega32;
mod atmega644;
pub use atmega644::Atmega644;
mod avrd_port_io;
pub use avrd_port_io::AvrdPortIo;

use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

/// Reads an eight-bit memory-mapped register exactly once.
///
/// Volatile access prevents the optimizer from caching or removing peripheral
/// reads whose values can change independently of normal program memory.
///
/// # Safety
///
/// `register` must be non-null, properly aligned, and valid for a volatile
/// eight-bit read from the selected MCU's memory-mapped I/O space.
pub unsafe fn read_u8(register: *mut u8) -> u8 {
    // SAFETY: The caller supplies the validity and alignment guarantees.
    ptr::read_volatile(register)
}

/// Writes an eight-bit memory-mapped register exactly once.
///
/// Volatile access preserves writes that control peripherals even when Rust
/// cannot observe a corresponding read through ordinary memory.
///
/// # Safety
///
/// `register` must be non-null, properly aligned, and valid for a volatile
/// eight-bit write to the selected MCU's memory-mapped I/O space.
pub unsafe fn write_u8(register: *mut u8, value: u8) {
    // SAFETY: The caller supplies the validity and alignment guarantees.
    ptr::write_volatile(register, value);
}

/// Applies `f` to the current value of an eight-bit peripheral register.
///
/// The explicit volatile read-modify-write is used for individual control bits
/// so unrelated pins in the same AVR port retain their previous latch values.
///
/// # Safety
///
/// `register` must satisfy the requirements of both [`read_u8`] and
/// [`write_u8`]. The caller must also prevent interrupt or concurrent writers
/// from racing this non-atomic read-modify-write sequence.
pub unsafe fn update_u8(register: *mut u8, f: impl FnOnce(u8) -> u8) {
    // SAFETY: Forwarded from this function's caller contract.
    let value = read_u8(register);
    // SAFETY: Forwarded from this function's caller contract.
    write_u8(register, f(value));
}

/// Returns `value` with one bit set or cleared according to `high`.
///
/// Hardware adapters use this helper to express active-high and active-low pin
/// assignments without duplicating masks or disturbing adjacent port bits.
///
/// # Panics
///
/// Panics when overflow checks are enabled and `bit` is greater than seven;
/// callers must pass a bit number belonging to an eight-bit AVR register.
pub const fn set_or_clear_bit(value: u8, bit: u8, high: bool) -> u8 {
    if high {
        value | (1 << bit)
    } else {
        value & !(1 << bit)
    }
}
