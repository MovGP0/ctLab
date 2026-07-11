#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Parser outcome converted to the exact legacy prompt label when verbose mode is active.
pub enum PromptCode {
    /// Successful command.
    NoErr,

    /// Missing, malformed, or out-of-range parameter.
    ParamErr,

    /// Command rejected while firmware state is busy.
    BusyErr,

    /// EEPROM/calibration setter attempted without unlock.
    LockedErr,

    /// Setter was accepted only after calibrated clamping.
    CheckLimitErr,

    /// Frame checksum mismatch.
    CheckSumErr,

    /// Channel, mnemonic, or delimiter syntax failure.
    SyntaxErr,
}

impl PromptCode {
    /// Returns the Pascal label embedded in verbose serial prompts.
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
