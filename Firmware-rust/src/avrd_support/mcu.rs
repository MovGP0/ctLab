/// Supplies the register addresses and masks needed by generic AVR port I/O.
///
/// Implementations are zero-sized MCU descriptions. Keeping these values as
/// associated constants allows LLVM to inline direct peripheral accesses,
/// which avoids a run-time dispatch table and saves flash on AVR builds.
/// Implementors must map every pointer to an aligned, readable and writable
/// eight-bit peripheral register for the active device because safe
/// [`AvrdPortIo`](super::AvrdPortIo) methods rely on that invariant.
pub trait Mcu {
    /// Input-pin register for GPIO bank A.
    const PINA: *mut u8;

    /// Input-pin register for GPIO bank B.
    const PINB: *mut u8;

    /// Input-pin register for GPIO bank C.
    const PINC: *mut u8;

    /// Input-pin register for GPIO bank D.
    const PIND: *mut u8;

    /// Data-direction register for GPIO bank A.
    const DDRA: *mut u8;

    /// Data-direction register for GPIO bank B.
    const DDRB: *mut u8;

    /// Data-direction register for GPIO bank C.
    const DDRC: *mut u8;

    /// Data-direction register for GPIO bank D.
    const DDRD: *mut u8;

    /// Output latch and pull-up register for GPIO bank A.
    const PORTA: *mut u8;

    /// Output latch and pull-up register for GPIO bank B.
    const PORTB: *mut u8;

    /// Output latch and pull-up register for GPIO bank C.
    const PORTC: *mut u8;

    /// Output latch and pull-up register for GPIO bank D.
    const PORTD: *mut u8;

    /// SPI control register address.
    const SPCR: *mut u8;

    /// SPI status register address.
    const SPSR: *mut u8;

    /// SPI data register address used for both transmit and receive.
    const SPDR: *mut u8;

    /// SPI status mask that signals completion of a byte transfer.
    const SPIF_MASK: u8;

    /// ADC multiplexer and voltage-reference selection register.
    const ADMUX: *mut u8;

    /// ADC control and status register.
    const ADCSRA: *mut u8;

    /// Low byte of the ADC conversion result; it must be read first.
    const ADCL: *mut u8;

    /// High byte of the ADC conversion result.
    const ADCH: *mut u8;

    /// ADC control mask that starts a conversion and remains set while it runs.
    const ADSC_MASK: u8;

    /// Write-one-to-clear ADC completion flag used before starting a conversion.
    const ADIF_MASK: u8;

    /// Mask retaining only ADC channel-selection bits in [`Self::ADMUX`].
    const MUX_MASK: u8;

    /// Mask selecting the MCU's internal/default ADC reference bits.
    const REFS_MASK: u8;
}
