//! Best-effort Rust port of `mp3control.pas`.

/// Fixes the auxiliary YAMPP frame at eight LSB-first data bits between its start and idle intervals.
const SER_AUX_DATA_BITS: u8 = 8;

/// Five micro-delay units form one 19.2-kbaud YAMPP data-bit interval on the auxiliary transmit line.
const SER_AUX_BIT_DELAY_TICKS: u8 = 5;

/// Ten micro-delay units hold the YAMPP line idle high for two bit periods after each byte.
const SER_AUX_STOP_DELAY_TICKS: u8 = 10;

#[path = "mp3control/mp3_control_hardware.rs"]
mod mp3_control_hardware;
pub use mp3_control_hardware::Mp3ControlHardware;

#[path = "mp3control/mp3_control_state.rs"]
mod mp3_control_state;
pub use mp3_control_state::Mp3ControlState;

#[path = "mp3control/yi3_command.rs"]
mod yi3_command;
pub use yi3_command::Yi3Command;

/// Bit-bangs the auxiliary UART with the edge spacing and idle level expected by the legacy MP3 controller.
pub fn ser_aux<H: Mp3ControlHardware>(hw: &mut H, value: u8) {
    let mut current = value;
    let mut bits_remaining = SER_AUX_DATA_BITS;

    // Pascal bit-bangs 19.2 kBd on SerAux: start bit, 8 data bits LSB first.
    hw.set_ser_aux(false);
    hw.micro_delay(SER_AUX_BIT_DELAY_TICKS);

    while bits_remaining > 0 {
        let data_high = current & 0x01 != 0;
        current >>= 1;

        hw.set_ser_aux(data_high);
        hw.micro_delay(SER_AUX_BIT_DELAY_TICKS);
        bits_remaining -= 1;
    }

    // Return the line to idle high and keep the Pascal routine's two-bit idle gap.
    hw.set_ser_aux(true);
    hw.micro_delay(SER_AUX_STOP_DELAY_TICKS);
}

/// Waits for the decoder command interval, then adds the board's dB correction to the YAMPP mid-volume command.
pub fn mp3_set_volume<H: Mp3ControlHardware>(state: &Mp3ControlState, hw: &mut H) {
    hw.milli_delay(20);
    ser_aux(
        hw,
        Yi3Command::MidVolume
            .byte()
            .wrapping_add(state.db_correction),
    );
}

/// Sends the new track and then refreshes volume because the decoder resets its attenuation when changing tracks.
pub fn mp3_goto_track<H: Mp3ControlHardware>(state: &mut Mp3ControlState, hw: &mut H) {
    // Track numbers are sent directly as single-byte player commands.
    ser_aux(hw, state.track);
    state.current_track = state.track;
    // Re-apply the calibrated level after changing tracks.
    mp3_set_volume(state, hw);
}

/// Disables decoder-side looping, restores the corrected volume, stops stale playback, and finally advertises the powered state through the shared shift register.
pub fn mp3_on<H: Mp3ControlHardware>(state: &mut Mp3ControlState, hw: &mut H) {
    // Disable the player's internal repeat mode; the firmware handles repeats itself.
    ser_aux(hw, Yi3Command::NoLoop.byte());
    ser_aux(
        hw,
        Yi3Command::MidVolume
            .byte()
            .wrapping_add(state.db_correction),
    );
    // Stop first so playback always starts from a known state.
    ser_aux(hw, Yi3Command::Stop.byte());
    hw.milli_delay(100);
    state.current_track = 0;
    state.is_on = true;
    // Propagate the power-state change to the shared shift register outputs.
    hw.send_shift_register();
}

/// Disables looping, mutes before stop for a silent shutdown, then clears the power and current-track shadows before latching them.
pub fn mp3_off<H: Mp3ControlHardware>(state: &mut Mp3ControlState, hw: &mut H) {
    ser_aux(hw, Yi3Command::NoLoop.byte());
    // Mute before stopping so power-down is silent.
    ser_aux(hw, Yi3Command::Mute.byte());
    ser_aux(hw, Yi3Command::Stop.byte());
    state.is_on = false;
    state.current_track = 0;
    hw.send_shift_register();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Event {
        SerAux(bool),
        MicroDelay(u8),
        MilliDelay(u16),
        SendShiftRegister,
    }

    #[derive(Debug, Default)]
    struct MockHardware {
        events: Vec<Event>,
    }

    impl Mp3ControlHardware for MockHardware {
        fn set_ser_aux(&mut self, high: bool) {
            self.events.push(Event::SerAux(high));
        }

        fn micro_delay(&mut self, ticks: u8) {
            self.events.push(Event::MicroDelay(ticks));
        }

        fn milli_delay(&mut self, ticks: u16) {
            self.events.push(Event::MilliDelay(ticks));
        }

        fn send_shift_register(&mut self) {
            self.events.push(Event::SendShiftRegister);
        }
    }

    #[test]
    fn ser_aux_preserves_pascal_uart_edges_and_delays() {
        let mut hardware = MockHardware::default();

        ser_aux(&mut hardware, 0b1010_0101);

        assert_eq!(
            hardware.events,
            vec![
                Event::SerAux(false),
                Event::MicroDelay(5),
                Event::SerAux(true),
                Event::MicroDelay(5),
                Event::SerAux(false),
                Event::MicroDelay(5),
                Event::SerAux(true),
                Event::MicroDelay(5),
                Event::SerAux(false),
                Event::MicroDelay(5),
                Event::SerAux(false),
                Event::MicroDelay(5),
                Event::SerAux(true),
                Event::MicroDelay(5),
                Event::SerAux(false),
                Event::MicroDelay(5),
                Event::SerAux(true),
                Event::MicroDelay(5),
                Event::SerAux(true),
                Event::MicroDelay(10),
            ]
        );
    }

    #[test]
    fn mp3_goto_track_sends_track_then_pascal_volume_refresh() {
        let mut state = Mp3ControlState {
            track: 7,
            db_correction: 3,
            ..Mp3ControlState::default()
        };
        let mut hardware = MockHardware::default();

        mp3_goto_track(&mut state, &mut hardware);

        assert_eq!(state.current_track, 7);
        assert_eq!(
            hardware
                .events
                .iter()
                .filter(|event| matches!(event, Event::MilliDelay(20)))
                .count(),
            1
        );
        assert!(hardware.events.starts_with(&[
            Event::SerAux(false),
            Event::MicroDelay(5),
            Event::SerAux(true),
            Event::MicroDelay(5),
            Event::SerAux(true),
            Event::MicroDelay(5),
            Event::SerAux(true),
            Event::MicroDelay(5),
            Event::SerAux(false),
            Event::MicroDelay(5),
        ]));
    }
}
