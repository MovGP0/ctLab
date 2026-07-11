use super::*;

pub struct AvrdPortIo<M: Mcu> {
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
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn init_port(&mut self, port: RegisterPort, ddr: u8, value: u8) {
        unsafe {
            write_u8(self.ddr_ptr(port), ddr);
            write_u8(self.port_ptr(port), value);
        }
    }

    pub fn write_port(&mut self, port: RegisterPort, value: u8) {
        unsafe {
            write_u8(self.port_ptr(port), value);
        }
    }

    pub fn write_bit(&mut self, port: RegisterPort, bit: u8, high: bool) {
        unsafe {
            update_u8(self.port_ptr(port), |value| {
                set_or_clear_bit(value, bit, high)
            });
        }
    }

    pub fn read_bit(&self, port: RegisterPort, bit: u8) -> bool {
        unsafe { read_u8(self.pin_ptr(port)) & (1 << bit) != 0 }
    }

    pub fn spin_delay_cycles(&mut self, cycles: u16) {
        for _ in 0..cycles {
            compiler_fence(Ordering::SeqCst);
        }
    }

    pub fn spi_transfer(&mut self, tx: u8) -> u8 {
        unsafe {
            write_u8(M::SPDR, tx);
            while read_u8(M::SPSR) & M::SPIF_MASK == 0 {}
            read_u8(M::SPDR)
        }
    }

    pub fn wait_for_adc(&mut self) {
        unsafe { while read_u8(M::ADCSRA) & M::ADSC_MASK != 0 {} }
    }

    pub fn read_adc_blocking(&mut self, channel_1_based: u8, external_ref: bool) -> u16 {
        unsafe {
            let mux_value = channel_1_based.saturating_sub(1) & M::MUX_MASK;
            let refs_value = if external_ref { 0 } else { M::REFS_MASK };
            write_u8(M::ADMUX, refs_value | mux_value);
            update_u8(M::ADCSRA, |value| value | M::ADIF_MASK | M::ADSC_MASK);
        }
        self.wait_for_adc();
        unsafe { u16::from(read_u8(M::ADCL)) | (u16::from(read_u8(M::ADCH)) << 8) }
    }

    fn pin_ptr(&self, port: RegisterPort) -> *mut u8 {
        match port {
            RegisterPort::A => M::PINA,
            RegisterPort::B => M::PINB,
            RegisterPort::C => M::PINC,
            RegisterPort::D => M::PIND,
        }
    }

    fn ddr_ptr(&self, port: RegisterPort) -> *mut u8 {
        match port {
            RegisterPort::A => M::DDRA,
            RegisterPort::B => M::DDRB,
            RegisterPort::C => M::DDRC,
            RegisterPort::D => M::DDRD,
        }
    }

    fn port_ptr(&self, port: RegisterPort) -> *mut u8 {
        match port {
            RegisterPort::A => M::PORTA,
            RegisterPort::B => M::PORTB,
            RegisterPort::C => M::PORTC,
            RegisterPort::D => M::PORTD,
        }
    }
}
