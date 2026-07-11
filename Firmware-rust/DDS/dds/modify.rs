#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Modify {
    WaveSel = 0,
    FreqSel = 1,
    AmplSel = 2,
    PeakSel = 3,
    InpSel = 4,
    BurstSel = 5,
    DcSel = 6,
}
impl Modify {
    pub(super) fn from_byte(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::WaveSel,
            1 => Self::FreqSel,
            2 => Self::AmplSel,
            3 => Self::PeakSel,
            4 => Self::InpSel,
            5 => Self::BurstSel,
            6 => Self::DcSel,
            _ => return None,
        })
    }

    pub(super) fn next(self) -> Self {
        Self::from_byte((self as u8 + 1) % 7).unwrap_or(Self::WaveSel)
    }

    pub(super) fn prev(self) -> Self {
        Self::from_byte((self as u8 + 6) % 7).unwrap_or(Self::WaveSel)
    }
}
