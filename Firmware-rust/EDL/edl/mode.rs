#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    OutputOff = 0,
    IHiVolt = 1,
    ILoVolt = 2,
    RHiVolt = 3,
    RLoVolt = 4,
    PHiVolt = 5,
    PLoVolt = 6,
}

impl Mode {
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

    pub fn is_low_voltage(self) -> bool {
        matches!(self, Self::ILoVolt | Self::RLoVolt | Self::PLoVolt)
    }

    pub fn is_current(self) -> bool {
        matches!(self, Self::IHiVolt | Self::ILoVolt)
    }

    pub fn is_resistance(self) -> bool {
        matches!(self, Self::RHiVolt | Self::RLoVolt)
    }

    pub fn is_power(self) -> bool {
        matches!(self, Self::PHiVolt | Self::PLoVolt)
    }
}
