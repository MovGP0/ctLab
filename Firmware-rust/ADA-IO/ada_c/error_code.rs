//! Defines ADA serial error codes whose numeric values are part of the wire protocol.

#[allow(unused_imports)]
use super::*;

/// Encodes protocol outcomes using the numeric status values expected by existing clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorCode {
    /// Reports successful completion.
    NoErr = 0,

    /// Reports a front-panel user service request.
    UserReq = 1,

    /// Rejects a command while an operation owns the instrument.
    BusyErr = 2,

    /// Reports converter or input overload.
    OvlErr = 3,

    /// Reports malformed command framing or mnemonic syntax.
    SyntaxErr = 4,

    /// Reports a value outside the accepted parameter domain.
    ParamErr = 5,

    /// Rejects an EEPROM-changing command before write enable.
    LockedErr = 6,

    /// Reports an XOR checksum mismatch.
    ChecksumErr = 7,
}

impl ErrorCode {
    /// Returns the bracketed ADA status label emitted beside this numeric error code.
    pub const fn as_str(self) -> &'static str {
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
