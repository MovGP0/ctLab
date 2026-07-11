//! Best-effort Rust port of `ADA-C-HW.pas`.
//!
//! The original Pascal source is a hardware-near helper unit for the ADA-IO
//! firmware. This Rust version keeps the procedure structure, signal names, and
//! data layout readable while abstracting direct register access behind a small
//! trait.

use core::marker::PhantomData;

use crate::avrd_support::{Atmega32, AvrdPortIo, Mcu, RegisterPort};

/// Fixes `Byte` to the integer width used by the AVR-facing Pascal declarations.
pub type Byte = u8;

/// Fixes `Word` to the integer width used by the AVR-facing Pascal declarations.
pub type Word = u16;

/// Fixes `Integer` to the integer width used by the AVR-facing Pascal declarations.
pub type Integer = i16;

/// Fixes `LongInt` to the integer width used by the AVR-facing Pascal declarations.
pub type LongInt = i32;

/// Bounds mux channel count to the storage or channel capacity available on the controller.
pub const MUX_CHANNEL_COUNT: usize = 8;

/// Averages four external AD16 conversions per ADA systick when integration is enabled.
pub const ADC16_SAMPLES_PER_TICK: usize = 4;

/// Keeps fixed Port C control bits high while the low mux bits select an ADA channel.
pub const PORTC_MUX_BASE: Byte = 0b1100_0011;

/// Matches the Pascal 15-iteration delay before discarding the first AD16 sample after mux switching.
pub const ADC16_DISCARD_CONVERSION_DELAY_CYCLES: u16 = avr_dec_brne_delay_cycles(15);

/// Matches the Pascal four-iteration hold time after clocking a DAC word before latching it.
pub const DAC_SETTLE_DELAY_CYCLES: u16 = avr_dec_brne_delay_cycles(4);

/// Selects ADMUX bits 0..2, which encode the AVR ADC channel after the protocol's one-based channel is decremented.
pub const ADC10_CHANNEL_MASK: Byte = 0x07;

/// Sets ADMUX REFS1:REFS0 to `11`, selecting the AVR's internal 2.56 V reference.
pub const ADC10_INTERNAL_REFERENCE_MASK: Byte = 0xC0;

/// Matches the Pascal ten-iteration delay between ADMUX selection and conversion start.
pub const ADC10_SETTLE_DELAY_CYCLES: u16 = avr_dec_brne_delay_cycles(10);

/// Starts the AVR ADC with interrupt flag clear and a divide-by-128 conversion clock.
pub const ADCSRA_START_DIV128: Byte = 0xC7;

/// Selects ADCSRA bit 6 (`ADSC`), which remains high while the AVR ADC conversion is running.
pub const ADCSRA_BUSY_BIT: Byte = 1 << 6;

/// Points to the AVR status register whose global-interrupt bit is saved and restored around ADC register updates.
#[cfg(target_arch = "avr")]
const AVR_SREG_ADDRESS: *mut Byte = 0x5f as *mut Byte;

/// Selects the AVR SREG global-interrupt-enable bit that guards the ADC critical section.
#[cfg(target_arch = "avr")]
const AVR_SREG_INTERRUPT_ENABLE_MASK: Byte = 0x80;

/// Calculates the exact cycles consumed by the Pascal DEC/BRNE delay loop, including its final fall-through iteration.
/// Documents the compile-time value used by this small register/timing helper.
const fn avr_dec_brne_delay_cycles(iterations: u16) -> u16 {
    // ldi + repeated dec/brne costs exactly 3 * n cycles for n > 0.
    iterations * 3
}

#[path = "ada_c_hw/signal.rs"]
mod signal;
pub use signal::Signal;
#[path = "ada_c_hw/adac_hardware.rs"]
mod adac_hardware;
pub use adac_hardware::AdacHardware;
#[path = "ada_c_hw/adac_avrd.rs"]
mod adac_avrd;
pub use adac_avrd::AdacAvrd;
#[path = "ada_c_hw/adac_state.rs"]
mod adac_state;
pub use adac_state::AdacState;

