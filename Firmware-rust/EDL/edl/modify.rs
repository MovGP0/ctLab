#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Front-panel menu value selected for encoder editing.
pub enum Modify {
    /// Lower line of the main measurement page.
    LowerMainMenu,

    /// Upper line of the main measurement page.
    UpperMainMenu,

    /// Regulation mode selector.
    ModeMenu,

    /// Ripple active-time editor.
    TOn,

    /// Ripple inactive-time editor.
    TOff,

    /// Ripple off-current editor.
    IOff,

    /// Temperature/status page.
    TempMenu,

    /// Internal-resistance page.
    RiMenu,

    /// Charge/energy capacity page.
    CapMenu,

    /// Power/current setpoint page.
    PwrCurMenu,
}
