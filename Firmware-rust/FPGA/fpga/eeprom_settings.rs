//! EEPROM defaults that reconstruct the Pascal controller's cold-start state.

use super::*;

/// EEPROM-backed boot configuration retained by the ATmega644 controller.
///
/// The layout mirrors the Pascal defaults so a newly initialized controller
/// selects the same serial routing, register values, and startup files.
#[derive(Debug, Clone)]
pub struct EepromSettings
{
    /// Magic value distinguishing initialized EEPROM from erased storage.
    pub initialized: u16,

    /// AVR UART divisor restored before the command loop starts.
    pub serial_baud_register: u8,

    /// Whether numeric replies use the Pascal hexadecimal presentation mode.
    pub hex_mode: bool,

    /// Indexed firmware options, including channels and core serial registers.
    pub options: [i16; 20],

    /// Initial values copied into the first four calculator registers at boot.
    pub initial_registers: [i32; 4],

    /// Script loaded during controller initialization when automatic startup is enabled.
    pub init_file_name: String,

    /// Default payload file used by load/save commands without an explicit name.
    pub data_file_name: String,
}

impl Default for EepromSettings
{
    /// Reproduces the values written by the Pascal `InitEEProm` path.
    #[rustfmt::skip]
    fn default() -> Self
    {
        Self
        {
            initialized: EEPROM_INITIALIZED,
            serial_baud_register: 51,
            hex_mode: false,
            options: [
                255,
                255,
                0,
                128,
                0,
                10,
                7,
                255,
                9,
                500,
                64,
                65,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            initial_registers: [0; 4],
            init_file_name: DEFAULT_INIT_FILE.to_string(),
            data_file_name: DEFAULT_DATA_FILE.to_string(),
        }
    }
}
