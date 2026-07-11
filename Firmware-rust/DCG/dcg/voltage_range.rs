//! DC-generator output-voltage divider ranges.

/// Selects the output-voltage path used by ADC scaling, DAC calibration, and relay switching.
///
/// Keeping the divider state explicit ensures a setpoint, its feedback sample,
/// and the calibration coefficients all refer to the same analog path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VoltageRange {
    /// Uses the low-voltage path for finer resolution below the EEPROM switch point.
    ULow,

    /// Uses the high-voltage path when the setpoint exceeds the EEPROM switch point.
    UHigh,
}
