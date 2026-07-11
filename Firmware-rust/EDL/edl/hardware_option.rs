//! Identifies optional EDL hardware encoded in the EEPROM option byte.

/// Selects one installed-hardware flag from the Pascal `Options` byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HardwareOption {
    /// Enables the LM75 at address `0x49` on the internal power stage.
    InternalLm75 = 2,

    /// Enables the LM75 at address `0x48` on the external power stage.
    ExternalLm75 = 3,

    /// Identifies the Pascal auxiliary LM75 flag reserved in bit four.
    AuxiliaryLm75 = 4,
}

impl HardwareOption {
    /// Returns the bit mask stored in the persisted hardware-option byte.
    pub const fn mask(self) -> u8 {
        1 << self as u8
    }

    /// Reports whether this hardware path is enabled in a persisted option byte.
    pub const fn is_set_in(self, options: u8) -> bool {
        options & self.mask() != 0
    }
}
