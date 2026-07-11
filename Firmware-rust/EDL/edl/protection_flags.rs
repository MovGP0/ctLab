#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]

/// Independent shutdown causes retained so status replies can report every active fault.
pub struct ProtectionFlags {
    /// Requested or measured load exceeds the calibrated power ceiling.
    pub over_power: bool,

    /// External fuse supervision indicates the current path is no longer intact.
    pub fuse_blown: bool,

    /// Measured input voltage exceeds the range-specific safe limit.
    pub over_voltage: bool,

    /// LM75 temperature supervision exceeded the programmed threshold.
    pub over_temp: bool,

    /// Input dropped below the configured cutoff and the latch has not been cleared.
    pub low_volt: bool,
}

impl ProtectionFlags {
    /// Collapses all causes for the fast output-disable decision without losing detail.
    pub fn any(self) -> bool {
        self.bits() != 0
    }

    /// Packs faults into the low five protocol bits in the original Pascal order.
    pub fn bits(self) -> u8 {
        (self.over_power as u8)
            | ((self.fuse_blown as u8) << 1)
            | ((self.over_voltage as u8) << 2)
            | ((self.over_temp as u8) << 3)
            | ((self.low_volt as u8) << 4)
    }
}
