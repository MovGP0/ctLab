//! Names the relay-shadow outputs controlled by the two-register SQG board.

/// Relay or routing output stored in the SQG shift-register shadow byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum SwitchOutput {
    /// Routes the comparator-derived square waveform to the output stage.
    Square = 4,

    /// Selects the attenuated amplitude range for better low-level resolution.
    Attenuator = 5,

    /// Enables the external-source output route.
    External = 6,

    /// Connects the offset DAC path when a DC offset or logic level is required.
    Offset = 7,
}

impl SwitchOutput {
    /// Returns the shift-register mask changed when this output is enabled or disabled.
    pub(super) const fn mask(self) -> u8 {
        1 << self as u8
    }
}
