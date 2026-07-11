//! Best-effort Rust port of `DIV-HW.pas`.

use core::marker::PhantomData;

use crate::avrd_support::{Atmega32, AvrdPortIo, Mcu, RegisterPort};

#[path = "div_hw/div_hardware.rs"]
mod div_hardware;
pub use div_hardware::DivHardware;
#[path = "div_hw/div_external_interrupt_mcu.rs"]
mod div_external_interrupt_mcu;
pub use div_external_interrupt_mcu::DivExternalInterruptMcu;
#[path = "div_hw/div_avrd.rs"]
mod div_avrd;
pub use div_avrd::DivAvrd;
#[path = "div_hw/div_hardware_state.rs"]
mod div_hardware_state;
pub use div_hardware_state::DivHardwareState;

/// Enables AVR SPI as master at the clock phase/polarity used to shift the LTC2400 payload.
pub const LTC2400_SPI_CONTROL: u8 = 0b0101_0001;

/// Clears SPCR after an LTC2400 read so manual clock/strobe control resumes.
pub const LTC2400_SPI_DISABLED: u8 = 0;

/// Fixes `DivAtmega32` to the integer width used by the AVR-facing Pascal declarations.
pub type DivAtmega32 = DivAvrd<Atmega32>;

/// Reads the LTC2400 status nibble and 24-bit payload, sign-extends negative results, clamps positive overrange, and clocks four trailing bits.
pub fn shift_in_2400<H: DivHardware>(state: &mut DivHardwareState, hw: &mut H) {
    hw.set_str_ad24(false);

    for bit_index in 0..4 {
        hw.set_sclk(true);
        // The LTC2400 presents its status bits before the 24 data bits:
        // bit 2 indicates the signed/clipping state and bit 3 the overrange flag.
        if bit_index == 2 {
            state.negative_flag = !hw.read_sdata_in1();
        }
        if bit_index == 3 {
            state.over_voltage_flag = hw.read_sdata_in1();
        }
        hw.set_sclk(false);
    }

    hw.set_spi_control(LTC2400_SPI_CONTROL);
    let b2 = hw.spi_transfer(LTC2400_SPI_CONTROL);
    let b1 = hw.spi_transfer(b2);
    let b0 = hw.spi_transfer(b1);
    hw.set_spi_control(LTC2400_SPI_DISABLED);

    // Clock out the remaining four trailing dither bits after the 24-bit payload.
    for _ in 0..4 {
        hw.spin_delay_cycles(1);
        hw.set_sclk(true);
        hw.spin_delay_cycles(1);
        hw.set_sclk(false);
    }

    // Negative readings are sign-extended to preserve the LTC2400 two's-complement format.
    if state.negative_flag {
        state.over_voltage_flag = false;
    }

    let msb = if state.negative_flag { 0xFF } else { 0 };
    state.ad24_temp = ((msb as u32) << 24) | ((b2 as u32) << 16) | ((b1 as u32) << 8) | (b0 as u32);

    // Overrange is treated as hard clipping and forced to the full-scale positive code.
    if state.over_voltage_flag {
        state.ad24_temp = 16_777_215;
    }

    hw.set_str_ad24(true);
}

/// Latches int2 trigger for deferred processing outside the interrupt-sensitive edge handler.
pub fn int2_trigger(state: &mut DivHardwareState) {
    int2_trigger_edge(state, false);
}

/// Latches int2 trigger edge for deferred processing outside the interrupt-sensitive edge handler.
pub fn int2_trigger_edge(state: &mut DivHardwareState, positive_edge: bool) {
    if !positive_edge {
        state.trigger = true;
    }
}

/// Handles sys tick as one bounded polling-loop or interrupt service step.
pub fn on_sys_tick<H: DivHardware>(state: &mut DivHardwareState, hw: &mut H) {
    hw.set_sclk(false);
    hw.set_str_ad24(false);

    if state.abort_flag {
        // Abort clears a pending conversion by issuing a short manual SCLK pulse.
        hw.set_str_ad24(false);
        hw.spin_delay_cycles(2);
        hw.set_sclk(true);
        hw.spin_delay_cycles(2);
        hw.set_sclk(false);
        state.abort_flag = false;
    } else if !hw.read_sdata_in1() {
        // SDATA low is the LTC2400 end-of-conversion signal; only then is a read valid.
        hw.set_str_ad24(true);
        shift_in_2400(state, hw);

        // Fast integration is a simple 2-sample moving average for light smoothing.
        state.ad24_temp_fast_integrated = (state.ad24_temp + state.ad24_integrate0) / 2;
        state.ad24_integrate0 = state.ad24_temp_fast_integrated;

        // Slow integration averages the current sample with the previous three filter states.
        state.ad24_temp_slow_integrated = (state.ad24_temp
            + state.ad24_integrate1
            + state.ad24_integrate2
            + state.ad24_integrate3)
            / 4;
        state.ad24_integrate3 = state.ad24_integrate2;
        state.ad24_integrate2 = state.ad24_integrate1;
        state.ad24_integrate1 = state.ad24_temp_slow_integrated;

        // Marks that the 24-bit conversion path has fresh data for the foreground loop.
        state.ad24_ready = true;
    }

    hw.set_str_ad24(true);
    // The original firmware also used the systick as the update point for the 10-bit path.
    state.ad10_ready = true;
}

#[cfg(test)]
#[path = "DIV-HW_tests.rs"]
mod tests;
