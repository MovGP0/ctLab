use crate::test_failures::TestFailures;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    SetBit(PortKind, u8),
    ClearBit(PortKind, u8),
    Nop,
    DelayUnits(u8),
    DelayMs(u16),
    BeginCriticalSection,
    EndCriticalSection,
}

#[derive(Default)]
struct MockIo {
    events: Vec<Event>,
}

impl MockIo {
    fn count_events(&self, expected: Event) -> usize {
        self.events
            .iter()
            .filter(|event| **event == expected)
            .count()
    }

    fn event_index(&self, expected: Event) -> usize {
        self.events
            .iter()
            .position(|event| *event == expected)
            .expect("event was not recorded")
    }

    fn shift_register_bytes(&self) -> Vec<u8> {
        let mut data_high = false;
        let mut bits = Vec::new();

        for event in &self.events {
            match event {
                Event::SetBit(PortKind::DdsOut, B_SDATAOUT) => {
                    data_high = true;
                }
                Event::ClearBit(PortKind::DdsOut, B_SDATAOUT) => {
                    data_high = false;
                }
                Event::SetBit(PortKind::DdsOut, B_SCLK) => {
                    bits.push(u8::from(data_high));
                }
                Event::SetBit(PortKind::DdsOut, B_STROBE) => {
                    break;
                }
                _ => {}
            }
        }

        bits.chunks_exact(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .fold(0_u8, |byte, bit| (byte << 1) | bit)
            })
            .collect()
    }
}

impl DdsHardwareIo for MockIo {
    fn set_bit(&mut self, port: PortKind, bit: u8) {
        self.events.push(Event::SetBit(port, bit));
    }

    fn clear_bit(&mut self, port: PortKind, bit: u8) {
        self.events.push(Event::ClearBit(port, bit));
    }

    fn nop(&mut self) {
        self.events.push(Event::Nop);
    }

    fn delay_units(&mut self, units: u8) {
        self.events.push(Event::DelayUnits(units));
    }

    fn delay_ms(&mut self, milliseconds: u16) {
        self.events.push(Event::DelayMs(milliseconds));
    }

    fn begin_critical_section(&mut self) {
        self.events.push(Event::BeginCriticalSection);
    }

    fn end_critical_section(&mut self) {
        self.events.push(Event::EndCriticalSection);
    }
}

#[test]
fn integer_tuning_word_matches_pascal_digit_sum() {
    let mut assert = TestFailures::default();

    assert.eq(DdsHardwareState::dds_tuning_word_integer(10_000), 64_000);
    assert.eq(DdsHardwareState::dds_tuning_word_integer(12_345_670), 79_012_288);
    assert.finish();
}

#[test]
fn tuning_word_is_split_into_ad9833_frequency_frames() {
    let mut assert = TestFailures::default();

    let frames = DdsHardwareState::dds_frequency_frames(0x0123_4567);
    assert.eq(frames, [0x4567, 0x448d]);
    assert.finish();
}

#[test]
fn set_level_dds_runs_without_touching_other_state_files() {
    let mut assert = TestFailures::default();

    let mut state = DdsHardwareState {
        dac_level: 80.0,
        frequency_tenths_hz: 10_000,
        ..Default::default()
    };
    let mut io = MockIo::default();
    state.set_level_dds(&mut io, Waveform::Sine);
    assert.eq(state.dds_frequency_word, 64_000);
    assert.finish();
}

#[test]
fn set_level_dds_sqg_preserves_existing_shift_register_payload() {
    let mut assert = TestFailures::default();

    let mut state = DdsHardwareState {
        dac_level: 80.0,
        level_byte_hi: 0x12,
        level_byte_lo: 0x34,
        frequency_tenths_hz: 10_000,
        ..Default::default()
    };
    let mut io = MockIo::default();

    state.set_level_dds_sqg(&mut io, Waveform::Sine);

    assert.eq(state.switch_state, 1 << TWO_SR_ATTN_SW_BIT);
    assert.eq(state.level_byte_hi, 0x12 | (1 << TWO_SR_ATTN_SW_BIT));
    assert.eq(state.level_byte_lo, 0x34);
    assert.eq(
        io.shift_register_bytes(),
        vec![state.switch_state, state.level_byte_hi, state.level_byte_lo],
    );
    assert.finish();
}

