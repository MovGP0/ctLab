//! Defines the finite ring of front-panel values that encoder and navigation actions may edit.

/// Front-panel editing target; this state determines which setpoint an encoder movement is allowed to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum Modify {
    /// Edits waveform routing.
    WaveSel = 0,

    /// Edits frequency using coarse preferred values or fine steps.
    FreqSel = 1,

    /// Edits output amplitude.
    AmplSel = 2,

    /// Edits burst duration.
    BurstSel = 3,

    /// Edits output DC offset.
    DcSel = 4,
}

impl Modify {
    /// Converts the wire/display selector into a panel edit target.
    pub(super) const fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::WaveSel),
            1 => Some(Self::FreqSel),
            2 => Some(Self::AmplSel),
            3 => Some(Self::BurstSel),
            4 => Some(Self::DcSel),
            _ => None,
        }
    }

    /// Returns the selector byte reported by the `DSP` protocol subchannel.
    pub(super) const fn as_byte(self) -> u8 {
        self as u8
    }

    /// Returns the serial subchannel for the value edited by this panel target.
    pub(super) const fn subchannel(self) -> u8 {
        match self {
            Self::FreqSel => 0,
            Self::AmplSel => 1,
            Self::WaveSel => 4,
            Self::BurstSel => 5,
            Self::DcSel => 20,
        }
    }

    /// Advances through the finite SQG panel menu with explicit wraparound.
    pub(super) const fn next(self) -> Self {
        match self {
            Self::WaveSel => Self::FreqSel,
            Self::FreqSel => Self::AmplSel,
            Self::AmplSel => Self::BurstSel,
            Self::BurstSel => Self::DcSel,
            Self::DcSel => Self::WaveSel,
        }
    }

    /// Moves backward through the finite SQG panel menu with explicit wraparound.
    pub(super) const fn previous(self) -> Self {
        match self {
            Self::WaveSel => Self::DcSel,
            Self::FreqSel => Self::WaveSel,
            Self::AmplSel => Self::FreqSel,
            Self::BurstSel => Self::AmplSel,
            Self::DcSel => Self::BurstSel,
        }
    }
}
