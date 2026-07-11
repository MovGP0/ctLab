#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InputRange {
    Ac100mV = 0,
    Ac1V = 1,
    Ac10V = 2,
    Ac100V = 3,
    NoRange = 4,
}
impl InputRange {
    pub fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::Ac100mV,
            1 => Self::Ac1V,
            2 => Self::Ac10V,
            3 => Self::Ac100V,
            _ => Self::NoRange,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }
}
