//! Best-effort Rust port of `ADA-C.pas`.
//!
//! This keeps the original firmware structure readable:
//! - command/error enums
//! - EEPROM defaults and runtime state
//! - DAC, port, and parameter conversion logic
//! - serial response formatting
//! - high-level initialization, parser, and trigger scan flow
//!
//! Hardware-specific units imported by the Pascal source
//! (`SysTick`, `ADCport`, `TWImaster`, `LCDmultiPort`, `SerPort`,
//! `I2CExpand`, `IncrPort4`, `ADA-C-HW.pas`, `ADA-C-Parser.pas`)
//! are represented here behind a trait and local test doubles.

#![allow(dead_code)]

use std::array;
use std::fmt::Write as _;

/// Uses 32-bit floating point to match the precision and storage cost of the Pascal firmware.
pub type Float = f32;

/// Declares the 16 MHz AVR clock used to derive UART, TWI, ADC, and systick timing.
pub const PROC_CLOCK: u32 = 16_000_000;

/// Defines the two-millisecond ADA systick used to convert timer values into polling-loop ticks.
pub const SYS_TICK_MS: u16 = 2;

/// Limits one ADA serial poll to 20 ms so the main loop continues servicing triggers and the panel.
pub const SERIAL_POLL_TIMEOUT_MS: u16 = 20;

/// Keeps the ADA trigger LED asserted for 30 systicks after a trigger.
pub const TRIGGER_LED_TICKS: u16 = 30;

/// Configures `DDR_B_INIT` output bits for the board's strobes, clocks, LEDs, and serial lines.
pub const DDR_B_INIT: u8 = 0b0101_1011;

/// Sets `PORT_B_INIT` pull-ups and idle output levels before peripherals are accessed.
pub const PORT_B_INIT: u8 = 0b1011_1111;

/// Configures `DDR_C_INIT` output bits for the board's strobes, clocks, LEDs, and serial lines.
pub const DDR_C_INIT: u8 = 0b1111_1100;

/// Sets `PORT_C_INIT` pull-ups and idle output levels before peripherals are accessed.
pub const PORT_C_INIT: u8 = 0b0000_0011;

/// Configures `DDR_D_INIT` output bits for the board's strobes, clocks, LEDs, and serial lines.
pub const DDR_D_INIT: u8 = 0b0000_1100;

/// Sets `PORT_D_INIT` pull-ups and idle output levels before peripherals are accessed.
pub const PORT_D_INIT: u8 = 0b1111_1100;

/// Selects Port B bit 0, the shared serial clock for the ADA converters and shift registers.
pub const B_SCLK: u8 = 0;

/// Selects Port B bit 1, the shared serial-data output for DAC and 4094 writes.
pub const B_SDATAOUT: u8 = 1;

/// Selects Port B bit 2, the board trigger/activity LED output.
pub const B_TRIG: u8 = 2;

/// Selects Port B bit 3, the latch/chip-select strobe for the installed DAC.
pub const B_STR_DAC: u8 = 3;

/// Selects Port B bit 4, the active-low conversion/read strobe for the LTC1864 ADC.
pub const B_STR_AD16: u8 = 4;

/// Selects Port B bit 5, the serial-data input sampled from the LTC1864 ADC.
pub const B_SDATAIN1: u8 = 5;

/// Selects Port B bit 6, the latch strobe for the four cascaded 4094 output registers.
pub const B_STR_SR: u8 = 6;

/// Selects Port B bit 7, the board-presence/sense input read during daughterboard detection.
pub const B_SENSE: u8 = 7;

/// Selects Port C bit 5, the latch strobe for the DAC channel multiplexer.
pub const B_STR_DA_MUX: u8 = 5;

/// Provides the full identification string returned by the `IDN` command.
pub const VERS1_STR: &str = "1.742 [ADA by CM/c't 04/2007; ";

/// Provides the compact firmware name shown on the front-panel startup screen.
pub const VERS3_STR: &str = "ADA 1.74";

/// Prefixes the ADA slave address on the startup display.
pub const ADR_STR: &str = "Adr ";

/// Labels the startup page that enumerates detected ADA daughterboards.
pub const CARDS_STR: &str = "IO-Cards";

/// Labels the numeric value field on ADA parameter-edit screens.
pub const VALUE_STR: &str = "Value ";

/// Warns at startup that the EEPROM sentinel was absent and default ADA settings were loaded.
pub const EE_NOT_PROGRAMMED_STR: &str = "EEPROM EMPTY! ";

/// Adds the `DA12` capability label when an LTC1257 daughterboard is detected.
pub const DAC12_STR: &str = "DA12 ";

/// Adds the `DA16` capability label when an LTC1655 or DAC714 daughterboard is detected.
pub const DAC16_STR: &str = "DA16 ";

/// Adds the `AD16` capability label when the external LTC1864 ADC is detected.
pub const ADC16_STR: &str = "AD16 ";

/// Adds the `LCD` capability label when a display daughterboard is detected.
pub const LCD_STR: &str = "LCD ";

/// Adds the `IO32` capability label for the four cascaded eight-bit 4094 output ports.
pub const IO816_STR: &str = "IO32 ";

/// Supplies the original firmware's subchannel-28.5 Easter-egg response text.
pub const EGG_STR: &str = "28.5 [Michaela, ich liebe dich!]";

/// Reserves err sub ch as the wire-level subchannel used by existing ctLab clients.
pub const ERR_SUB_CH: u8 = 255;

#[path = "ada_c/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;
#[path = "ada_c/error_code.rs"]
mod error_code;
pub use error_code::ErrorCode;
#[path = "ada_c/eeprom_data.rs"]
mod eeprom_data;
pub use eeprom_data::EepromData;
#[path = "ada_c/runtime_status.rs"]
mod runtime_status;
pub use runtime_status::RuntimeStatus;
#[path = "ada_c/ada_hardware.rs"]
mod ada_hardware;
pub use ada_hardware::AdaHardware;
#[path = "ada_c/device_state.rs"]
mod device_state;
pub use device_state::DeviceState;

#[cfg(test)]
#[path = "ADA-C_tests.rs"]
mod tests;
