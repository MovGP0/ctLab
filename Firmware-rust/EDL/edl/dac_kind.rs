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
