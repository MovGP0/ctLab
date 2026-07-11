#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Parser-side mode retaining unknown wire values for explicit validation.
pub enum Mode {
    /// Output disabled.
    OutputOff,

    /// Constant current on the high-voltage range.
    IhiVolt,

    /// Constant current on the low-voltage range.
    IloVolt,

    /// Constant resistance on the high-voltage range.
    RhiVolt,

    /// Constant resistance on the low-voltage range.
    RloVolt,

    /// Constant power on the high-voltage range.
    PhiVolt,

    /// Constant power on the low-voltage range.
    PloVolt,

    /// Unrecognized mode byte retained so setters can return a parameter error.
    Unknown(u8),
}

impl From<u8> for Mode {
    /// Maps all defined Pascal mode bytes and preserves unknown values for later rejection.
    fn from(value: u8) -> Self {
        match value {
            0 => Self::OutputOff,
            1 => Self::IhiVolt,
            2 => Self::IloVolt,
            3 => Self::RhiVolt,
            4 => Self::RloVolt,
            5 => Self::PhiVolt,
            6 => Self::PloVolt,
            other => Self::Unknown(other),
        }
    }
}
