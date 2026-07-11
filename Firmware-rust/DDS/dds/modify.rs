//! Defines the finite ring of front-panel values that encoder and navigation actions may edit.

/// Front-panel editing target; this state determines which setpoint an encoder movement is allowed to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Modify {
    /// Edits waveform routing.
    WaveSel = 0,

    /// Edits frequency using coarse preferred values or fine steps.
    FreqSel = 1,

    /// Edits output amplitude.
    AmplSel = 2,

    /// Edits peak-level representation.
    PeakSel = 3,

    /// Selects the input measurement page.
    InpSel = 4,

    /// Edits burst duration.
    BurstSel = 5,

    /// Edits output DC offset.
    DcSel = 6,
}
impl Modify {
    /// Decodes the persisted/wire byte into its typed state, mapping unsupported values to the safe disabled or error sentinel.
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

    /// Advances to the next panel edit target with explicit wraparound, matching the finite menu ring.
    pub(super) fn next(self) -> Self {
        Self::from_byte((self as u8 + 1) % 7).unwrap_or(Self::WaveSel)
    }

    /// Moves backward through the panel edit ring without unsigned underflow at the first target.
    pub(super) fn prev(self) -> Self {
        Self::from_byte((self as u8 + 6) % 7).unwrap_or(Self::WaveSel)
    }
}
