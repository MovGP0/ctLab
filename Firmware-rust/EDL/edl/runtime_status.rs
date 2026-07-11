#[derive(Debug, Default, Clone, Copy)]

/// Non-fault flags occupying the high nibble of the EDL status byte.
pub struct RuntimeStatus {
    /// Allows calibration/EEPROM setters that are rejected during normal operation.
    pub ee_unlocked: bool,

    /// Mirrors an active protection condition for clients expecting one overload bit.
    pub overload_flag: bool,

    /// Records a user-generated service request until it is acknowledged.
    pub user_srq_flag: bool,

    /// Prevents commands during timing-sensitive acquisition or output changes.
    pub busy_flag: bool,
}

impl RuntimeStatus {
    /// Positions volatile flags in bits 4-7 for combination with fault bits 0-4.
    pub fn flag_bits(self) -> u8 {
        ((self.ee_unlocked as u8) << 4)
            | ((self.overload_flag as u8) << 5)
            | ((self.user_srq_flag as u8) << 6)
            | ((self.busy_flag as u8) << 7)
    }
}