/// Fixes `AdacAtmega32` to the integer width used by the AVR-facing Pascal declarations.
pub type AdacAtmega32 = AdacAvrd<Atmega32>;

/// Drives one named ADA control line low through its AVR or test-double mapping.
fn set_low(hw: &mut impl AdacHardware, signal: Signal) {
    hw.set_signal(signal, false);
}

/// Drives one named ADA control line high through its AVR or test-double mapping.
fn set_high(hw: &mut impl AdacHardware, signal: Signal) {
    hw.set_signal(signal, true);
}

/// Provides the nop timing gap required between peripheral signal edges.
fn nop(hw: &mut impl AdacHardware) {
    hw.nop();
}

/// Tests the outgoing most-significant bit before each shift so serial words are clocked in Pascal's MSB-first order.
fn msb_is_set(value: Byte) -> bool {
    value & 0x80 != 0
}

/// Clocks eight LTC1864 result bits from `SDataIn1`, most-significant bit first.
fn shift_in_byte_1864(hw: &mut impl AdacHardware) -> Byte {
    let mut acca: Byte = 0;

    for _ in 0..8 {
        set_high(hw, Signal::SClk);
        let carry = hw.read_signal(Signal::SDataIn1);
        set_low(hw, Signal::SClk);
        acca = (acca << 1) | Byte::from(carry);
    }

    acca
}

/// Encodes shift out sr byte in the compact representation consumed by registers or the serial protocol.
fn shift_out_sr_byte(hw: &mut impl AdacHardware, mut acca: Byte) {
    for _ in 0..8 {
        if msb_is_set(acca) {
            set_high(hw, Signal::SDataOut);
        }

        set_high(hw, Signal::SClk);
        acca <<= 1;
        set_low(hw, Signal::SDataOut);
        set_low(hw, Signal::SClk);
    }
}

// Sendet DACtemp an LTC1257.

/// Sends the 12-bit `dac_temp` code MSB-first to an LTC1257 and combines its final clock with the active-low load pulse.
pub fn shift_out1257(hw: &mut impl AdacHardware, state: &AdacState) {
    set_low(hw, Signal::SDataOut);
    set_low(hw, Signal::SClk);
    set_high(hw, Signal::StrDac);

    // The LTC1257 expects the high nibble of the 12-bit value first, left-aligned
    // onto the serial data line exactly like the Pascal bit-banging routine.
    let mut acca: Byte = ((state.dac_temp >> 8) as Byte) << 4; // MSB linksbuendig

    for _ in 0..4 {
        if msb_is_set(acca) {
            set_high(hw, Signal::SDataOut);
        }

        set_high(hw, Signal::SClk);
        acca <<= 1;
        nop(hw);
        set_low(hw, Signal::SDataOut);
        set_low(hw, Signal::SClk);
    }

    // The low byte follows afterwards; the final data bit is clocked together
    // with the DAC load strobe instead of as a plain shift cycle.
    let mut acca: Byte = state.dac_temp as Byte; // LSB Level zuletzt

    for _ in 0..7 {
        if msb_is_set(acca) {
            set_high(hw, Signal::SDataOut);
        }

        set_high(hw, Signal::SClk);
        acca <<= 1;
        nop(hw);
        set_low(hw, Signal::SDataOut);
        set_low(hw, Signal::SClk);
    }

    // LSB mit Load-Impuls.
    acca <<= 1;
    if msb_is_set(acca) {
        set_high(hw, Signal::SDataOut);
    }

    set_high(hw, Signal::SClk);
    set_low(hw, Signal::StrDac);
    nop(hw);
    set_low(hw, Signal::SDataOut);
    set_low(hw, Signal::SClk);
    set_high(hw, Signal::StrDac);
}

// Sendet DACtemp an LTC1655, etwas andere Sequenz als 1257.

