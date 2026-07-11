//! Collects SQG operating and EEPROM-write latches used by serial status reporting.

/// SQG status latches packed into their original bit positions for serial compatibility.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct StatusFlags {
    /// Latches `busy` until status packing/protection consumes it, ensuring asynchronous causes are not lost between service cycles.
    pub(super) busy: bool,

    /// Latches `user_srq` until status packing/protection consumes it, ensuring asynchronous causes are not lost between service cycles.
    pub(super) user_srq: bool,

    /// Latches overload so output protection and diagnostic reporting observe the same cause.
    pub(super) overload: bool,

    /// Authorizes exactly the protected EEPROM-setting path after a successful `WEN` command.
    pub(super) ee_unlocked: bool,
}
impl StatusFlags {
    /// Packs the latched condition into its assigned protocol bit or error-code position for the status response.
    pub(super) fn to_status_byte(self) -> u8 {
        // Original wire format: bit 7 = Busy, 6 = User SRQ, 5 = Overload,
        // 4 = EEPROM unlocked. The low nibble is the current error code.
        ((self.busy as u8) << 7)
            | ((self.user_srq as u8) << 6)
            | ((self.overload as u8) << 5)
            | ((self.ee_unlocked as u8) << 4)
    }
}
