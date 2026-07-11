//! Decodes EDL protocol numbers into semantic getter and setter operations.

use super::{Lm75Sensor, OptionSlot, Shunt, VoltageRange};

/// Names every subchannel implemented by the EDL parser, including typed calibration families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdlSubChannel
{
    /// Reads or changes whether the electronic-load output path is enabled.
    OutputEnable,

    /// Reads or changes the current setpoint in amperes.
    CurrentSetpointAmperes,

    /// Reads or changes the current setpoint in milliamperes.
    CurrentSetpointMilliamperes,

    /// Reads or changes the constant-power setpoint in watts.
    PowerSetpoint,

    /// Reads or changes the low-voltage cutoff threshold in volts.
    LowVoltageCutoff,

    /// Reads or changes the constant-resistance setpoint in ohms.
    ResistanceSetpoint,

    /// Reads accumulated charge or resets both accumulated counters.
    CapacityAmpereHours,

    /// Reads accumulated energy or resets both accumulated counters.
    EnergyWattHours,

    /// Reads the active shunt or changes the manual/automatic shunt request.
    ShuntRange,

    /// Reports terminal voltage during the active ripple phase.
    MeasuredVoltageOn,

    /// Reports terminal current in amperes during the active ripple phase.
    MeasuredCurrentOnAmperes,

    /// Reports terminal current in milliamperes during the active ripple phase.
    MeasuredCurrentOnMilliamperes,

    /// Reports terminal voltage during the inactive ripple phase.
    MeasuredVoltageOff,

    /// Reports terminal current in amperes during the inactive ripple phase.
    MeasuredCurrentOffAmperes,

    /// Reports terminal current in milliamperes during the inactive ripple phase.
    MeasuredCurrentOffMilliamperes,

    /// Reports duty-cycle-weighted load power.
    MeasuredPower,

    /// Reads or changes the regulation law and voltage range.
    Mode,

    /// Reads or changes current modulation as a percentage; wire channels 21 and 22 are aliases.
    CurrentModulationPercent,

    /// Reads or changes the active ripple duration.
    RippleOnTime,

    /// Reads or changes the inactive ripple duration.
    RippleOffTime,

    /// Reads or changes the off-phase current percentage.
    RippleOffCurrentPercent,

    /// Reports the interrupt-owned active-phase voltage ADC16 sample.
    RawVoltageAdc16,

    /// Reports the interrupt-owned active-phase current ADC16 sample.
    RawCurrentAdc16,

    /// Reports the AVR ADC10 voltage diagnostic input.
    RawVoltageAdc10,

    /// Reports the AVR ADC10 current diagnostic input.
    RawCurrentAdc10,

    /// Reads or directly writes the active-phase DAC code for diagnostics.
    RawDacOn,

    /// Reads or directly writes the inactive-phase DAC code for diagnostics.
    RawDacOff,

    /// Reports the DAC code currently selected for output.
    RawDacActive,

    /// Reads or changes the front-panel menu/edit target.
    DisplaySelection,

    /// Reads or changes the EEPROM-backed encoder detent spacing.
    EncoderRaster,

    /// Emits the four active/inactive voltage and current measurements as one response burst.
    AllMeasurements,

    /// Preserves an unused voltage-DAC offset position retained for DCG layout compatibility.
    ReservedVoltageDacOffset(VoltageRange),

    /// Reads or changes the zero correction for one current-DAC shunt.
    CurrentDacOffset(Shunt),

    /// Reads or changes the zero correction for one voltage ADC range.
    VoltageAdcOffset(VoltageRange),

    /// Reads or changes the zero correction for one current-ADC shunt.
    CurrentAdcOffset(Shunt),

    /// Reads or changes one EEPROM-backed calibration, limit, or startup option.
    Option(OptionSlot),

    /// Preserves an unused voltage-DAC scale position retained for DCG layout compatibility.
    ReservedVoltageDacScale(VoltageRange),

    /// Reads or changes the gain correction for one current-DAC shunt.
    CurrentDacScale(Shunt),

    /// Reads or changes the gain correction for one voltage ADC range.
    VoltageAdcScale(VoltageRange),

    /// Reads or changes the gain correction for one current-ADC shunt.
    CurrentAdcScale(Shunt),

    /// Preserves unlocked no-op calibration channels accepted by the Pascal outer range.
    ReservedCalibration(u8),

    /// Reports the temperature measured at one LM75 board location.
    Temperature(Lm75Sensor),

    /// Reads or changes trigger and temperature-sensor enable bits.
    TriggerMask,

    /// Unlocks exactly one protected EEPROM/calibration write and reports status when queried.
    WriteEnable,

    /// Reads or resets the accumulated parser error count.
    ErrorCount,

    /// Reads or changes the EEPROM UART divisor applied after reset.
    SerialBaudDivisor,

    /// Echoes the original input frame for serial-path diagnostics.
    Echo,

    /// Returns the EDL firmware identification string.
    Identification,

    /// Returns the packed status and prompt result.
    Status,
}

