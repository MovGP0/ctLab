//! Best-effort Rust port of `DDS-HW.pas`.
//!
//! The original Pascal unit bit-bangs an AD9833 DDS, a LTC1257 offset DAC,
//! a 4094 shift register chain, and an auxiliary serial output. This port
//! keeps the hardware-facing constants and routines readable while replacing
//! direct AVR register access with a small I/O trait.

use core::marker::PhantomData;

use crate::avrd_support::{Atmega32, AvrdPortIo, Mcu, RegisterPort};
pub use crate::{Ad9833Control, Waveform};

/// AVR data-space address of SREG, used to save and restore the caller's interrupt-enable state around converter frames.
#[cfg(target_arch = "avr")]
const AVR_SREG_ADDRESS: *mut u8 = 0x5f as *mut u8;

/// Masks AVR SREG bit 7 so critical-section code can restore the caller's interrupt-enable state exactly.
#[cfg(target_arch = "avr")]
const AVR_SREG_INTERRUPT_ENABLE_MASK: u8 = 0x80;

/// Converts each Pascal auxiliary-delay unit into 160 CPU cycles for the YAMPP bit-banged baud rate.
pub const SER_AUX_DELAY_CYCLES_PER_UNIT: u16 = 160;

#[path = "dds_hw/port_kind.rs"]
mod port_kind;
pub use port_kind::PortKind;

#[path = "dds_hw/dds_hardware_io.rs"]
mod dds_hardware_io;
pub use dds_hardware_io::DdsHardwareIo;

#[path = "dds_hw/dds_avrd.rs"]
mod dds_avrd;
pub use dds_avrd::DdsAvrd;

#[path = "dds_hw/dds_hardware_state.rs"]
mod dds_hardware_state;
pub use dds_hardware_state::DdsHardwareState;

/// Concrete DDS hardware adapter binding the generic AVR implementation to the ATmega32 register map.
pub type DdsAtmega32 = DdsAvrd<Atmega32>;

/// DDS output-port bit driving the shared serial clock.
pub const B_SCLK: u8 = 0;

/// DDS output-port bit carrying AD9833, DAC, and relay serial data.
pub const B_SDATAOUT: u8 = 1;

/// DDS output-port bit framing each AD9833 sixteen-bit register write.
pub const B_FSYNC: u8 = 2;

/// Control-port bit latching the cascaded relay shift registers.
pub const B_STROBE: u8 = 3;

/// Control-port bit latching a completed amplitude-DAC word.
pub const B_STRDAC: u8 = 4;

/// Control-port bit used as the bit-banged auxiliary YAMPP transmit line.
pub const B_SER_AUX: u8 = 5;

/// AD9833 frame prefix selecting frequency register zero for each fourteen-bit tuning-word half.
pub const DDS_FREQ_REGISTER_WRITE: u16 = 0b0100_0000_0000_0000;

/// Integer decimal-decade factors used to assemble the 28-bit AD9833 tuning word without floating-point work.
#[rustfmt::skip]
pub const FHZ_INT: [i32; 8] = [
    64_000_000,
    6_400_000,
    640_000,
    64_000,
    6_400,
    640,
    64,
    6,
];

/// SQG decimal-decade factors for its reference clock and floating-point frequency representation.
#[rustfmt::skip]
pub const FHZ_SQG: [f32; 9] = [
    134_217_728.0,
    13_421_772.8,
    1_342_177.3,
    134_217.73,
    13_421.772,
    1_342.177_2,
    134.217_73,
    13.421_773,
    1.342_177_3,
];

/// Square-wave relay position in the two-register board's second payload byte.
const TWO_SR_SQUARE_SW_BIT: u8 = 4;

/// Attenuator relay position in the two-register board's second payload byte.
const TWO_SR_ATTN_SW_BIT: u8 = 5;

/// External-output enable position in the two-register board's second payload byte.
const TWO_SR_EXT_ON_BIT: u8 = 6;

/// Offset-path relay position in the two-register board's second payload byte.
const TWO_SR_OFFS_SW_BIT: u8 = 7;

/// Square-wave relay position in the three-register board's control byte.
const THREE_SR_SQUARE_SW_BIT: u8 = 0;

/// Attenuator relay position in the three-register board's control byte.
const THREE_SR_ATTN_SW_BIT: u8 = 1;

/// External-output enable position in the three-register board's control byte.
const THREE_SR_EXT_ON_BIT: u8 = 2;

/// Offset-path relay position in the three-register board's control byte.
const THREE_SR_OFFS_SW_BIT: u8 = 3;

/// Logic-level output relay position in the three-register board's control byte.
const THREE_SR_LOGIC_SW_BIT: u8 = 4;

/// Front-panel switch LED position in the LED shift-register image.
const LED_SWITCH_BIT: u8 = 3;

