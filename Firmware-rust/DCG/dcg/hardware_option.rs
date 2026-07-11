//! Identifies optional DCG hardware encoded in the EEPROM option byte.

/// Selects one installed-hardware flag from the Pascal `Options` byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HardwareOption {
    /// Selects the 16-bit LTC1655 DAC instead of the 12-bit LTC1257 path.
    Ltc1655Dac = 0,

    /// Selects the external 16-bit LTC1864 ADC instead of the AVR ADC.
    Ltc1864Adc = 1,

    /// Enables the DC power daughterboard relay, sensing, and temperature paths.
    DcPowerBoard = 2,
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
