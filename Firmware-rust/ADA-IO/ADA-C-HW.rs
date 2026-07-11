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
#[path = "ADA-C-HW_tests.rs"]
mod tests;
