//! Defines the fixed AD9833 control words shared by DDS and SQG output programming.

/// Complete AD9833 control-register word for one supported generator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Ad9833Control {
    /// Holds the DDS in reset while frequency words or relay routes change.
    Reset = 0b0010_0001_0000_0000,

    /// Releases reset with sine output selected.
    Sine = 0b0010_0000_0000_0000,

    /// Releases reset with triangle output selected.
    Triangle = 0b0010_0000_0000_0010,

    /// Releases reset with sign-bit square output and its required divider mode.
    Square = 0b0010_0000_0010_1000,
}

impl Ad9833Control {
    /// Returns the exact sixteen-bit word shifted into the AD9833 control register.
    pub const fn as_word(self) -> u16 {
        self as u16
    }
}
