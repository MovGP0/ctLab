pub trait Mcu {
    const PINA: *mut u8;
    const PINB: *mut u8;
    const PINC: *mut u8;
    const PIND: *mut u8;

    const DDRA: *mut u8;
    const DDRB: *mut u8;
    const DDRC: *mut u8;
    const DDRD: *mut u8;

    const PORTA: *mut u8;
    const PORTB: *mut u8;
    const PORTC: *mut u8;
    const PORTD: *mut u8;

    const SPCR: *mut u8;
    const SPSR: *mut u8;
    const SPDR: *mut u8;
    const SPIF_MASK: u8;

    const ADMUX: *mut u8;
    const ADCSRA: *mut u8;
    const ADCL: *mut u8;
    const ADCH: *mut u8;
    const ADSC_MASK: u8;
    const ADIF_MASK: u8;
    const MUX_MASK: u8;
    const REFS_MASK: u8;
}
