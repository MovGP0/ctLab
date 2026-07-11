//! Defines debounced panel events delivered to the foreground state machine.

use super::*;

/// Debounced front-panel event; separating events from polling keeps state transitions deterministic and testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PanelEvent {
    /// Represents a polling iteration with no completed panel action.
    None,

    /// Delivers one signed, debounced encoder movement to the active SQG edit target.
    Encoder(i32),

    /// Delivers one debounced SQG panel button without exposing scan-level contact state.
    Button(PanelButton),

    /// Ends the current encoder gesture so the next movement performs first-turn grid rounding again.
    IncrTimerExpired,

    /// Restores the normal SQG display after the temporary edited-value interval.
    DisplayTimerExpired,
}
