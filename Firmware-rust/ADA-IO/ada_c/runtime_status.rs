//! Defines ADA the runtime status responsibilities separated from the original monolithic source.

#[allow(unused_imports)]
use super::*;

/// Collects runtime status that must survive across polling-loop or interrupt updates.
#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeStatus {
    /// Contains protocol error code bits 0..3 before runtime flags are packed above them.
    pub error_low_nibble: u8,

    /// Sets packed status bit 4 while `WEN` permits EEPROM and calibration writes.
    pub ee_unlocked: bool,

    /// Latches overload flag from the same converter status bits as the associated sample.
    pub overload_flag: bool,

    /// Adds the user-service-request bit to the next status response after a panel action.
    pub user_srq_flag: bool,

    /// Rejects state-changing commands while initialization, calibration, or a panel operation owns the device.
    pub busy_flag: bool,
}

impl RuntimeStatus {
    /// Encodes as byte in the compact representation consumed by registers or the serial protocol.
    pub fn as_byte(self) -> u8 {
        (self.error_low_nibble & 0x0f)
            | ((self.ee_unlocked as u8) << 4)
            | ((self.overload_flag as u8) << 5)
            | ((self.user_srq_flag as u8) << 6)
            | ((self.busy_flag as u8) << 7)
    }
}
