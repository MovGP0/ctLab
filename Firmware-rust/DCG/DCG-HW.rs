//! Best-effort Rust port of `DCG-HW.pas`.
//!
//! The original Pascal source mixes inline AVR assembly with module globals.
//! This port keeps the algorithmic structure and the hardware responsibilities
//! but expresses them through explicit state and a hardware access trait.

use core::marker::PhantomData;

use crate::avrd_support::{Atmega32, AvrdPortIo, Mcu, RegisterPort};

/// Keeps only AVR ADMUX channel bits 0 through 2 while preserving the configured reference bits.
pub const ADC10_CHANNEL_MASK: u8 = 0x07;

/// Enables the AVR ADC, starts conversion, and selects the divide-by-128 ADC clock in one register write.
pub const ADCSRA_START_DIV128: u8 = 0xC7;

/// Masks ADSC, which remains set while the AVR conversion is in progress.
pub const ADCSRA_BUSY_BIT: u8 = 1 << 6;

/// Allows the AVR ADC sample capacitor to settle after changing the analog multiplexer channel.
pub const ADC10_SETTLE_CYCLES: u16 = 15;

/// Provides the three idle CPU cycles required between LTC1864 channel selection and conversion clocking.
pub const LTC1864_ACQUISITION_DELAY_CYCLES: u16 = 3;

/// Holds the DCG DAC latch timing after a word so the converter accepts the new code before another edge.
pub const DAC_POST_WRITE_SETTLE_LOOP_ITERATIONS: u8 = 40;

/// AVR data-space address of SREG, used to save and restore the caller's interrupt-enable state around converter frames.
#[cfg(target_arch = "avr")]
const AVR_SREG_ADDRESS: *mut u8 = 0x5f as *mut u8;

/// Masks AVR SREG bit 7 so critical-section code can restore the caller's interrupt-enable state exactly.
#[cfg(target_arch = "avr")]
const AVR_SREG_INTERRUPT_ENABLE_MASK: u8 = 0x80;

#[path = "dcg_hw/dac_kind.rs"]
mod dac_kind;
pub use dac_kind::DacKind;

#[path = "dcg_hw/dcg_hardware.rs"]
mod dcg_hardware;
pub use dcg_hardware::DcgHardware;

#[path = "dcg_hw/dcg_avrd.rs"]
mod dcg_avrd;
pub use dcg_avrd::DcgAvrd;

#[path = "dcg_hw/dcg_hardware_state.rs"]
mod dcg_hardware_state;
pub use dcg_hardware_state::DcgHardwareState;

/// Concrete DCG hardware adapter binding the generic AVR implementation to the ATmega32 register map.
pub type DcgAtmega32 = DcgAvrd<Atmega32>;

/// Serializes the 12-bit DAC word with the original hold delays required by the converter and board wiring.
pub fn shift_out_1257<H: DcgHardware>(hw: &mut H, dac_temp: u16) {
    hw.set_sdata_out(false);
    hw.set_sclk(false);
    hw.set_str_dac(true);

    // The LTC1257 consumes a 12-bit value: four high bits first, then the low
    // byte. The Pascal code left-aligns the high nibble so bit 11 is shifted
    // out on the first clock edge.
    let mut high = ((dac_temp >> 8) as u8) << 4;
    for _ in 0..4 {
        hw.set_sdata_out((high & 0x80) != 0);
        hw.set_sclk(true);
        high <<= 1;
        hw.spin_delay_cycles(1);
        hw.set_sdata_out(false);
        hw.set_sclk(false);
    }

    let mut low = dac_temp as u8;
    for _ in 0..7 {
        hw.set_sdata_out((low & 0x80) != 0);
        hw.set_sclk(true);
        low <<= 1;
        hw.spin_delay_cycles(1);
        hw.set_sdata_out(false);
        hw.set_sclk(false);
    }

    // The last bit is transferred together with the DAC load strobe, matching
    // the original "LSB mit Load-Impuls" sequence in the AVR assembly.
    hw.set_sdata_out((low & 0x80) != 0);
    hw.set_sclk(true);
    hw.set_str_dac(false);
    hw.spin_delay_cycles(1);
    hw.set_sdata_out(false);
    hw.set_sclk(false);
    hw.set_str_dac(true);
}

/// Serializes the wider DAC command used by the alternate DCG hardware option.
pub fn shift_out_1655<H: DcgHardware>(hw: &mut H, dac_temp: u16) {
    hw.set_sclk(false);
    hw.set_sdata_out(false);
    hw.set_str_dac(false);

    // The LTC1655 uses a full 16-bit transfer and latches only after all bits
    // have been clocked out, so its framing differs from the 1257 path above.
    for byte in dac_temp.to_be_bytes() {
        let mut current = byte;
        for _ in 0..8 {
            hw.set_sdata_out((current & 0x80) != 0);
            hw.set_sclk(true);
            current <<= 1;
            hw.set_sdata_out(false);
            hw.set_sclk(false);
        }
    }

    hw.set_str_dac(true);
}

