    use super::*;
    use std::cell::{Cell, RefCell};

    /// Records observable hardware operations so tests can assert exact edge and register order.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Event {
        /// Records a logical ADA signal transition.
        Signal(Signal, bool),

        /// Records a complete ADA Port C write.
        PortC(Byte),

        /// Records a scripted digital-input sample.
        Read(Signal),

        /// Records selection of an internal ADC channel and reference.
        Admux(Byte),

        /// Records starting or configuring an internal ADC conversion.
        AdcsraWrite(Byte),

        /// Records polling the internal ADC completion flag.
        AdcsraRead(Byte),

        /// Records reading the ADC low byte before the high byte.
        AdclRead(Byte),

        /// Records reading the ADC high byte that latches the result.
        AdchRead(Byte),

        /// Records saving interrupt state before a multi-edge transfer.
        BeginInterruptExclusion,

        /// Records restoring interrupt state after a transfer.
        EndInterruptExclusion(Byte),

        /// Records one deliberate hold-time cycle when used as a hardware event.
        Nop,

        /// Records a requested block of peripheral setup or hold cycles.
        WaitCycles(u16),

        /// Records waiting until the internal ADC clears its busy bit.
        WaitForAdc10Complete,
    }

    /// Captures ADA converter signal transitions and scripted ADC inputs for cycle-accurate hardware routine tests.
    #[derive(Debug, Default)]
    struct TestHardware {
        events: RefCell<Vec<Event>>,

        /// Records input bits in occurrence order so tests can assert the complete external interaction.
        input_bits: Vec<bool>,

        /// Tracks input index within the fixed-capacity sequence used by this routine.
        input_index: Cell<usize>,

        next_status: Byte,

        adcsra_reads: Vec<Byte>,

        adcsra_read_index: Cell<usize>,

        /// Contains adcl in converter counts until the owning conversion or output routine consumes it.
        adcl: Byte,

        /// Contains adch in converter counts until the owning conversion or output routine consumes it.
        adch: Byte,
    }

    impl TestHardware {
        /// Constructs a hardware test double preloaded with input word for a deterministic conversion trace.
        fn with_input_word(word: Word) -> Self {
            let input_bits = (0..16).rev().map(|bit| word & (1 << bit) != 0).collect();

            Self {
                events: RefCell::new(Vec::new()),
                input_bits,
                input_index: Cell::new(0),
                next_status: 0xa5,
                adcsra_reads: Vec::new(),
                adcsra_read_index: Cell::new(0),
                adcl: 0,
                adch: 0,
            }
        }

        /// Constructs a hardware test double preloaded with adc10 for a deterministic conversion trace.
        fn with_adc10(adcsra_reads: Vec<Byte>, adcl: Byte, adch: Byte) -> Self {
            Self {
                adcsra_reads,
                adcl,
                adch,
                ..Self::default()
            }
        }

        /// Queries recorded hardware events for count events so timing tests can assert order as well as final values.
        fn count_events(&self, event: Event) -> usize {
            self.events
                .borrow()
                .iter()
                .filter(|candidate| **candidate == event)
                .count()
        }

        /// Queries recorded hardware events for contains event so timing tests can assert order as well as final values.
        fn contains_event(&self, event: Event) -> bool {
            self.events.borrow().contains(&event)
        }

        /// Queries recorded hardware events for first event so timing tests can assert order as well as final values.
        fn first_event(&self) -> Option<Event> {
            self.events.borrow().first().copied()
        }

        /// Queries recorded hardware events for last event so timing tests can assert order as well as final values.
        fn last_event(&self) -> Option<Event> {
            self.events.borrow().last().copied()
        }

        /// Reports whether event window without mutating device state.
        fn has_event_window(&self, window: &[Event]) -> bool {
            self.events
                .borrow()
                .windows(window.len())
                .any(|candidate| candidate == window)
        }
    }

    impl AdacHardware for TestHardware {
        fn set_signal(&mut self, signal: Signal, high: bool) {
            self.events.borrow_mut().push(Event::Signal(signal, high));
        }

        /// Samples signal directly from its mapped input pin during the bit-level peripheral transaction.
        fn read_signal(&self, signal: Signal) -> bool {
            self.events.borrow_mut().push(Event::Read(signal));
            let index = self.input_index.get();
            self.input_index.set(index + 1);
            self.input_bits[index]
        }

        fn set_port_c(&mut self, value: Byte) {
            self.events.borrow_mut().push(Event::PortC(value));
        }

        fn set_admux(&mut self, value: Byte) {
            self.events.borrow_mut().push(Event::Admux(value));
        }

        /// Writes adcsra to the serial, display, or peripheral destination selected by the implementation.
        fn write_adcsra(&mut self, value: Byte) {
            self.events.borrow_mut().push(Event::AdcsraWrite(value));
        }

        /// Reads the AVR adcsra register used to detect completion and assemble the 10-bit conversion.
        fn read_adcsra(&self) -> Byte {
            let index = self.adcsra_read_index.get();
            let value = self.adcsra_reads.get(index).copied().unwrap_or(0);
            self.adcsra_read_index.set(index + 1);
            self.events.borrow_mut().push(Event::AdcsraRead(value));
            value
        }

        /// Reads the AVR adcl register used to detect completion and assemble the 10-bit conversion.
        fn read_adcl(&self) -> Byte {
            self.events.borrow_mut().push(Event::AdclRead(self.adcl));
            self.adcl
        }

        /// Reads the AVR adch register used to detect completion and assemble the 10-bit conversion.
        fn read_adch(&self) -> Byte {
            self.events.borrow_mut().push(Event::AdchRead(self.adch));
            self.adch
        }

        /// Marks the begin interrupt exclusion boundary so interrupt state is saved and restored around the complete transaction.
        fn begin_interrupt_exclusion(&mut self) -> Byte {
            self.events
                .borrow_mut()
                .push(Event::BeginInterruptExclusion);
            self.next_status
        }

        /// Marks the end interrupt exclusion boundary so interrupt state is saved and restored around the complete transaction.
        fn end_interrupt_exclusion(&mut self, saved_status: Byte) {
            self.events
                .borrow_mut()
                .push(Event::EndInterruptExclusion(saved_status));
        }

        /// Provides the nop timing gap required between peripheral signal edges.
        fn nop(&mut self) {
            self.events.borrow_mut().push(Event::Nop);
        }

        /// Waits for cycles so callers cannot consume a stale hardware result.
        fn wait_cycles(&mut self, cycles: u16) {
            self.events.borrow_mut().push(Event::WaitCycles(cycles));
        }

        /// Waits for for adc10 complete so callers cannot consume a stale hardware result.
        fn wait_for_adc10_complete(&mut self) {
            self.events.borrow_mut().push(Event::WaitForAdc10Complete);
        }
    }

    /// Verifies that shift in1864 blocks interrupts for the whole ltc1864 transaction remains faithful to the Pascal behavior.
    #[test]
    fn shift_in1864_blocks_interrupts_for_the_whole_ltc1864_transaction() {
        let mut hw = TestHardware::with_input_word(0xb65a);
        let mut state = AdacState::default();

        shift_in1864(&mut hw, &mut state);

        assert_eq!(state.ad_raw, 0xb65a);
        assert_eq!(hw.first_event(), Some(Event::BeginInterruptExclusion));
        assert_eq!(hw.last_event(), Some(Event::EndInterruptExclusion(0xa5)));
        assert_eq!(hw.count_events(Event::Read(Signal::SDataIn1)), 16);
        assert_eq!(hw.count_events(Event::Nop), 4);
    }

    /// Verifies that DAC shift routines keep pascal nop timing remains faithful to the Pascal behavior.
    #[test]
    fn dac_shift_routines_keep_pascal_nop_timing() {
        let state = AdacState {
            dac_temp: 0xa55a,
            ..AdacState::default()
        };

        let mut hw = TestHardware::default();
        shift_out1257(&mut hw, &state);
        assert_eq!(hw.count_events(Event::Nop), 12);

        let mut hw = TestHardware::default();
        shift_out1655(&mut hw, &state);
        assert_eq!(hw.count_events(Event::Nop), 8);

        let mut hw = TestHardware::default();
        shift_out714(&mut hw, &state);
        assert_eq!(hw.count_events(Event::Nop), 19);
    }

    /// Verifies that shift register strobe keeps pascal nop timing remains faithful to the Pascal behavior.
    #[test]
    fn shift_register_strobe_keeps_pascal_nop_timing() {
        let state = AdacState {
            port_sr0: 0x11,
            port_sr1: 0x22,
            port_sr2: 0x44,
            port_sr3: 0x88,
            ..AdacState::default()
        };
        let mut hw = TestHardware::default();

        shift_out_sr(&mut hw, &state);

        assert_eq!(hw.count_events(Event::Nop), 2);
        assert!(hw.has_event_window(&[
            Event::Signal(Signal::StrSr, true),
            Event::Nop,
            Event::Nop,
            Event::Signal(Signal::StrSr, false),
        ]));
    }

    /// Verifies that on sys tick uses pascal delay loop cycle counts remains faithful to the Pascal behavior.
    #[test]
    fn on_sys_tick_uses_pascal_delay_loop_cycle_counts() {
        let mut hw = TestHardware::with_input_word(0x8004);
        hw.input_bits
            .extend((0..16).rev().map(|bit| 0x8008 & (1 << bit) != 0));
        hw.input_bits
            .extend((0..16).rev().map(|bit| 0x800c & (1 << bit) != 0));
        hw.input_bits
            .extend((0..16).rev().map(|bit| 0x8010 & (1 << bit) != 0));

        let mut state = AdacState {
            adc16_present: true,
            mux_ch: 7,
            ..AdacState::default()
        };

        on_sys_tick(&mut hw, &mut state);

        assert!(hw.contains_event(Event::WaitCycles(ADC16_DISCARD_CONVERSION_DELAY_CYCLES)));
        assert!(hw.contains_event(Event::WaitCycles(DAC_SETTLE_DELAY_CYCLES)));
        assert!(!hw.contains_event(Event::WaitCycles(15)));
        assert!(!hw.contains_event(Event::WaitCycles(4)));
    }

    /// Verifies that get adc10 matches pascal register sequence remains faithful to the Pascal behavior.
    #[test]
    fn get_adc10_matches_pascal_register_sequence() {
        let mut hw =
            TestHardware::with_adc10(vec![ADCSRA_BUSY_BIT, ADCSRA_BUSY_BIT, 0], 0x34, 0x12);

        let result = get_adc10(&mut hw, 5, true);

        assert_eq!(result, 0x1234);
        assert_eq!(
            *hw.events.borrow(),
            vec![
                Event::Admux(ADC10_INTERNAL_REFERENCE_MASK | 4),
                Event::WaitCycles(ADC10_SETTLE_DELAY_CYCLES),
                Event::AdcsraWrite(ADCSRA_START_DIV128),
                Event::AdcsraRead(ADCSRA_BUSY_BIT),
                Event::AdcsraRead(ADCSRA_BUSY_BIT),
                Event::AdcsraRead(0),
                Event::AdclRead(0x34),
                Event::AdchRead(0x12),
            ]
        );
    }

    /// Verifies that get adc10 wraps and masks pascal byte channel remains faithful to the Pascal behavior.
    #[test]
    fn get_adc10_wraps_and_masks_pascal_byte_channel() {
        let mut hw = TestHardware::with_adc10(vec![0], 0, 0);

        let result = get_adc10(&mut hw, 0, false);

        assert_eq!(result, 0);
        assert_eq!(
            hw.events.borrow().first(),
            Some(&Event::Admux(ADC10_CHANNEL_MASK))
        );
    }
