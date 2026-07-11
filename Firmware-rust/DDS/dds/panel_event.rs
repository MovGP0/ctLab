#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelEvent {
    EncoderDelta(i16),
    ToggleFine,
    NextModify,
    PrevModify,
    Buttons {
        enter: bool,
        left: bool,
        right: bool,
    },
    IncrTimerElapsed,
    DisplayTimerElapsed,
    ReleaseBusy,
}