#[cfg(test)]
mod tests {
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
        assert_eq!(DdsHardwareState::dds_tuning_word_integer(10_000), 64_000);
        assert_eq!(
            DdsHardwareState::dds_tuning_word_integer(12_345_670),
            79_012_288
        );
    }

    #[test]
    fn tuning_word_is_split_into_ad9833_frequency_frames() {
        let frames = DdsHardwareState::dds_frequency_frames(0x0123_4567);
        assert_eq!(frames, [0x4567, 0x448d]);
    }

    #[test]
    fn set_level_dds_runs_without_touching_other_state_files() {
        let mut state = DdsHardwareState {
            dac_level: 80.0,
            frequency_tenths_hz: 10_000,
            ..Default::default()
        };
        let mut io = MockIo::default();
        state.set_level_dds(&mut io, Waveform::Sine);
        assert_eq!(state.dds_frequency_word, 64_000);
    }

    #[test]
    fn set_level_dds_sqg_preserves_existing_shift_register_payload() {
        let mut state = DdsHardwareState {
            dac_level: 80.0,
            level_byte_hi: 0x12,
            level_byte_lo: 0x34,
            frequency_tenths_hz: 10_000,
            ..Default::default()
        };
        let mut io = MockIo::default();

        state.set_level_dds_sqg(&mut io, Waveform::Sine);

        assert_eq!(state.switch_state, 1 << TWO_SR_ATTN_SW_BIT);
        assert_eq!(state.level_byte_hi, 0x12 | (1 << TWO_SR_ATTN_SW_BIT));
        assert_eq!(state.level_byte_lo, 0x34);
        assert_eq!(
            io.shift_register_bytes(),
            vec![state.switch_state, state.level_byte_hi, state.level_byte_lo]
        );
    }

    #[test]
    fn set_level_dds_masks_interrupts_around_pascal_dds_writes() {
        let mut state = DdsHardwareState {
            dac_level: 120.0,
            frequency_tenths_hz: 10_000,
            ..Default::default()
        };
        let mut io = MockIo::default();

        state.set_level_dds(&mut io, Waveform::Sine);

        let begin = io.event_index(Event::BeginCriticalSection);
        let end = io.event_index(Event::EndCriticalSection);
        assert!(begin < end);
        assert_eq!(io.count_events(Event::BeginCriticalSection), 1);
        assert_eq!(io.count_events(Event::EndCriticalSection), 1);
        assert_eq!(
            io.count_events(Event::ClearBit(PortKind::DdsOut, B_FSYNC)),
            3
        );
        assert_eq!(io.count_events(Event::SetBit(PortKind::DdsOut, B_FSYNC)), 3);

        for (index, event) in io.events.iter().enumerate() {
            if matches!(
                event,
                Event::ClearBit(PortKind::DdsOut, B_FSYNC)
                    | Event::SetBit(PortKind::DdsOut, B_FSYNC)
            ) {
                assert!(index > begin && index < end);
            }
        }
    }

    #[test]
    fn shift_out_1257_preserves_pascal_clock_hold_nops() {
        let mut state = DdsHardwareState::default();
        let mut io = MockIo::default();

        state.shift_out_1257(&mut io, 0);

        assert_eq!(io.count_events(Event::Nop), 12);

        let load_index = io.event_index(Event::ClearBit(PortKind::ControlBit, B_STRDAC));
        assert_eq!(io.events[load_index + 1], Event::Nop);
        assert_eq!(
            io.events[load_index + 2],
            Event::ClearBit(PortKind::ControlBit, B_SDATAOUT)
        );
        assert_eq!(
            io.events[load_index + 3],
            Event::ClearBit(PortKind::ControlBit, B_SCLK)
        );
        assert_eq!(
            io.events[load_index + 4],
            Event::SetBit(PortKind::ControlBit, B_STRDAC)
        );
    }

    #[test]
    fn shift_out_level_sr_preserves_pascal_strobe_hold_nops() {
        let mut state = DdsHardwareState::default();
        let mut io = MockIo::default();

        state.shift_out_level_sr(&mut io, 0x1234);

        let strobe_index = io.event_index(Event::SetBit(PortKind::DdsOut, B_STROBE));
        assert_eq!(io.events[strobe_index + 1], Event::Nop);
        assert_eq!(io.events[strobe_index + 2], Event::Nop);
        assert_eq!(
            io.events[strobe_index + 3],
            Event::ClearBit(PortKind::DdsOut, B_STROBE)
        );
        assert_eq!(
            io.events[strobe_index + 4],
            Event::SetBit(PortKind::DdsOut, B_SCLK)
        );
    }

    #[test]
    fn ser_aux_preserves_pascal_mp3_serial_bit_timing() {
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

        assert_eq!(delays, [5, 5, 5, 5, 5, 5, 5, 5, 5, 10]);
        assert_eq!(
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
            ]
        );
    }
}
