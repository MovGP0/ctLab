#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeStatus {
    pub error_low_nibble: u8,
    pub ee_unlocked: bool,
    pub overload_flag: bool,
    pub user_srq_flag: bool,
    pub busy_flag: bool,
}
impl RuntimeStatus {
    pub fn as_byte(self) -> u8 {
        // STR responses in the Pascal firmware packed the status byte as
        // Busy/UserSRQ/Overload-or-CurrentLimit/WriteEnable plus the low
        // nibble carrying the current fault or parser error code.
        (self.error_low_nibble & 0x0f)
            | ((self.ee_unlocked as u8) << 4)
            | ((self.overload_flag as u8) << 5)
            | ((self.user_srq_flag as u8) << 6)
            | ((self.busy_flag as u8) << 7)
    }
}
