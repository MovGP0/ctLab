//! Selects between the installed 12-bit and 16-bit DCG DAC transaction formats.

/// Supported DAC wire protocols; the selected EEPROM option changes both word width and clock framing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DacKind {
    /// Selects the original 12-bit LTC1257 serial DAC and its latch timing.
    Ltc1257,

    /// Selects the optional 16-bit LTC1655 transaction format and full-scale code.
    Ltc1655,
}
