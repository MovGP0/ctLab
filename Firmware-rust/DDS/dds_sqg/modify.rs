//! Defines the finite ring of front-panel values that encoder and navigation actions may edit.

/// Front-panel editing target; this state determines which setpoint an encoder movement is allowed to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Modify {
    /// Edits waveform routing.
    WaveSel,

    /// Edits frequency using coarse preferred values or fine steps.
    FreqSel,

    /// Edits output amplitude.
    AmplSel,

    /// Edits burst duration.
    BurstSel,

    /// Edits output DC offset.
    DcSel,
}
