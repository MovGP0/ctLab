use super::*;

pub struct DdsAvrd<M: Mcu> {
    pub(super) io: AvrdPortIo<M>,
    pub(super) saved_status: u8,
    pub(super) _marker: PhantomData<M>,
}
impl<M: Mcu> Default for DdsAvrd<M> {
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            saved_status: 0,
            _marker: PhantomData,
        }
    }
}
impl<M: Mcu> DdsAvrd<M> {
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::B, 0b0001_1111, 0b0001_0111);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b1111_1111);
        self.io.init_port(RegisterPort::D, 0b0000_1100, 0b1111_1100);
    }

    fn map_port(port: PortKind) -> RegisterPort {
        match port {
            PortKind::DdsOut | PortKind::ControlBit => RegisterPort::B,
            PortKind::Extension => RegisterPort::C,
            PortKind::LedOut => RegisterPort::D,
        }
    }
}
impl<M: Mcu> DdsHardwareIo for DdsAvrd<M> {
    fn set_bit(&mut self, port: PortKind, bit: u8) {
        self.io.write_bit(Self::map_port(port), bit, true);
    }

    fn clear_bit(&mut self, port: PortKind, bit: u8) {
        self.io.write_bit(Self::map_port(port), bit, false);
    }

    fn nop(&mut self) {
        self.io.spin_delay_cycles(1);
    }

    fn delay_units(&mut self, units: u8) {
        self.io
            .spin_delay_cycles(u16::from(units) * SER_AUX_DELAY_CYCLES_PER_UNIT);
    }

    fn delay_ms(&mut self, milliseconds: u16) {
        for _ in 0..milliseconds {
            self.io.spin_delay_cycles(16_000);
        }
    }

    fn begin_critical_section(&mut self) {
        #[cfg(target_arch = "avr")]
        {
            self.saved_status = unsafe { core::ptr::read_volatile(AVR_SREG_ADDRESS) };
            unsafe {
                core::ptr::write_volatile(
                    AVR_SREG_ADDRESS,
                    self.saved_status & !AVR_SREG_INTERRUPT_ENABLE_MASK,
                );
            }
        }
    }

    fn end_critical_section(&mut self) {
        #[cfg(target_arch = "avr")]
        unsafe {
            core::ptr::write_volatile(AVR_SREG_ADDRESS, self.saved_status);
        }

        #[cfg(not(target_arch = "avr"))]
        {
            let _ = self.saved_status;
        }
    }
}
