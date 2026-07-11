//! ATmega644 controller for the c't-Lab FPGA module.
//!
//! The original `FPGA.pas` is controller firmware, not FPGA gateware. This
//! port covers its stateful register bridge, serial parser dispatch, FAT-style
//! file operations, FPGA configuration stream, and auto-increment data paths.

/// Filesystem boundary for SD-card configuration and data commands.
#[path = "fpga/file_system.rs"]
mod file_system;
pub use file_system::FileSystem;

/// Controller failures separated from parser and storage implementation details.
#[path = "fpga/controller_error.rs"]
mod controller_error;
pub use controller_error::ControllerError;

/// Transport-neutral command result values.
#[path = "fpga/response.rs"]
mod response;
pub use response::Response;

/// EEPROM state restoring Pascal-compatible boot options.
#[path = "fpga/eeprom_settings.rs"]
mod eeprom_settings;
pub use eeprom_settings::EepromSettings;

/// Main controller state machine joining parser, SPI bridge, and storage.
#[path = "fpga/fpga_controller.rs"]
mod fpga_controller;
pub use fpga_controller::FpgaController;

use super::fpga_hw::{FpgaBus, FpgaHardware};
use super::fpga_parser::{parse_frame, Parameter, ParseError, ParsedFrame};

/// Identification string returned by the legacy `IDN` subchannel.
pub const VERSION: &str = "2.61 [FPGA by CM/c't 06/2008]";

/// EEPROM magic written after defaults are initialized successfully.
pub const EEPROM_INITIALIZED: u16 = 0xAA55;

/// Startup script name used by an erased or newly initialized EEPROM image.
pub const DEFAULT_INIT_FILE: &str = "BASIC.INI";

/// Default FPGA memory-transfer file retained for protocol compatibility.
pub const DEFAULT_DATA_FILE: &str = "DATAFILE.XLS";

/// Number of floating-point calculator registers exposed by command families 300-669.
pub const REGISTER_COUNT: usize = 10;

/// Size of the FPGA's directly addressable 32-bit register mirror.
pub const FPGA_REGISTER_COUNT: usize = 64;

/// Requires a numeric parameter without coupling command dispatch to parser internals.
fn parameter_number<E>(parameter: &Parameter) -> Result<f64, ControllerError<E>>
{
    match parameter
    {
        Parameter::Number(value) => Ok(*value),
        _ => Err(ControllerError::InvalidParameter),
    }
}

/// Requires an owned text parameter for filename setters and file operations.
fn parameter_text<E>(parameter: Parameter) -> Result<String, ControllerError<E>>
{
    match parameter
    {
        Parameter::Text(value) => Ok(value),
        _ => Err(ControllerError::InvalidParameter),
    }
}

#[cfg(test)]
#[path = "FPGA_tests.rs"]
mod tests;
