//! Hardware-facing portion of the FPGA module controller firmware.
//!
//! This is a behavioral port of `FPGA-HW.pas`. The original ATmega644 uses
//! SPI for a register-select transaction followed by an 8/16/32-bit data
//! transaction. Multi-byte values are transferred most-significant byte first.

/// Electrical operations supplied by the ATmega644 backend.
#[path = "fpga_hw/fpga_hardware.rs"]
mod fpga_hardware;
pub use fpga_hardware::FpgaHardware;

/// Width-aware register bridge and interrupt receive buffer.
#[path = "fpga_hw/fpga_bus.rs"]
mod fpga_bus;
pub use fpga_bus::FpgaBus;

/// ASCII ACK sent to the FPGA core after internally routed serial handling.
pub const CORE_ACK: u8 = 0x06;

/// One-byte-index ring capacity matching the Pascal interrupt buffer design.
pub const CORE_BUFFER_CAPACITY: usize = 256;

#[cfg(test)]
#[path = "FPGA-HW_tests.rs"]
mod tests;