/// Sends the full 16-bit `dac_temp` code MSB-first to an LTC1655, then raises `StrDac` to latch it.
pub fn shift_out1655(hw: &mut impl AdacHardware, state: &AdacState) {
    set_low(hw, Signal::SClk);
    set_low(hw, Signal::SDataOut);
    set_low(hw, Signal::StrDac);

    // Unlike the LTC1257 path, the LTC1655 shifts a full high byte before the
    // low byte and only latches once all 16 bits have been transferred.
    let mut acca: Byte = (state.dac_temp >> 8) as Byte; // MSB linksbuendig

    for _ in 0..8 {
        if msb_is_set(acca) {
            set_high(hw, Signal::SDataOut);
        }

        set_high(hw, Signal::SClk);
        acca <<= 1;
        nop(hw);
        set_low(hw, Signal::SDataOut);
        set_low(hw, Signal::SClk);
    }

    let mut acca: Byte = state.dac_temp as Byte; // LSB Level zuletzt

    for _ in 0..8 {
        if msb_is_set(acca) {
            set_high(hw, Signal::SDataOut);
        }

        set_high(hw, Signal::SClk);
        acca <<= 1;
        set_low(hw, Signal::SDataOut);
        set_low(hw, Signal::SClk);
    }

    set_high(hw, Signal::StrDac);
}

// Sendet DACtemp an DAC714.

/// Sends the 16-bit `dac_temp` code to a DAC714 on rising clock edges and finishes with its low-high latch sequence.
pub fn shift_out714(hw: &mut impl AdacHardware, state: &AdacState) {
    set_low(hw, Signal::SDataOut);
    set_high(hw, Signal::SClk);
    set_high(hw, Signal::StrDac);

    // The DAC714 uses the opposite clock phase: drive data while SCLK is low,
    // then let the rising edge shift the current bit into the converter.
    let mut acca: Byte = (state.dac_temp >> 8) as Byte; // MSB linksbuendig

    for _ in 0..8 {
        set_low(hw, Signal::SClk);
        if msb_is_set(acca) {
            set_high(hw, Signal::SDataOut);
        }

        acca <<= 1;
        nop(hw);
        set_high(hw, Signal::SClk);
        set_low(hw, Signal::SDataOut);
    }

    let mut acca: Byte = state.dac_temp as Byte; // LSB Level zuletzt

    for _ in 0..8 {
        set_low(hw, Signal::SClk);
        if msb_is_set(acca) {
            set_high(hw, Signal::SDataOut);
        }

        acca <<= 1;
        nop(hw);
        set_high(hw, Signal::SClk);
        set_low(hw, Signal::SDataOut);
    }

    set_low(hw, Signal::SClk);
    nop(hw);
    set_low(hw, Signal::StrDac);
    nop(hw);
    set_high(hw, Signal::SClk);
    nop(hw);
    set_high(hw, Signal::StrDac);
}

// Holt ADraw aus LTC1864, Interrupt waehrend dieser Zeit gesperrt.

/// Reads one 16-bit LTC1864 result MSB-first while interrupts are excluded, storing it in `ad_raw`.
pub fn shift_in1864(hw: &mut impl AdacHardware, state: &mut AdacState) {
    let saved_status = hw.begin_interrupt_exclusion();

    // Assert chip-select and keep the conversion transaction contiguous.
    set_low(hw, Signal::StrAd16);
    set_low(hw, Signal::SClk);
    nop(hw);
    nop(hw);
    nop(hw);

    let hi = shift_in_byte_1864(hw);
    let lo = shift_in_byte_1864(hw);

    state.ad_raw = Word::from_be_bytes([hi, lo]);

    set_high(hw, Signal::SClk);
    nop(hw);
    set_high(hw, Signal::StrAd16);
    hw.end_interrupt_exclusion(saved_status);
}

// Sende PortArray-Bytes an 4094-SR.

