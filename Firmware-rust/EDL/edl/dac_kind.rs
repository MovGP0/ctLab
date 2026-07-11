#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Installed DAC family selected from EEPROM option bits.
pub enum DacKind {
    /// Twelve-bit LTC8043-compatible interface.
    Ltc8043,

    /// Twelve-bit AD5452 interface with command prefix.
    Ad5452,

    /// Sixteen-bit DAC8501/LTC1655 interface.
    Dac8501,

    /// Sixteen-bit DAC8811 interface.
    Dac8811,
}

impl DacKind {
    /// Decodes the two low EEPROM option bits written by the Pascal `tDAC` value.
    pub const fn from_options(options: u8) -> Self {
        match options & 0b0000_0011 {
            1 => Self::Ad5452,
            2 => Self::Dac8501,
            3 => Self::Dac8811,
            _ => Self::Ltc8043,
        }
    }
}
