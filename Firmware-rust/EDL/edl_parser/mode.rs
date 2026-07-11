#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    OutputOff,
    IhiVolt,
    IloVolt,
    RhiVolt,
    RloVolt,
    PhiVolt,
    PloVolt,
    Unknown(u8),
}

impl From<u8> for Mode {
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
