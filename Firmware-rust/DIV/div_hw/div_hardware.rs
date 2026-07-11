//! Defines DIV the board-I/O contract that separates protocol logic from register access.

#[allow(unused_imports)]
use super::*;

/// Defines the div hardware boundary so translated timing and protocol logic can run against AVR registers or deterministic host doubles.
pub trait DivHardware {
    /// Drives the LTC2400 chip-select/strobe on Port B bit 4.
    fn set_str_ad24(&mut self, high: bool);

    /// Drives the LTC2400 serial clock on Port B bit 7.
    fn set_sclk(&mut self, high: bool);

    /// Samples sdata in1 directly from its mapped input pin during the bit-level peripheral transaction; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_sdata_in1(&self) -> bool;

    /// Writes SPCR to enable the LTC2400 byte transfers or disable SPI after them.
    fn set_spi_control(&mut self, value: u8);

    /// Transfers SPI transfer using the byte order expected by the attached peripheral; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn spi_transfer(&mut self, tx: u8) -> u8;

    /// Burns the requested processor cycles between signal edges where the peripheral data sheet requires setup or hold time; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn spin_delay_cycles(&mut self, cycles: u16);
}
