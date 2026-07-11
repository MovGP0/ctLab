//! Names the debounced SQG buttons consumed by the panel state machine.

/// Physical SQG panel button after scan/debounce processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PanelButton {
    /// Confirms or advances the active SQG panel selection.
    Enter,

    /// Moves the SQG edit cursor or selection toward the previous item.
    Left,

    /// Moves the SQG edit cursor or selection toward the next item.
    Right,
}