/// Shifts SR3 through SR0 MSB-first into four cascaded 4094 registers, then pulses their common latch.
pub fn shift_out_sr(hw: &mut impl AdacHardware, state: &AdacState) {
    set_low(hw, Signal::SClk);
    set_low(hw, Signal::SDataOut);

    // The four cascaded 4094 registers are filled from SR3 down to SR0 so the
    // logical port image emerges at the right physical outputs after strobing.
    shift_out_sr_byte(hw, state.port_sr3);
    shift_out_sr_byte(hw, state.port_sr2);
    shift_out_sr_byte(hw, state.port_sr1); // LSB Level zuletzt
    shift_out_sr_byte(hw, state.port_sr0); // LSB Level zuletzt

    set_high(hw, Signal::StrSr);
    nop(hw);
    nop(hw);
    set_low(hw, Signal::StrSr);
    set_high(hw, Signal::SClk);
}

/// Converts one-based AVR ADC channel 1..8 after selecting the external or internal 2.56 V reference and waiting for ADSC to clear.
pub fn get_adc10(hw: &mut impl AdacHardware, my_channel: Byte, ext_ref: bool) -> Word {
    // Zu-Fuss-Implementation der getadc()-Funktion.
    let mux = my_channel.wrapping_sub(1) & ADC10_CHANNEL_MASK;
    let reference = if ext_ref {
        ADC10_INTERNAL_REFERENCE_MASK
    } else {
        0
    };

    hw.set_admux(reference | mux);
    hw.wait_cycles(ADC10_SETTLE_DELAY_CYCLES);
    hw.write_adcsra(ADCSRA_START_DIV128);

    while (hw.read_adcsra() & ADCSRA_BUSY_BIT) != 0 {}

    Word::from(hw.read_adcl()) | (Word::from(hw.read_adch()) << 8)
}

// Interrupt-Routine, alle 1 ms, dauert etwa 41 us bei DA16.

