use super::*;

/// Compile-time register map for the ATmega32 used by most firmware families.
///
/// The uninhabited marker carries no run-time state; selecting it specializes
/// [`AvrdPortIo`] calls directly to the ATmega32 addresses and masks.
pub enum Atmega32 {}
impl Mcu for Atmega32 {
    const PINA: *mut u8 = avrd::atmega32::PINA;
    const PINB: *mut u8 = avrd::atmega32::PINB;
    const PINC: *mut u8 = avrd::atmega32::PINC;
    const PIND: *mut u8 = avrd::atmega32::PIND;

    const DDRA: *mut u8 = avrd::atmega32::DDRA;
    const DDRB: *mut u8 = avrd::atmega32::DDRB;
    const DDRC: *mut u8 = avrd::atmega32::DDRC;
    const DDRD: *mut u8 = avrd::atmega32::DDRD;

    const PORTA: *mut u8 = avrd::atmega32::PORTA;
    const PORTB: *mut u8 = avrd::atmega32::PORTB;
    const PORTC: *mut u8 = avrd::atmega32::PORTC;
    const PORTD: *mut u8 = avrd::atmega32::PORTD;

    const SPCR: *mut u8 = avrd::atmega32::SPCR;
    const SPSR: *mut u8 = avrd::atmega32::SPSR;
    const SPDR: *mut u8 = avrd::atmega32::SPDR;
    const SPIF_MASK: u8 = 0x80;

    const ADMUX: *mut u8 = avrd::atmega32::ADMUX;
    const ADCSRA: *mut u8 = avrd::atmega32::ADCSRA;
    const ADCL: *mut u8 = avrd::atmega32::ADCL;
    const ADCH: *mut u8 = avrd::atmega32::ADCH;
    const ADSC_MASK: u8 = 0x40;
    const ADIF_MASK: u8 = 0x10;
    const MUX_MASK: u8 = 0x1f;
    const REFS_MASK: u8 = 0xc0;
}
