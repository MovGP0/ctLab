//! Assigns the error nibble and diagnostic meaning transmitted by legacy status frames.

/// Protocol-visible result code. Numeric ordering is preserved because the low status nibble is transmitted directly to controllers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ErrorCode {
    /// Indicates successful processing without adding an error nibble.
    NoErr,

    /// Reports local panel activity to remote controllers as a service request.
    UserReq,

    /// Rejects a mutating command while local operation owns the hardware.
    BusyErr,

    /// Reports an active protection overload.
    OvlErr,

    /// Reports a command that cannot be tokenized or addressed.
    SyntaxErr,

    /// Reports a parsed value that is invalid or had to be clamped.
    ParamErr,

    /// Rejects an EEPROM write without the write-enable latch.
    LockedErr,

    /// Rejects a frame whose XOR suffix does not match.
    ChecksumErr,
}
impl ErrorCode {
    /// Packs the latched condition into its assigned protocol bit or error-code position for the status response.
    pub(super) fn code(self) -> u8 {
        self as u8
    }

    /// Returns the exact status label emitted by the Pascal-compatible SQG response.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::NoErr => "[OK]",
            Self::UserReq => "[SRQUSR]",
            Self::BusyErr => "[BUSY]",
            Self::OvlErr => "[OVERLD]",
            Self::SyntaxErr => "[CMDERR]",
            Self::ParamErr => "[PARERR]",
            Self::LockedErr => "[LOCKED]",
            Self::ChecksumErr => "[CHKSUM]",
        }
    }
}
