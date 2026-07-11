#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modify {
    Ampere,
    Volt,
    Ripple,
    TOn,
    TOff,
    TrackCh,
    CapMenu,
    PwrMenu,
}
