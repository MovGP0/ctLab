use crate::test_failures::TestFailures;
use super::*;
use std::cell::{Cell, RefCell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Event {
    SDataOut(bool),
    Sclk(bool),
    StrDac(bool),
    StrAd16(bool),
    MpxI(bool),
    MpxU(bool),
    Mpx1864(bool),
    SDataIn1Read(bool),
    Admux(u8),
    Delay(u16),
    Nop,
    AdcsraWrite(u8),
    AdcsraRead(u8),
    AdclRead(u8),
    AdchRead(u8),
    BeginInterruptExclusion,
    EndInterruptExclusion(u8),
    PostDacSettle,
}

#[derive(Debug)]
struct MockHardware {
    events: RefCell<Vec<Event>>,
    input_bits: Vec<bool>,
    input_index: Cell<usize>,
    adcsra_reads: Vec<u8>,
    adcsra_read_index: Cell<usize>,
    adcl: u8,
    adch: u8,
    saved_status: u8,
}

impl MockHardware {
    fn new(adcsra_reads: Vec<u8>, adcl: u8, adch: u8) -> Self {
        Self {
            events: RefCell::new(Vec::new()),
            input_bits: Vec::new(),
            input_index: Cell::new(0),
            adcsra_reads,
            adcsra_read_index: Cell::new(0),
            adcl,
            adch,
            saved_status: 0xa5,
        }
    }

    fn with_input_word(word: u16) -> Self {
        let input_bits = (0..16).rev().map(|bit| word & (1 << bit) != 0).collect();

        Self {
            input_bits,
            ..Self::new(Vec::new(), 0, 0)
        }
    }

    fn events_snapshot(&self) -> Vec<Event> {
        self.events.borrow().clone()
    }

    fn count_events(&self, event: Event) -> usize {
        self.events
            .borrow()
            .iter()
            .filter(|candidate| **candidate == event)
            .count()
    }
}

impl DcgHardware for MockHardware {
    fn set_sdata_out(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::SDataOut(high));
    }

    fn set_sclk(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::Sclk(high));
    }

    fn set_str_dac(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::StrDac(high));
    }

    fn set_str_ad16(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::StrAd16(high));
    }

    fn set_mpx_i(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::MpxI(high));
    }

    fn set_mpx_u(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::MpxU(high));
    }

    fn set_mpx_1864(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::Mpx1864(high));
    }

    fn read_sdata_in1(&self) -> bool {
        let index = self.input_index.get();
        let value = self.input_bits.get(index).copied().unwrap_or(false);
        self.input_index.set(index + 1);
        self.events.borrow_mut().push(Event::SDataIn1Read(value));
        value
    }

    fn spin_delay_cycles(&mut self, cycles: u16) {
        self.events.borrow_mut().push(Event::Delay(cycles));
    }

    fn set_admux(&mut self, value: u8) {
        self.events.borrow_mut().push(Event::Admux(value));
    }

    fn write_adcsra(&mut self, value: u8) {
        self.events.borrow_mut().push(Event::AdcsraWrite(value));
    }

    fn read_adcsra(&self) -> u8 {
        let index = self.adcsra_read_index.get();
        let value = self.adcsra_reads.get(index).copied().unwrap_or(0);
        self.adcsra_read_index.set(index + 1);
        self.events.borrow_mut().push(Event::AdcsraRead(value));
        value
    }

    fn read_adcl(&self) -> u8 {
        self.events.borrow_mut().push(Event::AdclRead(self.adcl));
        self.adcl
    }

    fn read_adch(&self) -> u8 {
        self.events.borrow_mut().push(Event::AdchRead(self.adch));
        self.adch
    }

    fn begin_interrupt_exclusion(&mut self) -> u8 {
        self.events
            .borrow_mut()
            .push(Event::BeginInterruptExclusion);
        self.saved_status
    }

    fn end_interrupt_exclusion(&mut self, saved_status: u8) {
        self.events
            .borrow_mut()
            .push(Event::EndInterruptExclusion(saved_status));
    }

    fn nop(&mut self) {
        self.events.borrow_mut().push(Event::Nop);
    }

    fn wait_post_dac_settle(&mut self) {
        self.events.borrow_mut().push(Event::PostDacSettle);
    }
}

