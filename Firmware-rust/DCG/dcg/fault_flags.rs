//! Retains independent DCG protection causes for output shutdown and remote diagnosis.

use super::FaultKind;

/// Independent protection causes retained separately so diagnostics can report why output was disabled instead of exposing one undifferentiated overload bit.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FaultFlags {
    /// Latches `over_power` until status packing/protection consumes it, ensuring asynchronous causes are not lost between service cycles.
    pub over_power: bool,

    /// Latches `fuse_blown` until status packing/protection consumes it, ensuring asynchronous causes are not lost between service cycles.
    pub fuse_blown: bool,

    /// Latches `over_voltage` until status packing/protection consumes it, ensuring asynchronous causes are not lost between service cycles.
    pub over_voltage: bool,

    /// Latches `over_temp` until status packing/protection consumes it, ensuring asynchronous causes are not lost between service cycles.
    pub over_temp: bool,
}
impl FaultFlags {
    /// Reports whether any is active so safety and protocol code share one predicate.
    pub fn any(self) -> bool {
        self.bits() != 0
    }

    /// Packs the latched condition into its assigned protocol bit or error-code position for the status response.
    pub fn bits(self) -> u8 {
        (self.over_power as u8)
            | ((self.fuse_blown as u8) << 1)
            | ((self.over_voltage as u8) << 2)
            | ((self.over_temp as u8) << 3)
    }

    /// Reports whether the named protection cause is currently latched.
    pub const fn is_active(self, fault: FaultKind) -> bool {
        match fault {
            FaultKind::OverPower => self.over_power,
            FaultKind::FuseBlown => self.fuse_blown,
            FaultKind::OverVoltage => self.over_voltage,
            FaultKind::OverTemperature => self.over_temp,
        }
    }
}
