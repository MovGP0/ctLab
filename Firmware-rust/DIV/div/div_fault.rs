//! Typed overload causes reported by the DIV status protocol.

/// Identifies an analog-input overload independently of its packed status bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DivFault {
    /// The converted input exceeded the negative end of the selected range.
    NegativeOverload = 0,

    /// The converted input exceeded the positive end of the selected range.
    PositiveOverload = 1,
}

impl DivFault {
    /// Returns the bit used for this fault in the low nibble of a status response.
    pub const fn mask(self) -> u8 {
        1 << self as u8
    }

    /// Returns the bracketed diagnostic appended when this fault is active.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NegativeOverload => "[OVRNEG]",
            Self::PositiveOverload => "[OVRPOS]",
        }
    }
}
