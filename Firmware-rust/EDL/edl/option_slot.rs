//! Names the fixed positions in the EDL EEPROM option image.

/// Identifies one persisted EDL calibration or startup value.
///
/// The discriminants preserve the Pascal `OptionArray` layout so protected
/// serial calibration commands and typed runtime accessors address identical
/// EEPROM cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OptionSlot {
    /// Startup low-voltage cutoff in volts.
    InitialVoltage = 0,

    /// Startup constant-current setpoint in amperes.
    InitialCurrent = 1,

    /// Voltage divider ratio used by low-voltage modes.
    LowVoltageDivider = 2,

    /// Voltage divider ratio used by high-voltage modes.
    HighVoltageDivider = 3,

    /// Current-measurement amplifier gain.
    CurrentMeasurementGain = 4,

    /// ADC and DAC reference voltage.
    ReferenceVoltage = 5,

    /// Maximum permitted load power.
    MaximumPower = 6,

    /// Sense resistance for shunt A.
    SenseResistanceA = 7,

    /// Sense resistance for shunt B.
    SenseResistanceB = 8,

    /// Sense resistance for shunt C.
    SenseResistanceC = 9,

    /// Sense resistance for shunt D.
    SenseResistanceD = 10,

    /// Maximum current for shunt A.
    MaximumCurrentA = 11,

    /// Maximum current for shunt B.
    MaximumCurrentB = 12,

    /// Maximum current for shunt C.
    MaximumCurrentC = 13,

    /// Maximum current for shunt D.
    MaximumCurrentD = 14,

    /// Over-voltage limit used by high-voltage modes.
    HighVoltageLimit = 15,

    /// Over-voltage limit used by low-voltage modes.
    LowVoltageLimit = 16,

    /// Packed DAC and temperature-sensor hardware options.
    InstalledHardware = 17,

    /// Startup off-phase current as a percentage of the active current.
    InitialCurrentPercent = 18,

    /// Startup duration of the active ripple phase.
    InitialRippleOnTime = 19,

    /// Startup duration of the reduced-current ripple phase.
    InitialRippleOffTime = 20,

    /// Temperature at which the fan output is requested.
    FanOnTemperature = 21,
}

impl OptionSlot {
    /// Number of cells in the persisted option image.
    pub const COUNT: usize = 22;

    /// Returns the EEPROM array index encoded by this schema entry.
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Resolves a persisted array position without allowing an invalid option index.
    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::InitialVoltage),
            1 => Some(Self::InitialCurrent),
            2 => Some(Self::LowVoltageDivider),
            3 => Some(Self::HighVoltageDivider),
            4 => Some(Self::CurrentMeasurementGain),
            5 => Some(Self::ReferenceVoltage),
            6 => Some(Self::MaximumPower),
            7 => Some(Self::SenseResistanceA),
            8 => Some(Self::SenseResistanceB),
            9 => Some(Self::SenseResistanceC),
            10 => Some(Self::SenseResistanceD),
            11 => Some(Self::MaximumCurrentA),
            12 => Some(Self::MaximumCurrentB),
            13 => Some(Self::MaximumCurrentC),
            14 => Some(Self::MaximumCurrentD),
            15 => Some(Self::HighVoltageLimit),
            16 => Some(Self::LowVoltageLimit),
            17 => Some(Self::InstalledHardware),
            18 => Some(Self::InitialCurrentPercent),
            19 => Some(Self::InitialRippleOnTime),
            20 => Some(Self::InitialRippleOffTime),
            21 => Some(Self::FanOnTemperature),
            _ => None,
        }
    }
}
