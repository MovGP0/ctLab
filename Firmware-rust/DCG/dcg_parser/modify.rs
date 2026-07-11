//! Defines the finite ring of front-panel values that encoder and navigation actions may edit.

/// Front-panel editing target; this state determines which setpoint an encoder movement is allowed to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Modify {
    /// Routes encoder changes to the current setpoint.
    Ampere = 0,

    /// Routes encoder changes to the voltage setpoint.
    Volt = 1,

    /// Routes encoder changes to ripple depth.
    Ripple = 2,

    /// Edits the energized ripple interval.
    TOn = 3,

    /// Edits the de-energized ripple interval.
    TOff = 4,

    /// Edits the peer channel used for tracking commands.
    TrackCh = 5,

    /// Displays accumulated capacity.
    CapMenu = 6,

    /// Displays measured power.
    PwrMenu = 7,
}
impl Modify {
    /// Decodes the persisted/wire byte into its typed state, mapping unsupported values to the safe disabled or error sentinel.
    pub fn from_byte(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Ampere),
            1 => Some(Self::Volt),
            2 => Some(Self::Ripple),
            3 => Some(Self::TOn),
            4 => Some(Self::TOff),
            5 => Some(Self::TrackCh),
            6 => Some(Self::CapMenu),
            7 => Some(Self::PwrMenu),
            _ => None,
        }
    }
}
