use super::*;
use std::cell::{Cell, RefCell};

/// Records observable hardware operations so tests can assert exact edge and register order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
    /// Records an LTC2400 conversion-strobe transition.
    StrAd24(bool),

    /// Records an LTC2400 serial-clock transition.
    Sclk(bool),

    /// Records the converter data bit sampled on a clock edge.
    SdataRead(bool),

    /// Records a write to the SPI control register.
    SpiControl(u8),

    /// Records one transmitted and received SPI byte pair.
    SpiTransfer {
        /// Records the byte written to the SPI data register.
        tx: u8,

        /// Records the scripted byte returned by the converter.
        rx: u8,
    },

    /// Records an intentional cycle delay between signal edges.
    Delay(u16),
}

/// Captures each LTC2400 signal edge and returns scripted data bits so tests can verify transaction order and timing.
#[derive(Debug)]
struct MockHardware {
    /// Records events in order so tests can verify every externally visible operation.
    events: RefCell<Vec<Event>>,

    /// Records sdata reads in order so tests can verify every externally visible operation.
    sdata_reads: Vec<bool>,

    /// Tracks sdata read index while the corresponding bounded sequence is processed.
    sdata_read_index: Cell<usize>,

    /// Records SPI reads in order so tests can verify every externally visible operation.
    spi_reads: Vec<u8>,

    /// Tracks SPI read index while the corresponding bounded sequence is processed.
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

    /// Samples sdata in1 directly from its mapped input pin during the bit-level peripheral transaction.
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

    /// Transfers SPI transfer using the byte order expected by the attached peripheral.
    fn spi_transfer(&mut self, tx: u8) -> u8 {
        let index = self.spi_read_index.get();
        let rx = self.spi_reads.get(index).copied().unwrap_or(0);
        self.spi_read_index.set(index + 1);
        self.events.borrow_mut().push(Event::SpiTransfer { tx, rx });
        rx
    }

    /// Burns the requested processor cycles between signal edges where the peripheral data sheet requires setup or hold time.
    fn spin_delay_cycles(&mut self, cycles: u16) {
        self.events.borrow_mut().push(Event::Delay(cycles));
    }
}

/// Verifies that shift in 2400 negative reading suppresses positive clipping remains faithful to the Pascal behavior.
#[test]
fn shift_in_2400_negative_reading_suppresses_positive_clipping() {
    let mut state = DivHardwareState::default();
    let mut hw = MockHardware::new(vec![false, true], vec![0x12, 0x34, 0x56]);

    shift_in_2400(&mut state, &mut hw);

    assert!(state.negative_flag);
    assert!(!state.over_voltage_flag);
    assert_eq!(state.ad24_temp, 0xFF12_3456);
}

/// Verifies that shift in 2400 uses pascal ltc2400 SPI transaction remains faithful to the Pascal behavior.
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

/// Verifies that on sys tick abort uses pascal manual clock timing remains faithful to the Pascal behavior.
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

/// Verifies that int2 trigger only accepts falling edge remains faithful to the Pascal behavior.
#[test]
fn int2_trigger_only_accepts_falling_edge() {
    let mut state = DivHardwareState::default();

    int2_trigger_edge(&mut state, true);
    assert!(!state.trigger);

    int2_trigger(&mut state);
    assert!(state.trigger);
}
