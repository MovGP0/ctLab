use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PanelEvent {
    None,
    Encoder(i32),
    Button(PanelButton),
    IncrTimerExpired,
    DisplayTimerExpired,
}
