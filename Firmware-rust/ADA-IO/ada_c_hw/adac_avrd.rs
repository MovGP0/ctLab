#[allow(unused_imports)]
use super::*;

pub struct AdacAvrd<M: Mcu> {
    pub(super) io: AvrdPortIo<M>,
    pub(super) _marker: PhantomData<M>,
}

impl<M: Mcu> Default for AdacAvrd<M> {
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            _marker: PhantomData,
        }
    }
}

impl<M: Mcu> AdacAvrd<M> {
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::B, 0b0101_1011, 0b1011_1111);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b0000_0011);
        self.io.init_port(RegisterPort::D, 0b0000_1100, 0b1111_1100);
    }

    pub(super) fn map_signal(signal: Signal) -> (RegisterPort, u8) {
        match signal {
            Signal::SDataOut => (RegisterPort::B, 1),
            Signal::SClk => (RegisterPort::B, 0),
            Signal::StrDac => (RegisterPort::B, 3),
            Signal::StrAd16 => (RegisterPort::B, 4),
            Signal::StrSr => (RegisterPort::B, 6),
            Signal::StrDaMux => (RegisterPort::C, 5),
            Signal::SDataIn1 => (RegisterPort::B, 5),
        }
    }
}

impl<M: Mcu> AdacHardware for AdacAvrd<M> {
    fn set_signal(&mut self, signal: Signal, high: bool) {
        let (port, bit) = Self::map_signal(signal);
        self.io.write_bit(port, bit, high);
    }

    fn read_signal(&self, signal: Signal) -> bool {
        let (port, bit) = Self::map_signal(signal);
        self.io.read_bit(port, bit)
    }

    fn set_port_c(&mut self, value: Byte) {
        self.io.write_port(RegisterPort::C, value);
    }

    fn set_admux(&mut self, value: Byte) {
        unsafe {
            crate::avrd_support::write_u8(M::ADMUX, value);
        }
    }

    fn write_adcsra(&mut self, value: Byte) {
        unsafe {
            crate::avrd_support::write_u8(M::ADCSRA, value);
        }
    }

    fn read_adcsra(&self) -> Byte {
        unsafe { crate::avrd_support::read_u8(M::ADCSRA) }
    }

    fn read_adcl(&self) -> Byte {
        unsafe { crate::avrd_support::read_u8(M::ADCL) }
    }

    fn read_adch(&self) -> Byte {
        unsafe { crate::avrd_support::read_u8(M::ADCH) }
    }

    fn begin_interrupt_exclusion(&mut self) -> Byte {
        #[cfg(target_arch = "avr")]
        {
            let saved_status = unsafe { core::ptr::read_volatile(AVR_SREG_ADDRESS) };
            unsafe {
                core::ptr::write_volatile(
                    AVR_SREG_ADDRESS,
                    saved_status & !AVR_SREG_INTERRUPT_ENABLE_MASK,
                );
            }
            saved_status
        }

        #[cfg(not(target_arch = "avr"))]
        {
            0
        }
    }

    fn end_interrupt_exclusion(&mut self, saved_status: Byte) {
        #[cfg(target_arch = "avr")]
        unsafe {
            core::ptr::write_volatile(AVR_SREG_ADDRESS, saved_status);
        }

        #[cfg(not(target_arch = "avr"))]
        {
            let _ = saved_status;
        }
    }

    fn nop(&mut self) {
        self.io.spin_delay_cycles(1);
    }

    fn wait_cycles(&mut self, cycles: u16) {
        self.io.spin_delay_cycles(cycles);
    }

    fn wait_for_adc10_complete(&mut self) {
        self.io.wait_for_adc();
    }
}
