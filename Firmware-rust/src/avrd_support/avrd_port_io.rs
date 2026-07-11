use super::*;

/// Performs register-level GPIO, SPI, and ADC operations for an [`Mcu`].
///
/// The type owns no peripheral state. Its marker binds operations to a concrete
/// MCU at compile time so firmware hardware traits can share code without
/// virtual dispatch or stored register pointers.
pub struct AvrdPortIo<M: Mcu> {
    /// Binds the zero-sized adapter to one MCU register map without storing it.
    _marker: PhantomData<M>,
}
impl<M: Mcu> Default for AvrdPortIo<M> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
impl<M: Mcu> AvrdPortIo<M> {
    /// Creates a zero-sized I/O adapter for `M`.
    ///
    /// Construction performs no hardware access, allowing firmware state to be
    /// assembled before its explicit port-initialization sequence runs.
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Configures a GPIO bank and then establishes its initial output levels.
    ///
    /// Writing direction before the output latch mirrors the Pascal startup
    /// sequence used by each board and centralizes its raw register access.
    pub fn init_port(&mut self, port: RegisterPort, ddr: u8, value: u8) {
        // SAFETY: Mcu implementations guarantee valid DDR and PORT registers.
        unsafe {
            write_u8(self.ddr_ptr(port), ddr);
            write_u8(self.port_ptr(port), value);
        }
    }

    /// Replaces every output-latch bit in `port` with `value`.
    ///
    /// This is used when a firmware family intentionally presents a complete
    /// parallel control word and therefore must update all pins together.
    pub fn write_port(&mut self, port: RegisterPort, value: u8) {
        // SAFETY: Mcu implementations guarantee valid PORT registers.
        unsafe {
            write_u8(self.port_ptr(port), value);
        }
    }

    /// Drives one output-latch bit while preserving the other pins in its bank.
    ///
    /// The read-modify-write supports individually sequenced strobes and chip
    /// selects whose neighboring outputs must remain unchanged.
    ///
    /// # Panics
    ///
    /// Panics when overflow checks are enabled and `bit` is greater than seven.
    pub fn write_bit(&mut self, port: RegisterPort, bit: u8, high: bool) {
        // SAFETY: Mcu implementations guarantee a valid PORT register, and the
        // firmware serializes foreground writes to each bank.
        unsafe {
            update_u8(self.port_ptr(port), |value| {
                set_or_clear_bit(value, bit, high)
            });
        }
    }

    /// Samples one physical input pin from `port`.
    ///
    /// Reading the PIN register, rather than the output latch, is required for
    /// status and serial-data inputs driven by external hardware.
    ///
    /// # Panics
    ///
    /// Panics when overflow checks are enabled and `bit` is greater than seven.
    pub fn read_bit(&self, port: RegisterPort, bit: u8) -> bool {
        // SAFETY: Mcu implementations guarantee valid PIN registers.
        unsafe { read_u8(self.pin_ptr(port)) & (1 << bit) != 0 }
    }

    /// Burns approximately `cycles` loop iterations without movable side effects.
    ///
    /// The compiler fence prevents removal or coalescing of short protocol gaps;
    /// exact elapsed time still depends on optimization and the MCU clock.
    pub fn spin_delay_cycles(&mut self, cycles: u16) {
        for _ in 0..cycles {
            compiler_fence(Ordering::SeqCst);
        }
    }

    /// Transfers one byte through the AVR SPI peripheral and returns the reply.
    ///
    /// Writing `SPDR` starts the transfer, then polling `SPIF` serializes the
    /// next protocol step until all eight clock pulses have completed.
    pub fn spi_transfer(&mut self, tx: u8) -> u8 {
        // SAFETY: Mcu implementations guarantee valid SPI data and status registers.
        unsafe {
            write_u8(M::SPDR, tx);
            while read_u8(M::SPSR) & M::SPIF_MASK == 0 {}
            read_u8(M::SPDR)
        }
    }

    /// Waits until the ADC start bit clears at conversion completion.
    ///
    /// Firmware calls this blocking primitive where the following operation
    /// consumes the new sample and cannot safely proceed with stale ADC data.
    pub fn wait_for_adc(&mut self) {
        // SAFETY: Mcu implementations guarantee a valid ADC status register.
        unsafe { while read_u8(M::ADCSRA) & M::ADSC_MASK != 0 {} }
    }

    /// Selects an ADC input, performs one conversion, and returns its ten-bit result.
    ///
    /// Board protocols number channels from one, so the channel is saturated and
    /// converted to the AVR's zero-based MUX value. `external_ref` clears the
    /// reference bits for externally supplied AREF; otherwise the MCU-defined
    /// reference mask is selected. Reading `ADCL` before `ADCH` preserves the
    /// AVR result-register locking sequence.
    pub fn read_adc_blocking(&mut self, channel_1_based: u8, external_ref: bool) -> u16 {
        // SAFETY: Mcu implementations guarantee valid ADC configuration and
        // status registers; this adapter owns the conversion sequence.
        unsafe {
            let mux_value = channel_1_based.saturating_sub(1) & M::MUX_MASK;
            let refs_value = if external_ref { 0 } else { M::REFS_MASK };
            write_u8(M::ADMUX, refs_value | mux_value);
            update_u8(M::ADCSRA, |value| value | M::ADIF_MASK | M::ADSC_MASK);
        }
        self.wait_for_adc();
        // SAFETY: Mcu implementations guarantee valid ADC result registers;
        // waiting above completes the conversion before low-byte-first access.
        unsafe { u16::from(read_u8(M::ADCL)) | (u16::from(read_u8(M::ADCH)) << 8) }
    }

    /// Maps a logical bank to the register that samples its physical pin levels.
    ///
    /// Keeping this separate from [`Self::port_ptr`] prevents input reads from
    /// accidentally observing only the last value written to the output latch.
    fn pin_ptr(&self, port: RegisterPort) -> *mut u8 {
        match port {
            RegisterPort::A => M::PINA,
            RegisterPort::B => M::PINB,
            RegisterPort::C => M::PINC,
            RegisterPort::D => M::PIND,
        }
    }

    /// Maps a logical bank to the register controlling input/output direction.
    ///
    /// Firmware startup uses this only while applying each board's schematic pin
    /// direction mask, keeping that selection distinct from later output writes.
    fn ddr_ptr(&self, port: RegisterPort) -> *mut u8 {
        match port {
            RegisterPort::A => M::DDRA,
            RegisterPort::B => M::DDRB,
            RegisterPort::C => M::DDRC,
            RegisterPort::D => M::DDRD,
        }
    }

    /// Maps a logical bank to its output latch and input pull-up register.
    ///
    /// Both full-bank writes and individual pin updates share this mapping so a
    /// board cannot silently use different registers for the two operations.
    fn port_ptr(&self, port: RegisterPort) -> *mut u8 {
        match port {
            RegisterPort::A => M::PORTA,
            RegisterPort::B => M::PORTB,
            RegisterPort::C => M::PORTC,
            RegisterPort::D => M::PORTD,
        }
    }
}
