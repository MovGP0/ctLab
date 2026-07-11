#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Off,
    Sine,
    Triangle,
    Square,
    Logic,
    External(u8),
}
impl Waveform {
    pub fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::Sine,
            2 => Self::Triangle,
            3 => Self::Square,
            4 => Self::Logic,
            5..=249 => Self::External(value - 5),
            _ => Self::Off,
        }
    }

    pub fn as_byte(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Sine => 1,
            Self::Triangle => 2,
            Self::Square => 3,
            Self::Logic => 4,
            Self::External(index) => 5u8.saturating_add(index),
        }
    }
}
