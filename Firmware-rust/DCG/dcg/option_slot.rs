//! Names the fixed positions in the DCG EEPROM option image.

/// Identifies one persisted DCG calibration or startup value.
///
/// The discriminants intentionally match the Pascal `OptionArray` layout so
/// serial option commands and typed firmware accessors read and write the same
/// EEPROM cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OptionSlot {
    /// Startup output-voltage setpoint in volts.
    InitialVoltage = 0,

    /// Startup current-limit setpoint in amperes.
    InitialCurrent = 1,

    /// Voltage-feedback preamplifier gain.
    PreamplifierGain = 2,

    /// Output-stage voltage-sense gain.
    OutputStageGain = 3,

    /// Current-measurement divider gain.
    CurrentMeasurementGain = 4,

    /// ADC and DAC reference voltage.
    ReferenceVoltage = 5,

    /// Maximum permitted output voltage.
    MaximumVoltage = 6,

    /// Sense resistance for the 2 mA range.
    SenseResistance2mA = 7,

    /// Sense resistance for the 20 mA range.
    SenseResistance20mA = 8,

    /// Sense resistance for the 200 mA range.
    SenseResistance200mA = 9,

    /// Sense resistance for the 2 A range.
    SenseResistance2A = 10,

    /// Full-scale current for the 2 mA range.
    MaximumCurrent2mA = 11,

    /// Full-scale current for the 20 mA range.
    MaximumCurrent20mA = 12,

    /// Full-scale current for the 200 mA range.
    MaximumCurrent200mA = 13,

    /// Full-scale current for the 2 A range.
    MaximumCurrent2A = 14,

    /// Voltage-feedback divider factor for the low range.
    LowVoltageAdcDivider = 15,

    /// Voltage-feedback divider factor for the high range.
    HighVoltageAdcDivider = 16,

    /// Packed flags describing installed optional hardware.
    InstalledHardware = 17,

    /// Voltage at which the DAC voltage path changes range.
    VoltageRangeSwitchpoint = 18,

    /// Lower relay threshold used when returning to the low-voltage supply.
    RelayHysteresisLow = 19,

    /// Upper relay threshold used when switching to the high-voltage supply.
    RelayHysteresisHigh = 20,

    /// Temperature at which the cooling fan is requested.
    FanOnTemperature = 21,

    /// Startup ripple depth as a percentage of the energized voltage.
    InitialRipplePercent = 22,

    /// Startup duration of the energized ripple phase.
    InitialRippleOnTime = 23,

    /// Startup duration of the reduced-voltage ripple phase.
    InitialRippleOffTime = 24,
}

impl OptionSlot {
    /// Number of cells in the persisted option image.
    pub const COUNT: usize = 25;

    /// Returns the EEPROM array index encoded by this schema entry.
    pub const fn index(self) -> usize {
        self as usize
    }
}
