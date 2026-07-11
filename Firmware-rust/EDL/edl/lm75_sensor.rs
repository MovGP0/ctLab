//! Identifies the two LM75 locations supported by the EDL power stages.

use super::HardwareOption;

/// Selects an LM75 by its board role while retaining its I2C address and enable flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lm75Sensor {
    /// Sensor on the internal power stage at I2C address `0x49`.
    Internal,

    /// Sensor on the external power stage at I2C address `0x48`.
    External,
}

impl Lm75Sensor {
    /// Returns the seven-bit I2C address wired for this sensor location.
    pub const fn address(self) -> u8 {
        match self {
            Self::Internal => 0x49,
            Self::External => 0x48,
        }
    }

    /// Returns the EEPROM option flag that declares this sensor installed.
    pub const fn hardware_option(self) -> HardwareOption {
        match self {
            Self::Internal => HardwareOption::InternalLm75,
            Self::External => HardwareOption::ExternalLm75,
        }
    }
}
