use super::*;

pub enum Atmega644 {}
impl Mcu for Atmega644 {
    const PINA: *mut u8 = avrd::atmega644::PINA;
    const PINB: *mut u8 = avrd::atmega644::PINB;
    const PINC: *mut u8 = avrd::atmega644::PINC;
    const PIND: *mut u8 = avrd::atmega644::PIND;

    const DDRA: *mut u8 = avrd::atmega644::DDRA;
    const DDRB: *mut u8 = avrd::atmega644::DDRB;
    const DDRC: *mut u8 = avrd::atmega644::DDRC;
    const DDRD: *mut u8 = avrd::atmega644::DDRD;

    const PORTA: *mut u8 = avrd::atmega644::PORTA;
    const PORTB: *mut u8 = avrd::atmega644::PORTB;
    const PORTC: *mut u8 = avrd::atmega644::PORTC;
    const PORTD: *mut u8 = avrd::atmega644::PORTD;

    const SPCR: *mut u8 = avrd::atmega644::SPCR;
    const SPSR: *mut u8 = avrd::atmega644::SPSR;
    const SPDR: *mut u8 = avrd::atmega644::SPDR;
    const SPIF_MASK: u8 = 0x80;

    const ADMUX: *mut u8 = avrd::atmega644::ADMUX;
    const ADCSRA: *mut u8 = avrd::atmega644::ADCSRA;
    const ADCL: *mut u8 = avrd::atmega644::ADCL;
    const ADCH: *mut u8 = avrd::atmega644::ADCH;
    const ADSC_MASK: u8 = 0x40;
    const ADIF_MASK: u8 = 0x10;
    const MUX_MASK: u8 = 0x1f;
    const REFS_MASK: u8 = 0xc0;
}
