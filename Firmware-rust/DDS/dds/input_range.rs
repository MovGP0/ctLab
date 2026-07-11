//! Alternating-voltage ranges for the DDS module's measurement input.

/// Selects the full-scale alternating-voltage range applied to input measurements.
///
/// The discriminants index the gain and calibration tables and are sent to the
/// range-selection hardware, so their order is part of the firmware protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InputRange {
    /// Measures alternating voltage up to 100 millivolts.
    Ac100mV = 0,

    /// Measures alternating voltage up to 1 volt.
    Ac1V = 1,

    /// Measures alternating voltage up to 10 volts.
    Ac10V = 2,

    /// Measures alternating voltage up to 100 volts.
    Ac100V = 3,

    /// Sentinel forcing the first real range selection to program hardware.
    NoRange = 4,
}
impl InputRange {
    /// Decodes the EEPROM/protocol range byte, returning [`Self::NoRange`] for
    /// invalid values so the next limit check forces a real hardware selection.
    pub fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::Ac100mV,
            1 => Self::Ac1V,
            2 => Self::Ac10V,
            3 => Self::Ac100V,
            _ => Self::NoRange,
        }
    }

    /// Returns the EEPROM/protocol value associated with this range.
    pub fn as_byte(self) -> u8 {
        self as u8
    }
}
