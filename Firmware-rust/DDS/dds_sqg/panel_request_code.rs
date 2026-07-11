//! Defines the panel action codes carried in SQG user-service-request responses.

/// Front-panel action encoded in the low bits of a user-service-request status byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum PanelRequestCode {
    /// Reports that the encoder gesture ended and the panel returned to its released state.
    Released = 64,

    /// Reports a left-button action after debounce.
    Left = 65,

    /// Reports a right-button action after debounce.
    Right = 66,

    /// Reports the first encoder detent or Enter action that made the panel active.
    PanelActive = 67,
}

impl PanelRequestCode {
    /// Returns the exact legacy status contribution added to the current operating flags.
    pub(super) const fn as_byte(self) -> u8 {
        self as u8
    }
}
