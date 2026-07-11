#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modify {
    LowerMainMenu,
    UpperMainMenu,
    ModeMenu,
    TOn,
    TOff,
    IOff,
    TempMenu,
    RiMenu,
    CapMenu,
    PwrCurMenu,
    Unknown(u8),
}

impl From<u8> for Modify {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::LowerMainMenu,
            1 => Self::UpperMainMenu,
            2 => Self::ModeMenu,
            3 => Self::TOn,
            4 => Self::TOff,
            5 => Self::IOff,
            6 => Self::TempMenu,
            7 => Self::RiMenu,
            8 => Self::CapMenu,
            9 => Self::PwrCurMenu,
            other => Self::Unknown(other),
        }
    }
}
