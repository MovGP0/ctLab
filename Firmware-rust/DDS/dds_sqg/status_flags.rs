#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct StatusFlags {
    pub(super) busy: bool,
    pub(super) user_srq: bool,
    pub(super) overload: bool,
    pub(super) ee_unlocked: bool,
}
impl StatusFlags {
    pub(super) fn to_status_byte(self) -> u8 {
        // Original wire format: bit 7 = Busy, 6 = User SRQ, 5 = Overload,
        // 4 = EEPROM unlocked. The low nibble is the current error code.
        ((self.busy as u8) << 7)
            | ((self.user_srq as u8) << 6)
            | ((self.overload as u8) << 5)
            | ((self.ee_unlocked as u8) << 4)
    }
}
