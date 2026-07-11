//! Defines DIV the div external interrupt mcu responsibilities separated from the original monolithic source.

#[allow(unused_imports)]
use super::*;

/// Defines the div external interrupt mcu boundary so translated timing and protocol logic can run against AVR registers or deterministic host doubles.
pub trait DivExternalInterruptMcu: Mcu {
    /// Exposes the ATmega32 `MCUCSR` register used to configure and enable external interrupt INT2.
    const MCUCSR: *mut u8;

    /// Exposes the ATmega32 `GICR` register used to configure and enable external interrupt INT2.
    const GICR: *mut u8;

    /// Selects GICR bit 5 (`INT2`), which enables the board's external trigger interrupt.
    const INT2_MASK: u8;

    /// Selects MCUCSR bit 6 (`ISC2`): clear for falling-edge INT2, set for rising-edge INT2.
    const ISC2_MASK: u8;
}

impl DivExternalInterruptMcu for Atmega32 {
    /// Exposes the ATmega32 `MCUCSR` register used to configure and enable external interrupt INT2.
    const MCUCSR: *mut u8 = avrd::atmega32::MCUCSR;

    /// Exposes the ATmega32 `GICR` register used to configure and enable external interrupt INT2.
    const GICR: *mut u8 = avrd::atmega32::GICR;

    /// Uses ATmega32 GICR bit 5 to enable external interrupt INT2.
    const INT2_MASK: u8 = 0b0010_0000;

    /// Uses ATmega32 MCUCSR bit 6 to select the INT2 edge polarity.
    const ISC2_MASK: u8 = 0b0100_0000;
}
