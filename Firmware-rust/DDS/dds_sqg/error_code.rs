#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ErrorCode {
    NoErr,
    UserReq,
    BusyErr,
    OvlErr,
    SyntaxErr,
    ParamErr,
    LockedErr,
    ChecksumErr,
}
impl ErrorCode {
    pub(super) fn code(self) -> u8 {
        self as u8
    }
}
