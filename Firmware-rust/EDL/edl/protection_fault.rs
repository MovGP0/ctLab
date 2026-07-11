//! Defines EDL shutdown causes whose bit positions and labels are part of the status protocol.

/// One latched protection cause reported after an overload status response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionFault {
    /// Requested or measured dissipation exceeded the configured maximum power.
    OverPower,

    /// External supervision detected an open load-path fuse.
    FuseBlown,

    /// Input voltage exceeded the active low/high range ceiling.
    OverVoltage,

    /// Internal or external LM75 temperature exceeded its programmed threshold.
    OverTemperature,

    /// Input voltage fell below the configured cutoff and latched output shutdown.
    LowVoltage,
}

impl ProtectionFault {
    /// Returns the exact bracketed label appended to EDL overload responses.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OverPower => "[OVRPOWR]",
            Self::FuseBlown => "[FUSEBLW]",
            Self::OverVoltage => "[OVRVOLT]",
            Self::OverTemperature => "[OVRTEMP]",
            Self::LowVoltage => "[LOWVOLT]",
        }
    }
}
