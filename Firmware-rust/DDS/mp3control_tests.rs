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