#[test]
fn get_adc10_matches_pascal_register_sequence() {
    let mut assert = TestFailures::default();

    let mut hw = MockHardware::new(vec![ADCSRA_BUSY_BIT, ADCSRA_BUSY_BIT, 0], 0x34, 0x12);

    let result = get_adc10(&mut hw, 5);

    assert.eq(result, 0x1234);
    assert.eq(
        hw.events.borrow().as_slice(),
        [
            Event::Admux(4),
            Event::Delay(ADC10_SETTLE_CYCLES),
            Event::AdcsraWrite(ADCSRA_START_DIV128),
            Event::AdcsraRead(ADCSRA_BUSY_BIT),
            Event::AdcsraRead(ADCSRA_BUSY_BIT),
            Event::AdcsraRead(0),
            Event::AdclRead(0x34),
            Event::AdchRead(0x12),
        ],
    );
    assert.finish();
}

#[test]
fn get_adc10_wraps_and_masks_pascal_byte_channel() {
    let mut assert = TestFailures::default();

    let mut hw = MockHardware::new(vec![0], 0, 0);

    let result = get_adc10(&mut hw, 0);

    assert.eq(result, 0);
    assert.eq(hw.events.borrow().first(), Some(&Event::Admux(ADC10_CHANNEL_MASK)));
    assert.finish();
}

#[test]
fn shift_in_1864_masks_interrupts_and_waits_before_clocking_sample() {
    let mut assert = TestFailures::default();

    let mut hw = MockHardware::with_input_word(0xb65a);

    let result = shift_in_1864(&mut hw);

    assert.eq(result, 0xb65a);
    assert.eq(
        &hw.events_snapshot()[..6],
        [
            Event::BeginInterruptExclusion,
            Event::StrAd16(false),
            Event::Sclk(false),
            Event::Nop,
            Event::Nop,
            Event::Nop,
        ],
    );
    assert.eq(hw.count_events(Event::Nop), LTC1864_ACQUISITION_DELAY_CYCLES as usize);
    assert.eq(hw.count_events(Event::SDataIn1Read(true)), 9);
    assert.eq(hw.count_events(Event::SDataIn1Read(false)), 7);
    assert.eq(hw.events_snapshot().last(), Some(&Event::EndInterruptExclusion(0xa5)));
    assert.finish();
}

#[test]
fn on_sys_tick_selects_dac_from_dac16_present_flag() {
    let mut assert = TestFailures::default();

    let mut state = DcgHardwareState {
        dac16_present: false,
        dac_raw_i: 0x0abc,
        ..DcgHardwareState::default()
    };
    let mut hw = MockHardware::new(Vec::new(), 0, 0);

    on_sys_tick(&mut state, &mut hw);

    let events = hw.events_snapshot();
    let str_dac_events: Vec<Event> = events
        .iter()
        .copied()
        .filter(|event| matches!(event, Event::StrDac(_)))
        .collect();
    assert.eq(
        str_dac_events,
        vec![
            Event::StrDac(true),
            Event::StrDac(false),
            Event::StrDac(true)
        ],
    );

    state.dac16_present = true;
    let mut hw = MockHardware::new(Vec::new(), 0, 0);

    on_sys_tick(&mut state, &mut hw);

    let events = hw.events_snapshot();
    let str_dac_events: Vec<Event> = events
        .iter()
        .copied()
        .filter(|event| matches!(event, Event::StrDac(_)))
        .collect();
    assert.eq(str_dac_events, vec![Event::StrDac(false), Event::StrDac(true)]);
    assert.finish();
}

#[test]
fn on_sys_tick_waits_for_post_dac_settle_before_mux_update() {
    let mut assert = TestFailures::default();

    let mut state = DcgHardwareState {
        dac16_present: true,
        dac_raw_i: 0x1234,
        ..DcgHardwareState::default()
    };
    let mut hw = MockHardware::new(Vec::new(), 0, 0);

    on_sys_tick(&mut state, &mut hw);

    let events = hw.events_snapshot();
    let settle_index = events
        .iter()
        .position(|event| *event == Event::PostDacSettle)
        .unwrap();
    let latch_index = events
        .iter()
        .rposition(|event| *event == Event::StrDac(true))
        .unwrap();
    let adc_store_index = events
        .iter()
        .position(|event| *event == Event::Mpx1864(false))
        .unwrap();
    let output_enable_index = events
        .iter()
        .position(|event| *event == Event::MpxI(false))
        .unwrap();

    assert.is_true(latch_index < settle_index);
    assert.is_true(settle_index < adc_store_index);
    assert.is_true(settle_index < output_enable_index);
    assert.eq(hw.count_events(Event::PostDacSettle), 1);
    assert.finish();
}
