//! Defines ADA the AVR register-backed implementation of the board-I/O contract.

#[allow(unused_imports)]
use super::*;

/// Implements adac AVR register by mapping logical signals onto AVR device registers.
pub struct AdacAvrd<M: Mcu> {
    /// Owns the AVR register adapter that maps logical signals to MCU ports.
    pub(super) io: AvrdPortIo<M>,

    /// Binds marker to its MCU type without occupying runtime storage.
    pub(super) _marker: PhantomData<M>,
}

impl<M: Mcu> Default for AdacAvrd<M> {
    /// Creates the zero-sized ATmega register adapter before `init_ports` applies board directions and idle levels.
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            _marker: PhantomData,
        }
    }
}

impl<M: Mcu> AdacAvrd<M> {
    /// Initializes ports in the same order as the original startup routine.
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::B, 0b0101_1011, 0b1011_1111);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b0000_0011);
        self.io.init_port(RegisterPort::D, 0b0000_1100, 0b1111_1100);
    }

    /// Maps a logical ADA signal to its AVR port and bit so higher-level shift routines remain pin-name independent.
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
    /// Drives the AVR port bit returned by `map_signal`.
    fn set_signal(&mut self, signal: Signal, high: bool) {
        let (port, bit) = Self::map_signal(signal);
        self.io.write_bit(port, bit, high);
    }

    /// Samples signal directly from its mapped input pin during the bit-level peripheral transaction.
    fn read_signal(&self, signal: Signal) -> bool {
        let (port, bit) = Self::map_signal(signal);
        self.io.read_bit(port, bit)
    }

    /// Writes all eight Port C bits, including mux selection bits 2..4.
    fn set_port_c(&mut self, value: Byte) {
        self.io.write_port(RegisterPort::C, value);
    }

    /// Writes the ATmega ADMUX channel/reference selection register.
    fn set_admux(&mut self, value: Byte) {
        unsafe {
            crate::avrd_support::write_u8(M::ADMUX, value);
        }
    }

    /// Writes adcsra to the serial, display, or peripheral destination selected by the implementation.
    fn write_adcsra(&mut self, value: Byte) {
        unsafe {
            crate::avrd_support::write_u8(M::ADCSRA, value);
        }
    }

    /// Reads the AVR adcsra register used to detect completion and assemble the 10-bit conversion.
    fn read_adcsra(&self) -> Byte {
        unsafe { crate::avrd_support::read_u8(M::ADCSRA) }
    }

    /// Reads the AVR adcl register used to detect completion and assemble the 10-bit conversion.
    fn read_adcl(&self) -> Byte {
        unsafe { crate::avrd_support::read_u8(M::ADCL) }
    }

    /// Reads the AVR adch register used to detect completion and assemble the 10-bit conversion.
    fn read_adch(&self) -> Byte {
        unsafe { crate::avrd_support::read_u8(M::ADCH) }
    }

    /// Marks the begin interrupt exclusion boundary so interrupt state is saved and restored around the complete transaction.
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

    /// Marks the end interrupt exclusion boundary so interrupt state is saved and restored around the complete transaction.
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

    /// Provides the nop timing gap required between peripheral signal edges.
    fn nop(&mut self) {
        self.io.spin_delay_cycles(1);
    }

    /// Waits for cycles so callers cannot consume a stale hardware result.
    fn wait_cycles(&mut self, cycles: u16) {
        self.io.spin_delay_cycles(cycles);
    }

    /// Waits for for adc10 complete so callers cannot consume a stale hardware result.
    fn wait_for_adc10_complete(&mut self) {
        self.io.wait_for_adc();
    }
}
