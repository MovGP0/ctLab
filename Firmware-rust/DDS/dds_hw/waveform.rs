//! Maps waveform choices to AD9833 control modes and board relay routes.

/// Waveform routing selection used by DDS control words and output relay logic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Waveform {
    /// Disables generated output while leaving the controller responsive.
    Off,

    /// Programs the AD9833 sine mode and leaves the square/logic routing relays inactive.
    Sine,

    /// Programs the AD9833 triangle bit while retaining the selected frequency register.
    Triangle,

    /// Routes the comparator-derived square output through the board's square-wave relay.
    Square,

    /// Routes the logic-level square output instead of the analog waveform path.
    Logic,

    /// Selects an externally numbered waveform/relay route; the payload retains the board-specific route index.
    External(u8),
}
