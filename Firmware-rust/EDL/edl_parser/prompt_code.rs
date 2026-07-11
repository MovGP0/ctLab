#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptCode {
    NoErr,
    ParamErr,
    BusyErr,
    LockedErr,
    CheckLimitErr,
    CheckSumErr,
    SyntaxErr,
}

impl PromptCode {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::NoErr => "NoErr",
            Self::ParamErr => "ParamErr",
            Self::BusyErr => "BusyErr",
            Self::LockedErr => "LockedErr",
            Self::CheckLimitErr => "CheckLimitErr",
            Self::CheckSumErr => "CheckSumErr",
            Self::SyntaxErr => "SyntaxErr",
        }
    }
}
