//! Distinguishes successful parser work from syntax, parameter, checksum, and EEPROM-lock failures.

/// Parser result used to distinguish malformed input, locked EEPROM writes, and values corrected by limit enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Error {
    /// Indicates successful processing without adding an error nibble.
    NoErr = 0,

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

    /// Reports loss of the protected supply path.
    FuseErr,

    /// Reports a hardware fault not represented by a narrower code.
    FaultErr,
}
impl Error {
    /// Returns the exact status label associated with the parser result.
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
