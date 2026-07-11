//! Names each positional DCG protection bit and its wire-visible diagnostic label.

/// One independently latched DCG protection cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultKind {
    /// Output power exceeded the configured protection limit.
    OverPower,

    /// The protected supply path indicates a blown or missing fuse.
    FuseBlown,

    /// Measured output voltage exceeded the configured safe limit.
    OverVoltage,

    /// The LM75 temperature exceeded the configured shutdown threshold.
    OverTemperature,
}
impl FaultKind {
    /// Faults in the original low-nibble bit order used by status frames.
    #[rustfmt::skip]
    pub const ALL: [Self; 4] = [
        Self::OverPower,
        Self::FuseBlown,
        Self::OverVoltage,
        Self::OverTemperature,
    ];

    /// Returns the exact diagnostic label appended to a DCG status frame.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OverPower => "[OVRPOWR]",
            Self::FuseBlown => "[FUSEBLW]",
            Self::OverVoltage => "[OVRVOLT]",
            Self::OverTemperature => "[OVRTEMP]",
        }
    }

    /// Returns the fault's assigned bit in the low status nibble.
    pub const fn bit(self) -> u8 {
        match self {
            Self::OverPower => 1 << 0,
            Self::FuseBlown => 1 << 1,
            Self::OverVoltage => 1 << 2,
            Self::OverTemperature => 1 << 3,
        }
    }
}
