//! Defines debounced panel events delivered to the foreground state machine.

/// Debounced front-panel event; separating events from polling keeps state transitions deterministic and testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelEvent {
    /// Delivers the signed, debounced detent movement that will be applied to the active edit target.
    EncoderDelta(i16),

    /// Switches the active encoder between coarse and fine resolution.
    ToggleFine,

    /// Advances the panel edit target.
    NextModify,

    /// Moves to the previous panel edit target.
    PrevModify,

    /// Delivers one coherent debounced button scan so simultaneous contacts are handled together.
    Buttons {
        /// Indicates that Enter was pressed to confirm or advance the current panel selection.
        enter: bool,

        /// Indicates that Left was pressed to move toward the previous cursor position or edit target.
        left: bool,

        /// Indicates that Right was pressed to move toward the next cursor position or edit target.
        right: bool,
    },

    /// Signals the pause that ends one encoder gesture.
    IncrTimerElapsed,

    /// Signals expiry of the temporary setpoint display.
    DisplayTimerElapsed,

    /// Clears the busy latch after local interaction.
    ReleaseBusy,
}
