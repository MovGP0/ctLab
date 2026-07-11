use super::*;

pub struct DcgAvrd<M: Mcu> {
    pub(super) io: AvrdPortIo<M>,
    pub(super) _marker: PhantomData<M>,
}
impl<M: Mcu> Default for DcgAvrd<M> {
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            _marker: PhantomData,
        }
    }
}
impl<M: Mcu> DcgAvrd<M> {
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::B, 0b1011_1111, 0b1101_0011);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b0000_1111);
        self.io.init_port(RegisterPort::D, 0b0000_1100, 0b1111_1100);
    }
}
impl<M: Mcu> DcgHardware for DcgAvrd<M> {
    fn set_sdata_out(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 1, high);
    }

    fn set_sclk(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 0, high);
    }

    fn set_str_dac(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 4, high);
    }

    fn set_str_ad16(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 7, high);
    }

    fn set_mpx_i(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::C, 5, high);
    }

    fn set_mpx_u(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::C, 4, high);
    }

    fn set_mpx_1864(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::C, 6, high);
    }

    fn read_sdata_in1(&self) -> bool {
        self.io.read_bit(RegisterPort::B, 6)
    }

    fn spin_delay_cycles(&mut self, cycles: u16) {
        self.io.spin_delay_cycles(cycles);
    }

    fn set_admux(&mut self, value: u8) {
        unsafe {
            crate::avrd_support::write_u8(M::ADMUX, value);
        }
    }

    fn write_adcsra(&mut self, value: u8) {
        unsafe {
            crate::avrd_support::write_u8(M::ADCSRA, value);
        }
    }

    fn read_adcsra(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCSRA) }
    }

    fn read_adcl(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCL) }
    }

    fn read_adch(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCH) }
    }

    fn begin_interrupt_exclusion(&mut self) -> u8 {
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

    fn end_interrupt_exclusion(&mut self, saved_status: u8) {
        #[cfg(target_arch = "avr")]
        unsafe {
            core::ptr::write_volatile(AVR_SREG_ADDRESS, saved_status);
        }

        #[cfg(not(target_arch = "avr"))]
        {
            let _ = saved_status;
        }
    }

    fn wait_post_dac_settle(&mut self) {
        #[cfg(target_arch = "avr")]
        unsafe {
            core::arch::asm!(
                "ldi r24, 40",
                "2:",
                "dec r24",
                "brne 2b",
                lateout("r24") _,
                options(nomem, nostack)
            );
        }

        #[cfg(not(target_arch = "avr"))]
        {
            for _ in 0..DAC_POST_WRITE_SETTLE_LOOP_ITERATIONS {
                self.nop();
            }
        }
    }
}
