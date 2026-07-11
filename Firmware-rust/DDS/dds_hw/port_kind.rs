//! Names DDS port groups so bit-level routines remain independent of AVR register addresses.

/// Logical DDS port groups, decoupling shift routines from MCU-specific register addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortKind {
    /// Addresses the port carrying AD9833 data, clock, and frame-sync signals.
    DdsOut,

    /// Addresses the board control port used by DAC and relay latch signals.
    ControlBit,

    /// Addresses the extension port carrying optional daughterboard control lines.
    Extension,

    /// Addresses the front-panel LED output port independently of converter traffic.
    LedOut,
}
