#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// Protocol error values whose discriminants index the legacy error string table.
pub enum ErrorCode {
    /// Command completed without error.
    NoErr = 0,

    /// User-generated service request.
    UserReq = 1,

    /// Command rejected while timing-sensitive work is active.
    BusyErr = 2,

    /// Output protection or calibrated limit overload.
    OvlErr = 3,

    /// Command or frame syntax was not recognized.
    SyntaxErr = 4,

    /// Parameter was missing, invalid, or clamped.
    ParamErr = 5,

    /// Calibration write attempted without EEPROM unlock.
    LockedErr = 6,

    /// Supplied XOR checksum did not match.
    ChecksumErr = 7,
}

impl ErrorCode {
    /// Returns the exact bracketed diagnostic appended to an EDL status response.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoErr => "[OK]",
            Self::UserReq => "[SRQUSR]",
            Self::BusyErr => "[BUSY]",
            Self::OvlErr => "[OVRLD]",
            Self::SyntaxErr => "[CMDERR]",
            Self::ParamErr => "[PARERR]",
            Self::LockedErr => "[LOCKED]",
            Self::ChecksumErr => "[CHKSUM]",
        }
    }
}
