#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Installed DAC protocol selected by the low EEPROM option bits.
pub enum DacType {
    /// Twelve-bit LTC8043-compatible last-bit/latch sequence.
    Ltc8043,

    /// Twelve-bit AD5452 frame with two leading control bits.
    Ad5452,

    /// Sixteen-bit DAC8501/LTC1655 frame with power-control padding.
    Dac8501,

    /// Straight 16-bit DAC8811 frame held under one latch assertion.
    Dac8811,
}
