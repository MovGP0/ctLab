#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Parser-side front-panel menu selection retained by DSP subchannels.
pub enum Modify {
    /// Lower main-page value.
    LowerMainMenu,

    /// Upper main-page value.
    UpperMainMenu,

    /// Regulation mode editor.
    ModeMenu,

    /// Active ripple time editor.
    TOn,

    /// Inactive ripple time editor.
    TOff,

    /// Ripple off-current editor.
    IOff,

    /// Temperature page.
    TempMenu,

    /// Internal-resistance page.
    RiMenu,

    /// Charge/energy page.
    CapMenu,

    /// Power/current page.
    PwrCurMenu,

    /// Unrecognized menu byte retained for parameter-error reporting.
    Unknown(u8),
}

impl From<u8> for Modify {
    /// Maps defined DSP menu bytes and preserves unknown values for parameter-error reporting.
    fn from(value: u8) -> Self {
        match value {
            0 => Self::LowerMainMenu,
            1 => Self::UpperMainMenu,
            2 => Self::ModeMenu,
            3 => Self::TOn,
            4 => Self::TOff,
            5 => Self::IOff,
            6 => Self::TempMenu,
            7 => Self::RiMenu,
            8 => Self::CapMenu,
            9 => Self::PwrCurMenu,
            other => Self::Unknown(other),
        }
    }
}