/// Clocks one complete LTC1864 sample while interrupts are excluded, preventing a partial word from corrupting the measurement.
pub fn shift_in_1864<H: DcgHardware>(hw: &mut H) -> u16 {
    // Pulling STRAD16 low starts the LTC1864 read cycle. The original code
    // masks interrupts around this routine so all 16 bits are sampled with
    // deterministic timing.
    let saved_status = hw.begin_interrupt_exclusion();

    hw.set_str_ad16(false);
    hw.set_sclk(false);

    for _ in 0..LTC1864_ACQUISITION_DELAY_CYCLES {
        hw.nop();
    }

    let mut result = 0u16;
    for _ in 0..16 {
        hw.set_sclk(true);
        let bit = hw.read_sdata_in1();
        hw.set_sclk(false);
        result = (result << 1) | u16::from(bit);
    }

    hw.set_sclk(true);
    hw.spin_delay_cycles(1);
    hw.set_str_ad16(true);
    hw.end_interrupt_exclusion(saved_status);
    result
}

/// Advances the interrupt-time phase machine that must keep ADC sampling and output timing deterministic.
pub fn on_sys_tick<H: DcgHardware>(state: &mut DcgHardwareState, hw: &mut H) {
    // The 1 ms SysTick ISR begins by disabling both analog output paths before
    // reading/updating shared converter state.
    hw.set_mpx_i(true);
    hw.set_mpx_u(false);

    if state.adc16_present {
        state.adc_temp = shift_in_1864(hw);
    }

    if state.ui_toggle {
        if state.pw_on_off {
            if state.pw_counter == 0 {
                state.pw_counter = state.pw_off_time;
                state.pw_on_off = false;
                state.dac_temp = state.dac_raw_u_off;
            } else {
                state.dac_temp = state.dac_raw_u_on;
            }
        } else if state.pw_counter == 0 {
            state.pw_counter = state.pw_on_time;
            state.pw_on_off = true;
            state.dac_temp = state.dac_raw_u_on;
        } else {
            state.dac_temp = state.dac_raw_u_off;
        }
    } else {
        state.dac_temp = state.dac_raw_i;
    }

    if state.pw_counter > 0 {
        state.pw_counter -= 1;
    }

    if state.dac16_present {
        shift_out_1655(hw, state.dac_temp);
    } else {
        shift_out_1257(hw, state.dac_temp);
    }

    // The Pascal ISR burns roughly 10 us here so the external DAC and analog
    // switches can settle before the multiplexers are flipped again.
    hw.wait_post_dac_settle();

    if state.ui_toggle {
        // On alternating ticks the firmware services the voltage path, folds
        // in the freshly sampled ADC value, then points both muxes at U.
        state.adc_raw_u = ((state.adc_raw_u as u32 + state.adc_temp as u32) / 2) as u16;
        hw.set_mpx_1864(true);
        hw.set_mpx_u(true);
    } else {
        // The other half-cycle does the same for current, so U and I share one
        // converter by time-multiplexing the front end every millisecond.
        state.adc_raw_i = ((state.adc_raw_i as u32 + state.adc_temp as u32) / 2) as u16;
        hw.set_mpx_1864(false);
        hw.set_mpx_i(false);
    }

    state.ui_toggle = !state.ui_toggle;
}

/// Selects the one-based AVR ADC channel, waits for mux settling, starts conversion, polls completion, then combines ADCL before ADCH as required by the AVR latch rule.
pub fn get_adc10<H: DcgHardware>(hw: &mut H, channel: u8) -> u16 {
    // Hand-coded equivalent of the Pascal getadc() helper: select the mux input,
    // wait for it to settle, start an ADC conversion with prescaler 128, then
    // poll ADSC until the conversion is done before reading ADCL/ADCH.
    hw.set_admux(channel.wrapping_sub(1) & ADC10_CHANNEL_MASK);
    hw.spin_delay_cycles(ADC10_SETTLE_CYCLES);
    hw.write_adcsra(ADCSRA_START_DIV128);

    while (hw.read_adcsra() & ADCSRA_BUSY_BIT) != 0 {}

    u16::from(hw.read_adcl()) | (u16::from(hw.read_adch()) << 8)
}

#[cfg(test)]
#[path = "DCG-HW_tests.rs"]
mod tests;
