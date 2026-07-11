//! Names the LM75 register selectors used during temperature-limit setup.

/// Selects one register in the LM75 pointer protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Lm75Register {
    /// Temperature sample register selected before normal polling resumes.
    Temperature = 0,

    /// Configuration register controlling output polarity and operating mode.
    Configuration = 1,

    /// Lower temperature threshold that releases the comparator output.
    Hysteresis = 2,

    /// Upper temperature threshold that asserts the comparator output.
    Overtemperature = 3,
}

impl Lm75Register {
    /// Returns the register selector placed on the I2C bus.
    pub const fn address(self) -> u8 {
        self as u8
    }
}
