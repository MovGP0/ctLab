//! Assigns the error nibble and diagnostic meaning transmitted by legacy status frames.

/// Protocol-visible result code. Numeric ordering is preserved because the low status nibble is transmitted directly to controllers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorCode {
    /// Indicates successful processing without adding an error nibble.
    NoErr = 0,

    /// Reports local panel activity to remote controllers as a service request.
    UserReq = 1,

    /// Rejects a mutating command while local operation owns the hardware.
    BusyErr = 2,

    /// Reports an active protection overload.
    OvlErr = 3,

    /// Reports a command that cannot be tokenized or addressed.
    SyntaxErr = 4,

    /// Reports a parsed value that is invalid or had to be clamped.
    ParamErr = 5,

    /// Rejects an EEPROM write without the write-enable latch.
    LockedErr = 6,

    /// Rejects a frame whose XOR suffix does not match.
    ChecksumErr = 7,

    /// Reports loss of the protected supply path.
    FuseErr = 8,

    /// Reports a hardware fault not represented by a narrower code.
    FaultErr = 9,
}
impl ErrorCode {
    /// Returns the exact status label emitted by the Pascal-compatible DCG response.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoErr => "[OK]",
            Self::UserReq => "[SRQUSR]",
            Self::BusyErr => "[BUSY]",
            Self::OvlErr => "[OVRLD]",
            Self::SyntaxErr => "[CMDERR]",
            Self::ParamErr => "[PARERR]",
            Self::LockedErr => "[LOCKED]",
            Self::ChecksumErr | Self::FuseErr | Self::FaultErr => "[CHKSUM]",
        }
    }
}
