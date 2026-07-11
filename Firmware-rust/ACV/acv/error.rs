//! Defines ACV serial error codes whose numeric values are part of the wire protocol.

#[allow(unused_imports)]
use super::*;

/// Encodes protocol outcomes using the numeric status values expected by existing clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Error {
    /// Reports successful completion.
    NoErr,

    /// Reports a front-panel user service request.
    UserReq,

    /// Rejects a command while an operation owns the instrument.
    BusyErr,

    /// Reports converter or input overload.
    OvlErr,

    /// Reports malformed command framing or mnemonic syntax.
    SyntaxErr,

    /// Reports a value outside the accepted parameter domain.
    ParamErr,

    /// Rejects an EEPROM-changing command before write enable.
    LockedErr,

    /// Reports an XOR checksum mismatch.
    ChecksumErr,
}

impl Error {
    /// Returns the bracketed ACV status label emitted beside this numeric error code.
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