/// Handles sys tick as one bounded polling-loop or interrupt service step.
pub fn on_sys_tick(hw: &mut impl AdacHardware, state: &mut AdacState) {
    // A/D-Wandlung letzter Kanal, 1 ms Settling Time!
    set_high(hw, Signal::SClk);

    if state.adc16_present {
        set_low(hw, Signal::StrAd16);
        state.ad16_long = 0;
        set_high(hw, Signal::StrAd16);

        // The first post-switch conversion is intentionally discarded so the
        // external ADC sees the full settling interval before we accumulate data.
        hw.wait_cycles(ADC16_DISCARD_CONVERSION_DELAY_CYCLES);

        for _ in 0..ADC16_SAMPLES_PER_TICK {
            shift_in1864(hw, state);
            state.ad16_long += LongInt::from(state.ad_raw) - 0x8000;
        }
    }

    let previous_mux_ch = state.mux_ch;

    // Finish sampling the previous mux input, then blank the analog path before
    // advancing the channel selection lines to the next source.
    set_low(hw, Signal::StrDaMux);
    state.mux_ch = (state.mux_ch + 1) % MUX_CHANNEL_COUNT;

    // Port C carries the mux address in bits 4..2 while preserving the fixed
    // control bits from the original firmware mask.
    hw.set_port_c(((state.mux_ch as Byte) << 2) | PORTC_MUX_BASE);

    state.dac_temp = state.dac_raw_array[state.mux_ch];
    if state.dac16_present {
        // Level-Bytes an LTC1655.
        shift_out1655(hw, state);
    }
    if state.dac714_present {
        // Level-Bytes an DAC714.
        shift_out714(hw, state);
    }
    if state.dac12_present {
        // Level-Bytes an LTC1257.
        shift_out1257(hw, state);
    }

    // Give the new DAC code time to settle before storing the measurement that
    // belongs to the previously active channel.
    hw.wait_cycles(DAC_SETTLE_DELAY_CYCLES);

    // Only average and store after the DAC settle delay; the Pascal code makes
    // the same point because this sample still belongs to `previous_mux_ch`.
    state.ad16_long >>= 2;
    if state.integrate_ad16 {
        // Integrating mode forms a simple 1:1 running average with the previous
        // stored sample, which suppresses noise without extra buffer state.
        state.ad16_long += LongInt::from(state.adc_raw_array[previous_mux_ch]);
        state.adc_raw_array[previous_mux_ch] = (state.ad16_long >> 1) as Integer;
    // integrieren
    } else {
        // Direct mode keeps the freshly averaged four-sample ADC result.
        state.adc_raw_array[previous_mux_ch] = state.ad16_long as Integer; // direkt
    }

    set_high(hw, Signal::StrDaMux);

    // Auf AD-Wandlung AD10 warten, falls Systick "ueberfahren" wurde.
    hw.wait_for_adc10_complete();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    /// Records observable hardware operations so tests can assert exact edge and register order.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Event {
        /// Records a logical ADA signal transition.
        Signal(Signal, bool),

        /// Records a complete ADA Port C write.
        PortC(Byte),

        /// Records a scripted digital-input sample.
        Read(Signal),

        /// Records selection of an internal ADC channel and reference.
        Admux(Byte),

        /// Records starting or configuring an internal ADC conversion.
        AdcsraWrite(Byte),

        /// Records polling the internal ADC completion flag.
        AdcsraRead(Byte),

        /// Records reading the ADC low byte before the high byte.
        AdclRead(Byte),

        /// Records reading the ADC high byte that latches the result.
        AdchRead(Byte),

        /// Records saving interrupt state before a multi-edge transfer.
        BeginInterruptExclusion,

        /// Records restoring interrupt state after a transfer.
        EndInterruptExclusion(Byte),

        /// Records one deliberate hold-time cycle when used as a hardware event.
        Nop,

        /// Records a requested block of peripheral setup or hold cycles.
        WaitCycles(u16),

        /// Records waiting until the internal ADC clears its busy bit.
        WaitForAdc10Complete,
    }

    /// Captures ADA converter signal transitions and scripted ADC inputs for cycle-accurate hardware routine tests.
    #[derive(Debug, Default)]
    struct TestHardware {
        events: RefCell<Vec<Event>>,

        /// Records input bits in occurrence order so tests can assert the complete external interaction.
        input_bits: Vec<bool>,

        /// Tracks input index within the fixed-capacity sequence used by this routine.
        input_index: Cell<usize>,

        next_status: Byte,

        adcsra_reads: Vec<Byte>,

        adcsra_read_index: Cell<usize>,

        /// Contains adcl in converter counts until the owning conversion or output routine consumes it.
        adcl: Byte,

        /// Contains adch in converter counts until the owning conversion or output routine consumes it.
        adch: Byte,
    }

    impl TestHardware {
        /// Constructs a hardware test double preloaded with input word for a deterministic conversion trace.
        fn with_input_word(word: Word) -> Self {
            let input_bits = (0..16).rev().map(|bit| word & (1 << bit) != 0).collect();

            Self {
                events: RefCell::new(Vec::new()),
                input_bits,
                input_index: Cell::new(0),
                next_status: 0xa5,
                adcsra_reads: Vec::new(),
                adcsra_read_index: Cell::new(0),
                adcl: 0,
                adch: 0,
            }
        }

        /// Constructs a hardware test double preloaded with adc10 for a deterministic conversion trace.
        fn with_adc10(adcsra_reads: Vec<Byte>, adcl: Byte, adch: Byte) -> Self {
            Self {
                adcsra_reads,
                adcl,
                adch,
                ..Self::default()
            }
        }

        /// Queries recorded hardware events for count events so timing tests can assert order as well as final values.
        fn count_events(&self, event: Event) -> usize {
            self.events
                .borrow()
                .iter()
                .filter(|candidate| **candidate == event)
                .count()
        }

        /// Queries recorded hardware events for contains event so timing tests can assert order as well as final values.
        fn contains_event(&self, event: Event) -> bool {
            self.events.borrow().contains(&event)
        }

        /// Queries recorded hardware events for first event so timing tests can assert order as well as final values.
        fn first_event(&self) -> Option<Event> {
            self.events.borrow().first().copied()
        }

        /// Queries recorded hardware events for last event so timing tests can assert order as well as final values.
        fn last_event(&self) -> Option<Event> {
            self.events.borrow().last().copied()
        }

        /// Reports whether event window without mutating device state.
        fn has_event_window(&self, window: &[Event]) -> bool {
            self.events
                .borrow()
                .windows(window.len())
                .any(|candidate| candidate == window)
        }
    }

    impl AdacHardware for TestHardware {
        fn set_signal(&mut self, signal: Signal, high: bool) {
            self.events.borrow_mut().push(Event::Signal(signal, high));
        }

        /// Samples signal directly from its mapped input pin during the bit-level peripheral transaction.
        fn read_signal(&self, signal: Signal) -> bool {
            self.events.borrow_mut().push(Event::Read(signal));
            let index = self.input_index.get();
            self.input_index.set(index + 1);
            self.input_bits[index]
        }

        fn set_port_c(&mut self, value: Byte) {
            self.events.borrow_mut().push(Event::PortC(value));
        }

        fn set_admux(&mut self, value: Byte) {
            self.events.borrow_mut().push(Event::Admux(value));
        }

        /// Writes adcsra to the serial, display, or peripheral destination selected by the implementation.
        fn write_adcsra(&mut self, value: Byte) {
            self.events.borrow_mut().push(Event::AdcsraWrite(value));
        }

        /// Reads the AVR adcsra register used to detect completion and assemble the 10-bit conversion.
        fn read_adcsra(&self) -> Byte {
            let index = self.adcsra_read_index.get();
            let value = self.adcsra_reads.get(index).copied().unwrap_or(0);
            self.adcsra_read_index.set(index + 1);
            self.events.borrow_mut().push(Event::AdcsraRead(value));
            value
        }

        /// Reads the AVR adcl register used to detect completion and assemble the 10-bit conversion.
        fn read_adcl(&self) -> Byte {
            self.events.borrow_mut().push(Event::AdclRead(self.adcl));
            self.adcl
        }

        /// Reads the AVR adch register used to detect completion and assemble the 10-bit conversion.
        fn read_adch(&self) -> Byte {
            self.events.borrow_mut().push(Event::AdchRead(self.adch));
            self.adch
        }

        /// Marks the begin interrupt exclusion boundary so interrupt state is saved and restored around the complete transaction.
        fn begin_interrupt_exclusion(&mut self) -> Byte {
            self.events
                .borrow_mut()
                .push(Event::BeginInterruptExclusion);
            self.next_status
        }

        /// Marks the end interrupt exclusion boundary so interrupt state is saved and restored around the complete transaction.
        fn end_interrupt_exclusion(&mut self, saved_status: Byte) {
            self.events
                .borrow_mut()
                .push(Event::EndInterruptExclusion(saved_status));
        }

        /// Provides the nop timing gap required between peripheral signal edges.
        fn nop(&mut self) {
            self.events.borrow_mut().push(Event::Nop);
        }

        /// Waits for cycles so callers cannot consume a stale hardware result.
        fn wait_cycles(&mut self, cycles: u16) {
            self.events.borrow_mut().push(Event::WaitCycles(cycles));
        }

        /// Waits for for adc10 complete so callers cannot consume a stale hardware result.
        fn wait_for_adc10_complete(&mut self) {
            self.events.borrow_mut().push(Event::WaitForAdc10Complete);
        }
    }

    /// Verifies that shift in1864 blocks interrupts for the whole ltc1864 transaction remains faithful to the Pascal behavior.
    #[test]
    fn shift_in1864_blocks_interrupts_for_the_whole_ltc1864_transaction() {
        let mut hw = TestHardware::with_input_word(0xb65a);
        let mut state = AdacState::default();

        shift_in1864(&mut hw, &mut state);

        assert_eq!(state.ad_raw, 0xb65a);
        assert_eq!(hw.first_event(), Some(Event::BeginInterruptExclusion));
        assert_eq!(hw.last_event(), Some(Event::EndInterruptExclusion(0xa5)));
        assert_eq!(hw.count_events(Event::Read(Signal::SDataIn1)), 16);
        assert_eq!(hw.count_events(Event::Nop), 4);
    }

    /// Verifies that DAC shift routines keep pascal nop timing remains faithful to the Pascal behavior.
    #[test]
    fn dac_shift_routines_keep_pascal_nop_timing() {
        let state = AdacState {
            dac_temp: 0xa55a,
            ..AdacState::default()
        };

        let mut hw = TestHardware::default();
        shift_out1257(&mut hw, &state);
        assert_eq!(hw.count_events(Event::Nop), 12);

        let mut hw = TestHardware::default();
        shift_out1655(&mut hw, &state);
        assert_eq!(hw.count_events(Event::Nop), 8);

        let mut hw = TestHardware::default();
        shift_out714(&mut hw, &state);
        assert_eq!(hw.count_events(Event::Nop), 19);
    }

    /// Verifies that shift register strobe keeps pascal nop timing remains faithful to the Pascal behavior.
    #[test]
    fn shift_register_strobe_keeps_pascal_nop_timing() {
        let state = AdacState {
            port_sr0: 0x11,
            port_sr1: 0x22,
            port_sr2: 0x44,
            port_sr3: 0x88,
            ..AdacState::default()
        };
        let mut hw = TestHardware::default();

        shift_out_sr(&mut hw, &state);

        assert_eq!(hw.count_events(Event::Nop), 2);
        assert!(hw.has_event_window(&[
            Event::Signal(Signal::StrSr, true),
            Event::Nop,
            Event::Nop,
            Event::Signal(Signal::StrSr, false),
        ]));
    }

    /// Verifies that on sys tick uses pascal delay loop cycle counts remains faithful to the Pascal behavior.
    #[test]
    fn on_sys_tick_uses_pascal_delay_loop_cycle_counts() {
        let mut hw = TestHardware::with_input_word(0x8004);
        hw.input_bits
            .extend((0..16).rev().map(|bit| 0x8008 & (1 << bit) != 0));
        hw.input_bits
            .extend((0..16).rev().map(|bit| 0x800c & (1 << bit) != 0));
        hw.input_bits
            .extend((0..16).rev().map(|bit| 0x8010 & (1 << bit) != 0));

        let mut state = AdacState {
            adc16_present: true,
            mux_ch: 7,
            ..AdacState::default()
        };

        on_sys_tick(&mut hw, &mut state);

        assert!(hw.contains_event(Event::WaitCycles(ADC16_DISCARD_CONVERSION_DELAY_CYCLES)));
        assert!(hw.contains_event(Event::WaitCycles(DAC_SETTLE_DELAY_CYCLES)));
        assert!(!hw.contains_event(Event::WaitCycles(15)));
        assert!(!hw.contains_event(Event::WaitCycles(4)));
    }

    /// Verifies that get adc10 matches pascal register sequence remains faithful to the Pascal behavior.
    #[test]
    fn get_adc10_matches_pascal_register_sequence() {
        let mut hw =
            TestHardware::with_adc10(vec![ADCSRA_BUSY_BIT, ADCSRA_BUSY_BIT, 0], 0x34, 0x12);

        let result = get_adc10(&mut hw, 5, true);

        assert_eq!(result, 0x1234);
        assert_eq!(
            *hw.events.borrow(),
            vec![
                Event::Admux(ADC10_INTERNAL_REFERENCE_MASK | 4),
                Event::WaitCycles(ADC10_SETTLE_DELAY_CYCLES),
                Event::AdcsraWrite(ADCSRA_START_DIV128),
                Event::AdcsraRead(ADCSRA_BUSY_BIT),
                Event::AdcsraRead(ADCSRA_BUSY_BIT),
                Event::AdcsraRead(0),
                Event::AdclRead(0x34),
                Event::AdchRead(0x12),
            ]
        );
    }

    /// Verifies that get adc10 wraps and masks pascal byte channel remains faithful to the Pascal behavior.
    #[test]
    fn get_adc10_wraps_and_masks_pascal_byte_channel() {
        let mut hw = TestHardware::with_adc10(vec![0], 0, 0);

        let result = get_adc10(&mut hw, 0, false);

        assert_eq!(result, 0);
        assert_eq!(
            hw.events.borrow().first(),
            Some(&Event::Admux(ADC10_CHANNEL_MASK))
        );
    }
}
