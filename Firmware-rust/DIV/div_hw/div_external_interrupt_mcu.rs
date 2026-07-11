#[allow(unused_imports)]
use super::*;

pub trait DivExternalInterruptMcu: Mcu {
    const MCUCSR: *mut u8;
    const GICR: *mut u8;
    const INT2_MASK: u8;
    const ISC2_MASK: u8;
}

impl DivExternalInterruptMcu for Atmega32 {
    const MCUCSR: *mut u8 = avrd::atmega32::MCUCSR;
    const GICR: *mut u8 = avrd::atmega32::GICR;
    const INT2_MASK: u8 = 0b0010_0000;
    const ISC2_MASK: u8 = 0b0100_0000;
}
