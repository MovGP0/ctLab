//! Defines ACV front-panel edit and display modes.

#[allow(unused_imports)]
use super::*;

/// Selects the ACV front-panel value or visualization currently being edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Modify {
    /// Edits the auxiliary serial command byte on the ACV panel.
    AuxCmdSel,

    /// Edits the S/PDIF sample rate on the ACV panel.
    RateSel,

    /// Edits the programmable input gain on the ACV panel.
    GainSel,

    /// Displays both channels as fixed-width level bars.
    LevelBarDispl,

    /// Displays calibrated channel levels in millivolts.
    MvDispl,
}
