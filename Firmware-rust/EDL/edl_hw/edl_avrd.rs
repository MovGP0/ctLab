use super::*;

/// `avrd` register backend retaining single-instruction timing on real AVR targets.
pub struct EdlAvrd<M: Mcu> {
    /// MCU-specific port accessor used for ordinary GPIO and host-side fallback paths.
    io: AvrdPortIo<M>,

    /// Saved SREG value restored after the ADC clock train completes.
    saved_status: u8,

    /// Associates register addresses with the selected ATmega type without runtime storage.
    _marker: PhantomData<M>,
}

impl<M: Mcu> Default for EdlAvrd<M> {
    /// Creates a backend before board-specific data-direction initialization.
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            saved_status: 0,
            _marker: PhantomData,
        }
    }
}

impl<M: Mcu> EdlAvrd<M> {
    /// Applies the Pascal DDR/output masks so latches start at hardware-safe levels.
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::B, 0b1011_1011, 0b1111_1101);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b1100_0011);
        self.io.init_port(RegisterPort::D, 0b0000_1100, 0b1111_1111);
    }
}

impl<M: Mcu> EdlHardware for EdlAvrd<M> {
    /// Uses single-cycle AVR I/O instructions on hardware and an equivalent port write in tests.
    fn set_control_bit(&mut self, bit: ControlBit, high: bool) {
        #[cfg(target_arch = "avr")]
        {
            set_edl_control_bit_single_instruction(bit, high);
        }

        #[cfg(not(target_arch = "avr"))]
        {
            self.set_control_bit_fallback(bit, high);
        }
    }

    /// Uses the AVR skip-instruction timing path on hardware and ordinary pin reads on hosts.
    fn read_control_bit(&self, bit: ControlBit) -> bool {
        #[cfg(target_arch = "avr")]
        {
            read_edl_control_bit_with_skip(bit)
        }

        #[cfg(not(target_arch = "avr"))]
        {
            self.read_control_bit_fallback(bit)
        }
    }

    /// Drives Port C bit 7 so the trigger output follows the active ripple phase.
    fn set_trigger_out(&mut self, high: bool) {
        self.io.write_bit(RegisterPort::C, 7, high);
    }

    /// Samples Port B bit 2, the board's external trigger input.
    fn read_trigger_in(&self) -> bool {
        self.io.read_bit(RegisterPort::B, 2)
    }

    /// Switches Port B bit 1 atomically so the next pipelined ADC conversion sees the intended signal.
    fn set_ad16_mpx(&mut self, high: bool) {
        #[cfg(target_arch = "avr")]
        {
            set_edl_ad16_mpx_single_instruction(high);
        }

        #[cfg(not(target_arch = "avr"))]
        {
            self.io.write_bit(RegisterPort::B, 1, high);
        }
    }

    /// Writes the MCU-specific ADMUX register selected by the generic ATmega map.
    fn set_admux(&mut self, value: u8) {
        unsafe {
            crate::avrd_support::write_u8(M::ADMUX, value);
        }
    }

    /// Starts or configures conversion through the MCU-specific ADCSRA address.
    fn write_adcsra(&mut self, value: u8) {
        unsafe {
            crate::avrd_support::write_u8(M::ADCSRA, value);
        }
    }

    /// Reads live ADCSRA state so polling observes the hardware-cleared start bit.
    fn read_adcsra(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCSRA) }
    }

    /// Reads ADCL first, latching a coherent high byte on AVR hardware.
    fn read_adcl(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCL) }
    }

    /// Completes the ADC data-register read after ADCL has latched the sample.
    fn read_adch(&self) -> u8 {
        unsafe { crate::avrd_support::read_u8(M::ADCH) }
    }

    /// Saves SREG and clears only its interrupt-enable bit around the converter clock train.
    fn begin_interrupt_exclusion(&mut self) -> u8 {
        #[cfg(target_arch = "avr")]
        {
            self.saved_status = unsafe { core::ptr::read_volatile(AVR_SREG_ADDRESS) };
            unsafe {
                core::ptr::write_volatile(
                    AVR_SREG_ADDRESS,
                    self.saved_status & !AVR_SREG_INTERRUPT_ENABLE_MASK,
                );
            }
            self.saved_status
        }

        #[cfg(not(target_arch = "avr"))]
        {
            self.saved_status
        }
    }

    /// Restores the caller's exact SREG state instead of always enabling interrupts.
    fn end_interrupt_exclusion(&mut self, saved_status: u8) {
        #[cfg(target_arch = "avr")]
        unsafe {
            core::ptr::write_volatile(AVR_SREG_ADDRESS, saved_status);
        }

        #[cfg(not(target_arch = "avr"))]
        {
            self.saved_status = saved_status;
        }
    }

    /// Emits one deterministic idle cycle through the shared port helper.
    fn nop(&mut self) {
        self.io.spin_delay_cycles(1);
    }

    /// Uses the original 15-iteration AVR loop so ADC mux settling remains cycle stable.
    fn settle_adc10_mux(&mut self) {
        #[cfg(target_arch = "avr")]
        unsafe {
            core::arch::asm!(
                "ldi {counter}, 15",
                "2:",
                "dec {counter}",
                "brne 2b",
                counter = lateout(reg_upper) _,
                options(nomem, nostack)
            );
        }

        #[cfg(not(target_arch = "avr"))]
        {
            for _ in 0..ADC10_SETTLE_CYCLES {
                self.io.spin_delay_cycles(1);
            }
        }
    }
}

