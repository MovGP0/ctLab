//! Collects status latches packed into the protocol-visible operating byte.

/// Latched status flags that are packed into the Pascal-compatible serial status byte.
#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeStatus {
    /// Latches `error_low_nibble` until status packing/protection consumes it, ensuring asynchronous causes are not lost between service cycles.
    pub error_low_nibble: u8,

    /// Authorizes exactly the protected EEPROM-setting path after a successful `WEN` command.
    pub ee_unlocked: bool,

    /// Latches overload flag so output protection and diagnostic reporting observe the same cause.
    pub overload_flag: bool,

    /// Latches `user_srq_flag` until status packing/protection consumes it, ensuring asynchronous causes are not lost between service cycles.
    pub user_srq_flag: bool,

    /// Marks local panel ownership so a remote mutating command returns `BusyErr` instead of racing the encoder.
    pub busy_flag: bool,
}
impl RuntimeStatus {
    /// Encodes the typed state using the byte values retained by EEPROM and the serial protocol.
    pub fn as_byte(self) -> u8 {
        (self.error_low_nibble & 0x0f)
            | ((self.ee_unlocked as u8) << 4)
            | ((self.overload_flag as u8) << 5)
            | ((self.user_srq_flag as u8) << 6)
            | ((self.busy_flag as u8) << 7)
    }
}
