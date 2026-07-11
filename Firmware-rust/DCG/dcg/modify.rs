//! Defines the finite ring of front-panel values that encoder and navigation actions may edit.

/// Front-panel editing target; this state determines which setpoint an encoder movement is allowed to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modify {
    /// Routes encoder changes to the current setpoint.
    Ampere,

    /// Routes encoder changes to the voltage setpoint.
    Volt,

    /// Routes encoder changes to ripple depth.
    Ripple,

    /// Edits the energized ripple interval.
    TOn,

    /// Edits the de-energized ripple interval.
    TOff,

    /// Edits the peer channel used for tracking commands.
    TrackCh,

    /// Displays accumulated capacity.
    CapMenu,

    /// Displays measured power.
    PwrMenu,
}
