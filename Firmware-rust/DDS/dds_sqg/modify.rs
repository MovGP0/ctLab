//! Defines the finite ring of front-panel values that encoder and navigation actions may edit.

/// Front-panel editing target; this state determines which setpoint an encoder movement is allowed to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum Modify {
    /// Edits waveform routing.
    Waveform = 0,

    /// Edits frequency using coarse preferred values or fine steps.
    Frequency = 1,

    /// Edits output amplitude.
    Amplitude = 2,

    /// Edits burst duration.
    Burst = 3,

    /// Edits output DC offset.
    DcOffset = 4,
}

impl Modify {
    /// Converts the wire/display selector into a panel edit target.
    pub(super) const fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Waveform),
            1 => Some(Self::Frequency),
            2 => Some(Self::Amplitude),
            3 => Some(Self::Burst),
            4 => Some(Self::DcOffset),
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
            Self::Frequency => 0,
            Self::Amplitude => 1,
            Self::Waveform => 4,
            Self::Burst => 5,
            Self::DcOffset => 20,
        }
    }

    /// Advances through the finite SQG panel menu with explicit wraparound.
    pub(super) const fn next(self) -> Self {
        match self {
            Self::Waveform => Self::Frequency,
            Self::Frequency => Self::Amplitude,
            Self::Amplitude => Self::Burst,
            Self::Burst => Self::DcOffset,
            Self::DcOffset => Self::Waveform,
        }
    }

    /// Moves backward through the finite SQG panel menu with explicit wraparound.
    pub(super) const fn previous(self) -> Self {
        match self {
            Self::Waveform => Self::DcOffset,
            Self::Frequency => Self::Waveform,
            Self::Amplitude => Self::Frequency,
            Self::Burst => Self::Amplitude,
            Self::DcOffset => Self::Burst,
        }
    }
}
