#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CurrentRange {
    Dc2mA,
    Dc20mA,
    Dc200mA,
    Dc2A,
}
impl CurrentRange {
    pub(super) fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Dc2mA,
            1 => Self::Dc20mA,
            2 => Self::Dc200mA,
            _ => Self::Dc2A,
        }
    }
}