impl<M: Mcu> EdlAvrd<M> {
    /// Portable port-write equivalent used by host tests where AVR `SBI` is unavailable.
    fn set_control_bit_fallback(&mut self, bit: ControlBit, high: bool) {
        let bit_number = match bit {
            ControlBit::Sclk => 7,
            ControlBit::SDataOut => 5,
            ControlBit::StrDac => 3,
            ControlBit::StrAd16 => 4,
            ControlBit::SDataIn1 => 6,
        };
        self.io.write_bit(RegisterPort::B, bit_number, high);
    }

    /// Portable pin-read equivalent used by host tests where AVR skip instructions are unavailable.
    fn read_control_bit_fallback(&self, bit: ControlBit) -> bool {
        let bit_number = match bit {
            ControlBit::Sclk => 7,
            ControlBit::SDataOut => 5,
            ControlBit::StrDac => 3,
            ControlBit::StrAd16 => 4,
            ControlBit::SDataIn1 => 6,
        };
        self.io.read_bit(RegisterPort::B, bit_number)
    }
}

#[cfg(target_arch = "avr")]
/// Dispatches logical lines to constant-address `SBI`/`CBI` helpers for fixed cycle timing.
fn set_edl_control_bit_single_instruction(bit: ControlBit, high: bool) {
    match (bit, high) {
        (ControlBit::Sclk, true) => set_edl_port_b_bit_7(),
        (ControlBit::Sclk, false) => clear_edl_port_b_bit_7(),
        (ControlBit::SDataOut, true) => set_edl_port_b_bit_5(),
        (ControlBit::SDataOut, false) => clear_edl_port_b_bit_5(),
        (ControlBit::StrDac, true) => set_edl_port_b_bit_3(),
        (ControlBit::StrDac, false) => clear_edl_port_b_bit_3(),
        (ControlBit::StrAd16, true) => set_edl_port_b_bit_4(),
        (ControlBit::StrAd16, false) => clear_edl_port_b_bit_4(),
        (ControlBit::SDataIn1, true) => set_edl_port_b_bit_6(),
        (ControlBit::SDataIn1, false) => clear_edl_port_b_bit_6(),
    }
}

#[cfg(target_arch = "avr")]
/// Reads a logical line through the Pascal-equivalent skip-instruction sequence.
fn read_edl_control_bit_with_skip(bit: ControlBit) -> bool {
    let value = match bit {
        ControlBit::Sclk => read_edl_pin_b_bit_7_with_skip(),
        ControlBit::SDataOut => read_edl_pin_b_bit_5_with_skip(),
        ControlBit::StrDac => read_edl_pin_b_bit_3_with_skip(),
        ControlBit::StrAd16 => read_edl_pin_b_bit_4_with_skip(),
        ControlBit::SDataIn1 => read_edl_pin_b_bit_6_with_skip(),
    };
    value != 0
}

#[cfg(target_arch = "avr")]
/// Switches the ADC16 mux with one atomic I/O instruction.
fn set_edl_ad16_mpx_single_instruction(high: bool) {
    if high {
        set_edl_port_b_bit_1();
    } else {
        clear_edl_port_b_bit_1();
    }
}

#[cfg(target_arch = "avr")]
/// Expands a constant Port-B bit set into one `SBI` instruction.
macro_rules! edl_sbi {
    ($bit:literal) => {
        unsafe {
            core::arch::asm!(
                "sbi {port}, {bit}",
                port = const EDL_CONTROL_BIT_PORT_IO_ADDRESS,
                bit = const $bit,
                options(nomem, nostack, preserves_flags)
            );
        }
    };
}

