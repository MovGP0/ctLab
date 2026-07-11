#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Modify {
    Ampere = 0,
    Volt = 1,
    Ripple = 2,
    TOn = 3,
    TOff = 4,
    TrackCh = 5,
    CapMenu = 6,
    PwrMenu = 7,
}
impl Modify {
    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Ampere),
            1 => Some(Self::Volt),
            2 => Some(Self::Ripple),
            3 => Some(Self::TOn),
            4 => Some(Self::TOff),
            5 => Some(Self::TrackCh),
            6 => Some(Self::CapMenu),
            7 => Some(Self::PwrMenu),
            _ => None,
        }
    }
}
