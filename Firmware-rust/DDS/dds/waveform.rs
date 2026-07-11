//! Maps waveform choices to AD9833 control modes and board relay routes.

/// Waveform routing selection used by DDS control words and output relay logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
impl Waveform {
    /// Decodes the persisted/wire byte into its typed state, mapping unsupported values to the safe disabled or error sentinel.
    pub fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::Sine,
            2 => Self::Triangle,
            3 => Self::Square,
            4 => Self::Logic,
            5..=249 => Self::External(value - 5),
            _ => Self::Off,
        }
    }

    /// Encodes the typed state using the byte values retained by EEPROM and the serial protocol.
    pub fn as_byte(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Sine => 1,
            Self::Triangle => 2,
            Self::Square => 3,
            Self::Logic => 4,
            Self::External(index) => 5u8.saturating_add(index),
        }
    }
}
