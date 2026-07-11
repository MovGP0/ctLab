//! Defines DIV the AVR register-backed implementation of the board-I/O contract.

#[allow(unused_imports)]
use super::*;

/// Implements div AVR register by mapping logical signals onto AVR device registers.
pub struct DivAvrd<M: Mcu> {
    /// Owns the AVR register adapter that maps logical signals to MCU ports.
    pub(super) io: AvrdPortIo<M>,

    /// Binds marker to its MCU type without occupying runtime storage.
    pub(super) _marker: PhantomData<M>,
}

impl<M: Mcu> Default for DivAvrd<M> {
    /// Builds the reset-state value used when EEPROM data or host setup supplies no override.
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            _marker: PhantomData,
        }
    }
}

impl<M: DivExternalInterruptMcu> DivAvrd<M> {
    /// Initializes ports in the same order as the original startup routine.
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::A, 0b1110_0000, 0b0000_0011);
        self.io.init_port(RegisterPort::B, 0b1001_0000, 0b1001_0001);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b0000_0011);
        self.io.init_port(RegisterPort::D, 0b0001_1100, 0b1111_1100);
        self.configure_external_trigger_falling_edge();
    }

    /// Configures external trigger falling edge before code that relies on that hardware capability runs.
    pub(super) fn configure_external_trigger_falling_edge(&mut self) {
        unsafe {
            crate::avrd_support::update_u8(M::GICR, |value| value | M::INT2_MASK);
            crate::avrd_support::update_u8(M::MCUCSR, |value| value & !M::ISC2_MASK);
        }
    }
}

impl<M: Mcu> DivHardware for DivAvrd<M> {
    /// Drives LTC2400 strobe/chip-select on Port B bit 4.
    fn set_str_ad24(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 4, high);
    }

    /// Drives the LTC2400 clock on Port B bit 7.
    fn set_sclk(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 7, high);
    }

    /// Samples sdata in1 directly from its mapped input pin during the bit-level peripheral transaction.
    fn read_sdata_in1(&self) -> bool {
        self.io.read_bit(RegisterPort::B, 6)
    }

    /// Writes the ATmega SPCR register used during the three-byte LTC2400 read.
    fn set_spi_control(&mut self, value: u8) {
        unsafe {
            crate::avrd_support::write_u8(M::SPCR, value);
        }
    }

    /// Transfers SPI transfer using the byte order expected by the attached peripheral.
    fn spi_transfer(&mut self, tx: u8) -> u8 {
        self.io.spi_transfer(tx)
    }

    /// Burns the requested processor cycles between signal edges where the peripheral data sheet requires setup or hold time.
    fn spin_delay_cycles(&mut self, cycles: u16) {
        self.io.spin_delay_cycles(cycles);
    }
}