impl EdlSubChannel
{
    /// Decodes one validated wire number, centralizing aliases and indexed protocol families.
    pub fn from_wire(value: u16) -> Option<Self>
    {
        match value
        {
            0 => Some(Self::OutputEnable),
            1 => Some(Self::CurrentSetpointAmperes),
            2 => Some(Self::CurrentSetpointMilliamperes),
            3 => Some(Self::PowerSetpoint),
            4 => Some(Self::LowVoltageCutoff),
            5 => Some(Self::ResistanceSetpoint),
            7 => Some(Self::CapacityAmpereHours),
            8 => Some(Self::EnergyWattHours),
            9 => Some(Self::ShuntRange),
            10 => Some(Self::MeasuredVoltageOn),
            11 => Some(Self::MeasuredCurrentOnAmperes),
            12 => Some(Self::MeasuredCurrentOnMilliamperes),
            15 => Some(Self::MeasuredVoltageOff),
            16 => Some(Self::MeasuredCurrentOffAmperes),
            17 => Some(Self::MeasuredCurrentOffMilliamperes),
            18 => Some(Self::MeasuredPower),
            19 => Some(Self::Mode),
            21 | 22 => Some(Self::CurrentModulationPercent),
            27 => Some(Self::RippleOnTime),
            28 => Some(Self::RippleOffTime),
            29 => Some(Self::RippleOffCurrentPercent),
            50 => Some(Self::RawVoltageAdc16),
            51 => Some(Self::RawCurrentAdc16),
            52 => Some(Self::RawVoltageAdc10),
            53 => Some(Self::RawCurrentAdc10),
            70 => Some(Self::RawDacOn),
            71 => Some(Self::RawDacOff),
            72 => Some(Self::RawDacActive),
            80 => Some(Self::DisplaySelection),
            89 => Some(Self::EncoderRaster),
            99 => Some(Self::AllMeasurements),
            100..=101 => VoltageRange::from_index(usize::from(value - 100))
                .map(Self::ReservedVoltageDacOffset),
            102..=105 => Shunt::from_index(usize::from(value - 102))
                .map(Self::CurrentDacOffset),
            110..=111 => VoltageRange::from_index(usize::from(value - 110))
                .map(Self::VoltageAdcOffset),
            112..=115 => Shunt::from_index(usize::from(value - 112))
                .map(Self::CurrentAdcOffset),
            150..=171 => OptionSlot::from_index(usize::from(value - 150)).map(Self::Option),
            200..=201 => VoltageRange::from_index(usize::from(value - 200))
                .map(Self::ReservedVoltageDacScale),
            202..=205 => Shunt::from_index(usize::from(value - 202))
                .map(Self::CurrentDacScale),
            210..=211 => VoltageRange::from_index(usize::from(value - 210))
                .map(Self::VoltageAdcScale),
            212..=215 => Shunt::from_index(usize::from(value - 212))
                .map(Self::CurrentAdcScale),
            216..=223 => Some(Self::ReservedCalibration((value - 216) as u8)),
            233 => Some(Self::Temperature(Lm75Sensor::Internal)),
            234 => Some(Self::Temperature(Lm75Sensor::External)),
            240 => Some(Self::TriggerMask),
            250 => Some(Self::WriteEnable),
            251 => Some(Self::ErrorCount),
            252 => Some(Self::SerialBaudDivisor),
            253 => Some(Self::Echo),
            254 => Some(Self::Identification),
            255 => Some(Self::Status),
            _ => None,
        }
    }

    /// Returns the canonical wire number used when firmware emits this operation itself.
    ///
    /// Incoming aliases remain distinguishable through [`super::ResolvedSubChannel::wire_value`];
    /// this mapping selects the first Pascal number only for newly generated subchannel replies.
    pub fn canonical_wire_value(self) -> u16
    {
        match self
        {
            Self::OutputEnable => 0,
            Self::CurrentSetpointAmperes => 1,
            Self::CurrentSetpointMilliamperes => 2,
            Self::PowerSetpoint => 3,
            Self::LowVoltageCutoff => 4,
            Self::ResistanceSetpoint => 5,
            Self::CapacityAmpereHours => 7,
            Self::EnergyWattHours => 8,
            Self::ShuntRange => 9,
            Self::MeasuredVoltageOn => 10,
            Self::MeasuredCurrentOnAmperes => 11,
            Self::MeasuredCurrentOnMilliamperes => 12,
            Self::MeasuredVoltageOff => 15,
            Self::MeasuredCurrentOffAmperes => 16,
            Self::MeasuredCurrentOffMilliamperes => 17,
            Self::MeasuredPower => 18,
            Self::Mode => 19,
            Self::CurrentModulationPercent => 21,
            Self::RippleOnTime => 27,
            Self::RippleOffTime => 28,
            Self::RippleOffCurrentPercent => 29,
            Self::RawVoltageAdc16 => 50,
            Self::RawCurrentAdc16 => 51,
            Self::RawVoltageAdc10 => 52,
            Self::RawCurrentAdc10 => 53,
            Self::RawDacOn => 70,
            Self::RawDacOff => 71,
            Self::RawDacActive => 72,
            Self::DisplaySelection => 80,
            Self::EncoderRaster => 89,
            Self::AllMeasurements => 99,
            Self::ReservedVoltageDacOffset(range) => 100 + range.index() as u16,
            Self::CurrentDacOffset(shunt) => 102 + shunt.index() as u16,
            Self::VoltageAdcOffset(range) => 110 + range.index() as u16,
            Self::CurrentAdcOffset(shunt) => 112 + shunt.index() as u16,
            Self::Option(slot) => 150 + slot.index() as u16,
            Self::ReservedVoltageDacScale(range) => 200 + range.index() as u16,
            Self::CurrentDacScale(shunt) => 202 + shunt.index() as u16,
            Self::VoltageAdcScale(range) => 210 + range.index() as u16,
            Self::CurrentAdcScale(shunt) => 212 + shunt.index() as u16,
            Self::ReservedCalibration(offset) => 216 + u16::from(offset),
            Self::Temperature(Lm75Sensor::Internal) => 233,
            Self::Temperature(Lm75Sensor::External) => 234,
            Self::TriggerMask => 240,
            Self::WriteEnable => 250,
            Self::ErrorCount => 251,
            Self::SerialBaudDivisor => 252,
            Self::Echo => 253,
            Self::Identification => 254,
            Self::Status => 255,
        }
    }
}
