use super::*;
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Event {
    Control(ControlBit, bool),
    Read(ControlBit),
    TriggerOut(bool),
    TriggerInRead,
    Ad16Mpx(bool),
    Admux(u8),
    AdcsraWrite(u8),
    AdcsraRead(u8),
    AdclRead(u8),
    AdchRead(u8),
    BeginInterruptExclusion,
    EndInterruptExclusion(u8),
    Nop,
}

#[derive(Debug)]
struct MockHardware {
    events: RefCell<Vec<Event>>,
    input_bits: Vec<bool>,
    input_index: Cell<usize>,
    trigger_in: bool,
    saved_status: u8,
}

impl Default for MockHardware {
    fn default() -> Self {
        Self {
            events: RefCell::new(Vec::new()),
            input_bits: Vec::new(),
            input_index: Cell::new(0),
            trigger_in: true,
            saved_status: 0xa5,
        }
    }
}

impl MockHardware {
    fn with_adc16_word(word: u16) -> Self {
        Self {
            input_bits: (0..16).rev().map(|bit| word & (1 << bit) != 0).collect(),
            ..Self::default()
        }
    }

    fn count_events(&self, event: Event) -> usize {
        self.events
            .borrow()
            .iter()
            .filter(|candidate| **candidate == event)
            .count()
    }

    fn event_index(&self, event: Event) -> usize {
        self.events
            .borrow()
            .iter()
            .position(|candidate| *candidate == event)
            .expect("event was not recorded")
    }

    fn first_event(&self) -> Option<Event> {
        self.events.borrow().first().copied()
    }

    fn last_event(&self) -> Option<Event> {
        self.events.borrow().last().copied()
    }

    fn events_snapshot(&self) -> Vec<Event> {
        self.events.borrow().clone()
    }
}

impl EdlHardware for MockHardware {
    fn set_control_bit(&mut self, bit: ControlBit, high: bool) {
        self.events.borrow_mut().push(Event::Control(bit, high));
    }

    fn read_control_bit(&self, bit: ControlBit) -> bool {
        self.events.borrow_mut().push(Event::Read(bit));
        if bit != ControlBit::SDataIn1 {
            return false;
        }

        let index = self.input_index.get();
        self.input_index.set(index + 1);
        self.input_bits[index]
    }

    fn set_trigger_out(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::TriggerOut(high));
    }

    fn read_trigger_in(&self) -> bool {
        self.events.borrow_mut().push(Event::TriggerInRead);
        self.trigger_in
    }

    fn set_ad16_mpx(&mut self, high: bool) {
        self.events.borrow_mut().push(Event::Ad16Mpx(high));
    }

    fn set_admux(&mut self, value: u8) {
        self.events.borrow_mut().push(Event::Admux(value));
    }

    fn write_adcsra(&mut self, value: u8) {
        self.events.borrow_mut().push(Event::AdcsraWrite(value));
    }

    fn read_adcsra(&self) -> u8 {
        self.events.borrow_mut().push(Event::AdcsraRead(0));
        0
    }

    fn read_adcl(&self) -> u8 {
        self.events.borrow_mut().push(Event::AdclRead(0));
        0
    }

    fn read_adch(&self) -> u8 {
        self.events.borrow_mut().push(Event::AdchRead(0));
        0
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
}

#[test]
fn shift_in_1864_excludes_interrupts_for_the_full_adc16_clock_train() {
    let mut hw = EdlHw::new(MockHardware::with_adc16_word(0xb65a));

    hw.shift_in_1864();

    assert_eq!(hw.state.ad16_temp, 0xb65a);
    assert_eq!(hw.io.first_event(), Some(Event::BeginInterruptExclusion));
    assert_eq!(hw.io.last_event(), Some(Event::EndInterruptExclusion(0xa5)));
    assert_eq!(hw.io.count_events(Event::Read(ControlBit::SDataIn1)), 16);
    assert_eq!(hw.io.count_events(Event::Nop), 3);
    let events = hw.io.events_snapshot();
    assert_eq!(
        &events[events.len() - 3..],
        &[
            Event::Control(ControlBit::StrAd16, true),
            Event::Control(ControlBit::Sclk, false),
            Event::EndInterruptExclusion(0xa5),
        ]
    );
}

#[test]
fn on_sys_tick_starts_with_interrupt_safe_adc16_read_before_pwm_work() {
    let mut hw = EdlHw::new(MockHardware::with_adc16_word(0x2468));
    hw.state.pw_on_off = true;
    hw.state.pw_counter = 2;
    hw.state.dac_type = DacType::Dac8501;
    hw.state.dac_temp_on = 0x1234;

    hw.on_sys_tick();

    let begin = hw.io.event_index(Event::BeginInterruptExclusion);
    let end = hw.io.event_index(Event::EndInterruptExclusion(0xa5));
    let trigger = hw.io.event_index(Event::TriggerInRead);

    assert_eq!(begin, 0);
    assert!(begin < end);
    assert!(end < trigger);
    assert_eq!(hw.state.ad16_temp, 0x2468);
}

#[test]
fn get_adc10_keeps_mux_settle_before_starting_conversion() {
    let mut hw = EdlHw::new(MockHardware::default());

    let value = hw.get_adc10(4);

    assert_eq!(value, 0);
    let events = hw.io.events_snapshot();
    assert_eq!(events[0], Event::Admux(3));
    assert_eq!(
        events[1..=ADC10_SETTLE_CYCLES],
        [Event::Nop; ADC10_SETTLE_CYCLES]
    );
    assert_eq!(
        events[ADC10_SETTLE_CYCLES + 1],
        Event::AdcsraWrite(ADCSRA_START_DIV128)
    );
    assert_eq!(events[ADC10_SETTLE_CYCLES + 2], Event::AdcsraRead(0));
    assert_eq!(events[ADC10_SETTLE_CYCLES + 3], Event::AdclRead(0));
    assert_eq!(events[ADC10_SETTLE_CYCLES + 4], Event::AdchRead(0));
}
