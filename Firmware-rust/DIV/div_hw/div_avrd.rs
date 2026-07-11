#[allow(unused_imports)]
use super::*;

pub struct DivAvrd<M: Mcu> {
    pub(super) io: AvrdPortIo<M>,
    pub(super) _marker: PhantomData<M>,
}

impl<M: Mcu> Default for DivAvrd<M> {
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            _marker: PhantomData,
        }
    }
}

impl<M: DivExternalInterruptMcu> DivAvrd<M> {
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::A, 0b1110_0000, 0b0000_0011);
        self.io.init_port(RegisterPort::B, 0b1001_0000, 0b1001_0001);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b0000_0011);
        self.io.init_port(RegisterPort::D, 0b0001_1100, 0b1111_1100);
        self.configure_external_trigger_falling_edge();
    }

    pub(super) fn configure_external_trigger_falling_edge(&mut self) {
        unsafe {
            crate::avrd_support::update_u8(M::GICR, |value| value | M::INT2_MASK);
            crate::avrd_support::update_u8(M::MCUCSR, |value| value & !M::ISC2_MASK);
        }
    }
}

impl<M: Mcu> DivHardware for DivAvrd<M> {
    fn set_str_ad24(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 4, high);
    }

    fn set_sclk(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 7, high);
    }

    fn read_sdata_in1(&self) -> bool {
        self.io.read_bit(RegisterPort::B, 6)
    }

    fn set_spi_control(&mut self, value: u8) {
        unsafe {
            crate::avrd_support::write_u8(M::SPCR, value);
        }
    }

    fn spi_transfer(&mut self, tx: u8) -> u8 {
        self.io.spi_transfer(tx)
    }

    fn spin_delay_cycles(&mut self, cycles: u16) {
        self.io.spin_delay_cycles(cycles);
    }
}