#[test]
fn set_level_dds_masks_interrupts_around_pascal_dds_writes() {
    let mut assert = TestFailures::default();

    let mut state = DdsHardwareState {
        dac_level: 120.0,
        frequency_tenths_hz: 10_000,
        ..Default::default()
    };
    let mut io = MockIo::default();

    state.set_level_dds(&mut io, Waveform::Sine);

    let begin = io.event_index(Event::BeginCriticalSection);
    let end = io.event_index(Event::EndCriticalSection);
    assert.is_true(begin < end);
    assert.eq(io.count_events(Event::BeginCriticalSection), 1);
    assert.eq(io.count_events(Event::EndCriticalSection), 1);
    assert.eq(io.count_events(Event::ClearBit(PortKind::DdsOut, B_FSYNC)), 3);
    assert.eq(io.count_events(Event::SetBit(PortKind::DdsOut, B_FSYNC)), 3);

    for (index, event) in io.events.iter().enumerate() {
        if matches!(
            event,
            Event::ClearBit(PortKind::DdsOut, B_FSYNC)
                | Event::SetBit(PortKind::DdsOut, B_FSYNC)
        ) {
            assert.is_true(index > begin && index < end);
        }
    }
    assert.finish();
}

#[test]
fn shift_out_1257_preserves_pascal_clock_hold_nops() {
    let mut assert = TestFailures::default();

    let mut state = DdsHardwareState::default();
    let mut io = MockIo::default();

    state.shift_out_1257(&mut io, 0);

    assert.eq(io.count_events(Event::Nop), 12);

    let load_index = io.event_index(Event::ClearBit(PortKind::ControlBit, B_STRDAC));
    assert.eq(io.events[load_index + 1], Event::Nop);
    assert.eq(io.events[load_index + 2], Event::ClearBit(PortKind::ControlBit, B_SDATAOUT));
    assert.eq(io.events[load_index + 3], Event::ClearBit(PortKind::ControlBit, B_SCLK));
    assert.eq(io.events[load_index + 4], Event::SetBit(PortKind::ControlBit, B_STRDAC));
    assert.finish();
}

#[test]
fn shift_out_level_sr_preserves_pascal_strobe_hold_nops() {
    let mut assert = TestFailures::default();

    let mut state = DdsHardwareState::default();
    let mut io = MockIo::default();

    state.shift_out_level_sr(&mut io, 0x1234);

    let strobe_index = io.event_index(Event::SetBit(PortKind::DdsOut, B_STROBE));
    assert.eq(io.events[strobe_index + 1], Event::Nop);
    assert.eq(io.events[strobe_index + 2], Event::Nop);
    assert.eq(io.events[strobe_index + 3], Event::ClearBit(PortKind::DdsOut, B_STROBE));
    assert.eq(io.events[strobe_index + 4], Event::SetBit(PortKind::DdsOut, B_SCLK));
    assert.finish();
}

#[test]
fn ser_aux_preserves_pascal_mp3_serial_bit_timing() {
    let mut assert = TestFailures::default();

    let state = DdsHardwareState::default();
    let mut io = MockIo::default();

    state.ser_aux(&mut io, 0b1010_0101);

    let delays: Vec<u8> = io
        .events
        .iter()
        .filter_map(|event| match event {
            Event::DelayUnits(units) => Some(*units),
            _ => None,
        })
        .collect();
    let serial_edges: Vec<Event> = io
        .events
        .iter()
        .copied()
        .filter(|event| {
            matches!(
                event,
                Event::SetBit(PortKind::Extension, B_SER_AUX)
                    | Event::ClearBit(PortKind::Extension, B_SER_AUX)
            )
        })
        .collect();

    assert.eq(delays, [5, 5, 5, 5, 5, 5, 5, 5, 5, 10]);
    assert.eq(
        serial_edges,
        [
            Event::ClearBit(PortKind::Extension, B_SER_AUX),
            Event::SetBit(PortKind::Extension, B_SER_AUX),
            Event::ClearBit(PortKind::Extension, B_SER_AUX),
            Event::SetBit(PortKind::Extension, B_SER_AUX),
            Event::ClearBit(PortKind::Extension, B_SER_AUX),
            Event::ClearBit(PortKind::Extension, B_SER_AUX),
            Event::SetBit(PortKind::Extension, B_SER_AUX),
            Event::ClearBit(PortKind::Extension, B_SER_AUX),
            Event::SetBit(PortKind::Extension, B_SER_AUX),
            Event::SetBit(PortKind::Extension, B_SER_AUX),
        ],
    );
    assert.finish();
}
