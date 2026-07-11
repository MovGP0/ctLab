//! Maps DCG converter, multiplexer, and interrupt operations to AVR registers and pins.

use super::*;

/// AVR register adapter for DCG timing-sensitive pins and ADC registers.
pub struct DcgAvrd<M: Mcu> {
    /// Owns the hardware adapter, ensuring all side effects are routed through one testable boundary.
    pub(super) io: AvrdPortIo<M>,

    /// Caches `_marker` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub(super) _marker: PhantomData<M>,
}
impl<M: Mcu> Default for DcgAvrd<M> {
    /// Creates the zero-sized AVR DCG adapter without touching registers; board initialization remains an explicit later step.
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            _marker: PhantomData,
        }
    }
}
impl<M: Mcu> DcgAvrd<M> {
    /// Initializes ports before dependent calculations or outputs are enabled.
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::B, 0b1011_1111, 0b1101_0011);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b0000_1111);
        self.io.init_port(RegisterPort::D, 0b0000_1100, 0b1111_1100);
    }
}
impl<M: Mcu> DcgHardware for DcgAvrd<M> {
    /// Drives the shared serial-data pin used by the board's DAC and ADC shift protocols.
    fn set_sdata_out(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 1, high);
    }

    /// Drives the shared serial clock; callers control each edge explicitly because attached converters sample on different phases.
    fn set_sclk(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 0, high);
    }

    /// Drives the DAC latch strobe only after a complete serial word is stable.
    fn set_str_dac(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 4, high);
    }

    /// Drives the ADC conversion/chip-select strobe around one uninterrupted LTC1864 transfer.
    fn set_str_ad16(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::B, 7, high);
    }

    /// Selects the current measurement path before the ADC settling interval begins.
    fn set_mpx_i(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::C, 5, high);
    }

    /// Selects the voltage measurement path before the ADC settling interval begins.
    fn set_mpx_u(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::C, 4, high);
    }

    /// Selects the LTC1864 input phase used by the next pipelined conversion.
    fn set_mpx_1864(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::C, 6, high);
    }

    /// Samples the converter's serial-data pin on the clock edge chosen by the bit-level receive routine.
    fn read_sdata_in1(&self) -> bool {
        self.io.read_bit(RegisterPort::B, 6)
    }

    /// Burns the requested CPU cycles between converter edges whose minimum spacing is shorter than a scheduler tick.
    fn spin_delay_cycles(&mut self, cycles: u16) {
        self.io.spin_delay_cycles(cycles);
    }

    /// Writes the AVR analog-multiplexer selection used by the subsequent ADC10 conversion.
    fn set_admux(&mut self, value: u8) {
        unsafe {
            crate::avrd_support::write_u8(M::ADMUX, value);
        }
    }

    /// Writes the AVR ADC control/status byte used to start conversion with the configured prescaler.
    fn write_adcsra(&mut self, value: u8) {
        unsafe {
            crate::avrd_support::write_u8(M::ADCSRA, value);
        }
    }

    /// Reads ADC status so polling waits for ADSC to clear instead of consuming an unfinished conversion.
    fn read_adcsra(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCSRA) }
    }

    /// Reads the ADC low byte first, which latches the paired high byte on AVR hardware.
    fn read_adcl(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCL) }
    }

    /// Reads the high byte after ADCL to complete one coherent AVR ADC sample.
    fn read_adch(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCH) }
    }

    /// Saves SREG and disables interrupts so a multi-edge converter transaction cannot be split by timer service.
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

    /// Restores the saved SREG value so the caller's previous interrupt-enable state survives the converter transaction.
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

    /// Holds the DCG DAC interface idle for the converter's required post-latch settling loop.
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
