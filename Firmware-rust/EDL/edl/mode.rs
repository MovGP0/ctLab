#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// Operating mode persisted and reported as the Pascal-compatible numeric value.
pub enum Mode {
    /// Opens the load path and forces the programmed current to zero.
    OutputOff = 0,

    /// Constant-current regulation using the high-voltage divider.
    IHiVolt = 1,

    /// Constant-current regulation using the more sensitive low-voltage divider.
    ILoVolt = 2,

    /// Constant-resistance regulation using high-voltage measurements.
    RHiVolt = 3,

    /// Constant-resistance regulation using low-voltage measurements.
    RLoVolt = 4,

    /// Constant-power regulation using high-voltage measurements.
    PHiVolt = 5,

    /// Constant-power regulation using low-voltage measurements.
    PLoVolt = 6,
}

impl Mode {
    /// Validates a wire/EEPROM mode byte instead of accepting undefined hardware states.
    pub fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Self::OutputOff),
            1 => Some(Self::IHiVolt),
            2 => Some(Self::ILoVolt),
            3 => Some(Self::RHiVolt),
            4 => Some(Self::RLoVolt),
            5 => Some(Self::PHiVolt),
            6 => Some(Self::PLoVolt),
            _ => None,
        }
    }

    /// Selects the low-range divider and its corresponding safety limit.
    pub fn is_low_voltage(self) -> bool {
        matches!(self, Self::ILoVolt | Self::RLoVolt | Self::PLoVolt)
    }

    /// Identifies modes whose setpoint directly represents amperes.
    pub fn is_current(self) -> bool {
        matches!(self, Self::IHiVolt | Self::ILoVolt)
    }

    /// Identifies modes that derive current from measured voltage and requested resistance.
    pub fn is_resistance(self) -> bool {
        matches!(self, Self::RHiVolt | Self::RLoVolt)
    }

    /// Identifies modes that derive current from requested power and measured voltage.
    pub fn is_power(self) -> bool {
        matches!(self, Self::PHiVolt | Self::PLoVolt)
    }
}