#[cfg(target_arch = "avr")]
/// Expands a constant Port-B bit clear into one `CBI` instruction.
macro_rules! edl_cbi {
    ($bit:literal) => {
        unsafe {
            core::arch::asm!(
                "cbi {port}, {bit}",
                port = const EDL_CONTROL_BIT_PORT_IO_ADDRESS,
                bit = const $bit,
                options(nomem, nostack, preserves_flags)
            );
        }
    };
}

#[cfg(target_arch = "avr")]
/// Reads a constant Port-B input with `SBIC`, returning a normalized zero/one byte.
macro_rules! edl_sbic_read {
    ($bit:literal) => {{
        let value: u8;
        unsafe {
            core::arch::asm!(
                "ldi {value}, 0",
                "sbic {pin}, {bit}",
                "ldi {value}, 1",
                pin = const EDL_CONTROL_BIT_PIN_IO_ADDRESS,
                bit = const $bit,
                value = lateout(reg_upper) value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }};
}

#[cfg(target_arch = "avr")]
/// Sets the ADC16 multiplexer line on Port B bit 1.
fn set_edl_port_b_bit_1() {
    edl_sbi!(1);
}

#[cfg(target_arch = "avr")]
/// Clears the ADC16 multiplexer line on Port B bit 1.
fn clear_edl_port_b_bit_1() {
    edl_cbi!(1);
}

#[cfg(target_arch = "avr")]
/// Sets the DAC strobe on Port B bit 3.
fn set_edl_port_b_bit_3() {
    edl_sbi!(3);
}

#[cfg(target_arch = "avr")]
/// Clears the DAC strobe on Port B bit 3.
fn clear_edl_port_b_bit_3() {
    edl_cbi!(3);
}

#[cfg(target_arch = "avr")]
/// Sets the ADC16 conversion strobe on Port B bit 4.
fn set_edl_port_b_bit_4() {
    edl_sbi!(4);
}

#[cfg(target_arch = "avr")]
/// Clears the ADC16 conversion strobe on Port B bit 4.
fn clear_edl_port_b_bit_4() {
    edl_cbi!(4);
}

#[cfg(target_arch = "avr")]
/// Sets serial data output on Port B bit 5.
fn set_edl_port_b_bit_5() {
    edl_sbi!(5);
}

#[cfg(target_arch = "avr")]
/// Clears serial data output on Port B bit 5.
fn clear_edl_port_b_bit_5() {
    edl_cbi!(5);
}

#[cfg(target_arch = "avr")]
/// Sets the shared serial input bit only for parity with the Pascal bit dispatcher.
fn set_edl_port_b_bit_6() {
    edl_sbi!(6);
}

#[cfg(target_arch = "avr")]
/// Clears the shared serial input bit only for parity with the Pascal bit dispatcher.
fn clear_edl_port_b_bit_6() {
    edl_cbi!(6);
}

#[cfg(target_arch = "avr")]
/// Sets the converter serial clock on Port B bit 7.
fn set_edl_port_b_bit_7() {
    edl_sbi!(7);
}

#[cfg(target_arch = "avr")]
/// Clears the converter serial clock on Port B bit 7.
fn clear_edl_port_b_bit_7() {
    edl_cbi!(7);
}

#[cfg(target_arch = "avr")]
/// Samples the DAC strobe bit through the fixed-cycle skip helper.
fn read_edl_pin_b_bit_3_with_skip() -> u8 {
    edl_sbic_read!(3)
}

#[cfg(target_arch = "avr")]
/// Samples the ADC16 strobe bit through the fixed-cycle skip helper.
fn read_edl_pin_b_bit_4_with_skip() -> u8 {
    edl_sbic_read!(4)
}

#[cfg(target_arch = "avr")]
/// Samples serial data output through the fixed-cycle skip helper.
fn read_edl_pin_b_bit_5_with_skip() -> u8 {
    edl_sbic_read!(5)
}

#[cfg(target_arch = "avr")]
/// Samples converter serial input through the fixed-cycle skip helper.
fn read_edl_pin_b_bit_6_with_skip() -> u8 {
    edl_sbic_read!(6)
}

#[cfg(target_arch = "avr")]
/// Samples the serial clock bit through the fixed-cycle skip helper.
fn read_edl_pin_b_bit_7_with_skip() -> u8 {
    edl_sbic_read!(7)
}
