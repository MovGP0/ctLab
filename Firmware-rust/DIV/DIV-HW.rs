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

pub const LTC2400_SPI_CONTROL: u8 = 0b0101_0001;
pub const LTC2400_SPI_DISABLED: u8 = 0;

pub type DivAtmega32 = DivAvrd<Atmega32>;

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

pub fn int2_trigger(state: &mut DivHardwareState) {
    int2_trigger_edge(state, false);
}

pub fn int2_trigger_edge(state: &mut DivHardwareState, positive_edge: bool) {
    if !positive_edge {
        state.trigger = true;
    }
}

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
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Event {
        StrAd24(bool),
        Sclk(bool),
        SdataRead(bool),
        SpiControl(u8),
        SpiTransfer { tx: u8, rx: u8 },
        Delay(u16),
    }

    #[derive(Debug)]
    struct MockHardware {
        events: RefCell<Vec<Event>>,
        sdata_reads: Vec<bool>,
        sdata_read_index: Cell<usize>,
        spi_reads: Vec<u8>,
        spi_read_index: Cell<usize>,
    }

    impl MockHardware {
        fn new(sdata_reads: Vec<bool>, spi_reads: Vec<u8>) -> Self {
            Self {
                events: RefCell::new(Vec::new()),
                sdata_reads,
                sdata_read_index: Cell::new(0),
                spi_reads,
                spi_read_index: Cell::new(0),
            }
        }
    }

    impl DivHardware for MockHardware {
        fn set_str_ad24(&mut self, high: bool) {
            self.events.borrow_mut().push(Event::StrAd24(high));
        }

        fn set_sclk(&mut self, high: bool) {
            self.events.borrow_mut().push(Event::Sclk(high));
        }

        fn read_sdata_in1(&self) -> bool {
            let index = self.sdata_read_index.get();
            let value = self.sdata_reads.get(index).copied().unwrap_or(false);
            self.sdata_read_index.set(index + 1);
            self.events.borrow_mut().push(Event::SdataRead(value));
            value
        }

        fn set_spi_control(&mut self, value: u8) {
            self.events.borrow_mut().push(Event::SpiControl(value));
        }

        fn spi_transfer(&mut self, tx: u8) -> u8 {
            let index = self.spi_read_index.get();
            let rx = self.spi_reads.get(index).copied().unwrap_or(0);
            self.spi_read_index.set(index + 1);
            self.events.borrow_mut().push(Event::SpiTransfer { tx, rx });
            rx
        }

        fn spin_delay_cycles(&mut self, cycles: u16) {
            self.events.borrow_mut().push(Event::Delay(cycles));
        }
    }

    #[test]
    fn shift_in_2400_negative_reading_suppresses_positive_clipping() {
        let mut state = DivHardwareState::default();
        let mut hw = MockHardware::new(vec![false, true], vec![0x12, 0x34, 0x56]);

        shift_in_2400(&mut state, &mut hw);

        assert!(state.negative_flag);
        assert!(!state.over_voltage_flag);
        assert_eq!(state.ad24_temp, 0xFF12_3456);
    }

    #[test]
    fn shift_in_2400_uses_pascal_ltc2400_spi_transaction() {
        let mut state = DivHardwareState::default();
        let mut hw = MockHardware::new(vec![true, true], vec![0xAA, 0xBB, 0xCC]);

        shift_in_2400(&mut state, &mut hw);

        assert!(!state.negative_flag);
        assert!(state.over_voltage_flag);
        assert_eq!(state.ad24_temp, 16_777_215);
        assert_eq!(
            *hw.events.borrow(),
            vec![
                Event::StrAd24(false),
                Event::Sclk(true),
                Event::Sclk(false),
                Event::Sclk(true),
                Event::Sclk(false),
                Event::Sclk(true),
                Event::SdataRead(true),
                Event::Sclk(false),
                Event::Sclk(true),
                Event::SdataRead(true),
                Event::Sclk(false),
                Event::SpiControl(LTC2400_SPI_CONTROL),
                Event::SpiTransfer {
                    tx: LTC2400_SPI_CONTROL,
                    rx: 0xAA,
                },
                Event::SpiTransfer { tx: 0xAA, rx: 0xBB },
                Event::SpiTransfer { tx: 0xBB, rx: 0xCC },
                Event::SpiControl(LTC2400_SPI_DISABLED),
                Event::Delay(1),
                Event::Sclk(true),
                Event::Delay(1),
                Event::Sclk(false),
                Event::Delay(1),
                Event::Sclk(true),
                Event::Delay(1),
                Event::Sclk(false),
                Event::Delay(1),
                Event::Sclk(true),
                Event::Delay(1),
                Event::Sclk(false),
                Event::Delay(1),
                Event::Sclk(true),
                Event::Delay(1),
                Event::Sclk(false),
                Event::StrAd24(true),
            ]
        );
    }

    #[test]
    fn on_sys_tick_abort_uses_pascal_manual_clock_timing() {
        let mut state = DivHardwareState {
            abort_flag: true,
            ..DivHardwareState::default()
        };
        let mut hw = MockHardware::new(vec![false], vec![]);

        on_sys_tick(&mut state, &mut hw);

        assert!(!state.abort_flag);
        assert!(!state.ad24_ready);
        assert!(state.ad10_ready);
        assert_eq!(hw.sdata_read_index.get(), 0);
        assert_eq!(
            *hw.events.borrow(),
            vec![
                Event::Sclk(false),
                Event::StrAd24(false),
                Event::StrAd24(false),
                Event::Delay(2),
                Event::Sclk(true),
                Event::Delay(2),
                Event::Sclk(false),
                Event::StrAd24(true),
            ]
        );
    }

    #[test]
    fn int2_trigger_only_accepts_falling_edge() {
        let mut state = DivHardwareState::default();

        int2_trigger_edge(&mut state, true);
        assert!(!state.trigger);

        int2_trigger(&mut state);
        assert!(state.trigger);
    }
}
