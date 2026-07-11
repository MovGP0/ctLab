    use super::*;
    use std::collections::VecDeque;

    /// Verifies that command and error enums own the exact wire text without positional string tables.
    #[test]
    fn enum_text_mappings_preserve_wire_protocol() {
        assert_eq!(CmdWhich::from_mnemonic("  iCa  "), CmdWhich::Ica);
        assert_eq!(CmdWhich::Ica.as_str(), Some("ICA"));
        assert_eq!(CmdWhich::from_mnemonic("unknown"), CmdWhich::Err);
        assert_eq!(CmdWhich::Err.as_str(), None);
        assert_eq!(ErrorCode::LockedErr.as_str(), "[LOCKED]");
    }

    /// Emulates detected ADA daughterboards, conversion inputs, and serial output so startup and trigger flows can be tested without AVR registers.
    #[derive(Debug, Clone)]
    struct MockHardware {
        /// Contains adc values in converter counts until scaling or hardware output consumes it.
        adc_values: [i16; 8],

        io_values: [u8; 8],

        /// Controls the ADA host model's response to I2C expander discovery.
        detect_io_expander: bool,

        /// Records startup detection of has dac12 and gates commands that require that board feature.
        has_dac12: bool,

        /// Records startup detection of has dac714 and gates commands that require that board feature.
        has_dac714: bool,

        /// Records startup detection of has adc16 and gates commands that require that board feature.
        has_adc16: bool,

        /// Stores the address read from board straps and used to accept or prefix serial frames.
        slave_channel: u8,

        serial_in: VecDeque<u8>,

        serial_out: Vec<String>,

        twi_writes: Vec<(u8, u16)>,

        shift_register_writes: Vec<[u8; 8]>,

        dir_writes: Vec<(u8, u8)>,

        twi_byte_reads: VecDeque<u8>,

        twi_word_reads: VecDeque<u16>,

        /// Records the UBRR and double-speed pair applied by ADA startup.
        baud: Option<(u8, bool)>,

        /// Records whether ADA selected the AVR internal ADC reference.
        internal_reference: Option<bool>,

        /// Latches trigger edge positive with the corresponding ADC sample for status and protection reporting.
        trigger_edge_positive: Option<bool>,

        /// Records that ADA startup reached the point where global interrupts are enabled.
        interrupts_enabled: bool,

        trigger_led_events: Vec<bool>,

        sdataout_high: bool,

        str_dac_high: bool,

        str_ad16_high: bool,
    }

    impl Default for MockHardware {
        fn default() -> Self {
            Self {
                adc_values: [0; 8],
                io_values: [0; 8],
                detect_io_expander: false,
                has_dac12: false,
                has_dac714: false,
                has_adc16: false,
                slave_channel: 0,
                serial_in: VecDeque::new(),
                serial_out: Vec::new(),
                twi_writes: Vec::new(),
                shift_register_writes: Vec::new(),
                dir_writes: Vec::new(),
                twi_byte_reads: VecDeque::new(),
                twi_word_reads: VecDeque::new(),
                baud: None,
                internal_reference: None,
                trigger_edge_positive: None,
                interrupts_enabled: false,
                trigger_led_events: Vec::new(),
                sdataout_high: true,
                str_dac_high: true,
                str_ad16_high: true,
            }
        }
    }

    impl MockHardware {
        /// Queues serial for the next bounded consumer without changing unrelated state.
        fn push_serial(&mut self, text: &str) {
            self.serial_in.extend(text.bytes());
        }
    }

    impl AdaHardware for MockHardware {
        fn get_adc(&mut self, channel_1_based: u8) -> i16 {
            self.adc_values[(channel_1_based - 1) as usize]
        }

        /// Transfers TWI out using the byte order expected by the attached peripheral.
        fn twi_out(&mut self, slave_addr: u8, command: u16) -> bool {
            self.twi_writes.push((slave_addr, command));
            true
        }

        fn shift_out_sr(&mut self, port_array: &[u8; 8]) {
            self.shift_register_writes.push(*port_array);
        }

        /// Returns io pin from the selected local port or I2C expander cache.
        fn read_io_pin(&mut self, port: u8) -> u8 {
            self.io_values[port as usize]
        }

        /// Writes io dir to the serial, display, or peripheral destination selected by the implementation.
        fn write_io_dir(&mut self, port: u8, value: u8) {
            self.dir_writes.push((port, value));
        }

        /// Configures I2C expander before code that relies on that hardware capability runs.
        fn detect_i2c_expander(&mut self) -> bool {
            self.detect_io_expander
        }

        /// Configures sense before code that relies on that hardware capability runs.
        fn detect_sense(&mut self) -> bool {
            if !self.str_ad16_high {
                !self.has_adc16
            } else if !self.str_dac_high {
                !self.has_dac714
            } else if !self.sdataout_high {
                !self.has_dac12
            } else {
                true
            }
        }

        /// Reads the address strap pins once so serial routing uses the instrument's physical channel number.
        fn read_slave_channel(&mut self) -> u8 {
            self.slave_channel
        }

        fn set_external_trigger_edge(&mut self, positive: bool) {
            self.trigger_edge_positive = Some(positive);
        }

        /// Configures interrupts before code that relies on that hardware capability runs.
        fn enable_interrupts(&mut self) {
            self.interrupts_enabled = true;
        }

        /// Encodes TWI inp byte in the compact representation consumed by registers or the serial protocol.
        fn twi_inp_byte(&mut self, _slave_addr: u8) -> u8 {
            self.twi_byte_reads.pop_front().unwrap_or_default()
        }

        /// Transfers TWI inp word using the byte order expected by the attached peripheral.
        fn twi_inp_word(&mut self, _slave_addr: u8) -> u16 {
            self.twi_word_reads.pop_front().unwrap_or_default()
        }

        /// Polls the serial receiver for one byte while respecting the caller's timeout.
        fn serial_read_byte_timeout(&mut self, _timeout_ms: u16) -> Option<u8> {
            self.serial_in.pop_front()
        }

        /// Appends text to the active serial frame without changing parser state.
        fn serial_write(&mut self, text: &str) {
            self.serial_out.push(text.to_string());
        }

        fn set_serial_baud(&mut self, ubrr: u8, double_speed: bool) {
            self.baud = Some((ubrr, double_speed));
        }

        fn set_internal_reference(&mut self, internal: bool) {
            self.internal_reference = Some(internal);
        }

        fn set_sdataout(&mut self, high: bool) {
            self.sdataout_high = high;
        }

        fn set_str_dac(&mut self, high: bool) {
            self.str_dac_high = high;
        }

        fn set_str_ad16(&mut self, high: bool) {
            self.str_ad16_high = high;
        }

        fn set_trigger_led(&mut self, active: bool) {
            self.trigger_led_events.push(active);
        }
    }

    /// Verifies that init all restores detection and startup settings remains faithful to the Pascal behavior.
    #[test]
    fn init_all_restores_detection_and_startup_settings() {
        let hw = MockHardware {
            detect_io_expander: true,
            has_dac12: true,
            has_dac714: true,
            has_adc16: true,
            slave_channel: 2,
            ..MockHardware::default()
        };
        let mut state = DeviceState::new(hw);
        state.eeprom.ee_ser_baud_reg = 115;
        state.eeprom.ext_ref = 1;
        state.eeprom.trig_level = 1;
        state.eeprom.dir_init_array[3] = 0xAA;
        state.eeprom.port_init_array[3] = 0x55;

        let banner = state.init_all();

        assert!(state.io_present);
        assert!(state.dac16_present);
        assert!(!state.dac12_present);
        assert!(!state.dac714_present);
        assert!(state.adc16_present);
        assert_eq!(state.hw.baud, Some((115, true)));
        assert_eq!(state.hw.internal_reference, Some(true));
        assert_eq!(state.hw.trigger_edge_positive, Some(true));
        assert!(state.hw.interrupts_enabled);
        assert!(banner[0].contains("#2:254=1.742 [ADA by CM/c't 04/2007; [DA16 AD16 IO32 ]"));
        assert!(state.hw.twi_writes.contains(&(0x3b, 0x0155)));
        assert!(state.hw.twi_writes.contains(&(0x3b, 0x0200)));
    }

    /// Verifies that serial loop restores control subchannels and backspace handling remains faithful to the Pascal behavior.
    #[test]
    fn serial_loop_restores_control_subchannels_and_backspace_handling() {
        let mut hw = MockHardware {
            slave_channel: 0,
            ..MockHardware::default()
        };
        hw.push_serial("0:WEZ\u{8}N=1\r");
        hw.push_serial("0:OPT9=7?\r");
        hw.push_serial("0:WEN=1\r");
        hw.push_serial("0:TRM0=129?\r");
        hw.push_serial("0:WEN=1\r");
        hw.push_serial("0:TRT=10?\r");
        hw.push_serial("0:WEN=1\r");
        hw.push_serial("0:TRL=1?\r");
        hw.push_serial("0:WEN=1\r");
        hw.push_serial("0:REF=1?\r");
        hw.push_serial("0:WEN=1\r");
        hw.push_serial("0:SBD=115?\r");
        hw.push_serial("0:OPT9?\r");
        hw.push_serial("0:TRM0?\r");
        hw.push_serial("0:TRT?\r");
        hw.push_serial("0:TRL?\r");
        hw.push_serial("0:REF?\r");
        hw.push_serial("0:SBD?\r");
        hw.push_serial("#7:1=foreign\r");
        hw.push_serial("1:VAL0?\r");

        let mut state = DeviceState::new(hw);
        state.init_all();
        state.check_ser();

        assert_eq!(state.inc_rast, 7);
        assert_eq!(state.eeprom.inc_rast_def, 7);
        assert_eq!(state.eeprom.trig_mask_array[0], 129);
        assert_eq!(state.eeprom.trig_timer_value, 10);
        assert_eq!(state.eeprom.trig_level, 1);
        assert_eq!(state.eeprom.ext_ref, 1);
        assert_eq!(state.eeprom.ee_ser_baud_reg, 115);
        assert_eq!(state.hw.trigger_edge_positive, Some(true));
        assert_eq!(state.hw.internal_reference, Some(true));
        assert_eq!(state.hw.baud, Some((51, true)));
        assert!(state
            .hw
            .serial_out
            .iter()
            .any(|line| line.contains("#0:159=7\r\n")));
        assert!(state
            .hw
            .serial_out
            .iter()
            .any(|line| line.contains("#0:240=129\r\n")));
        assert!(state
            .hw
            .serial_out
            .iter()
            .any(|line| line.contains("#0:247=10\r\n")));
        assert!(state
            .hw
            .serial_out
            .iter()
            .any(|line| line.contains("#0:248=1\r\n")));
        assert!(state
            .hw
            .serial_out
            .iter()
            .any(|line| line.contains("#0:246=1\r\n")));
        assert!(state
            .hw
            .serial_out
            .iter()
            .any(|line| line.contains("#0:252=115\r\n")));
        assert!(state
            .hw
            .serial_out
            .iter()
            .any(|line| line == "#7:1=foreign\r\n"));
        assert!(state.hw.serial_out.iter().any(|line| line == "1:VAL0?\r\n"));
    }

    /// Verifies that auto trigger scheduler and side effects are restored remains faithful to the Pascal behavior.
    #[test]
    fn auto_trigger_scheduler_and_side_effects_are_restored() {
        let hw = MockHardware {
            adc_values: [10, 20, 30, 40, 50, 60, 70, 80],
            io_values: [0, 1, 2, 3, 4, 5, 6, 0xAA],
            detect_io_expander: true,
            slave_channel: 0,
            ..MockHardware::default()
        };
        let mut state = DeviceState::new(hw);
        state.init_all();
        state.adc_raw_array[7] = 160;
        state.eeprom.trig_mask_array[0] = 0x80;
        state.eeprom.trig_mask_array[1] = 0x80;
        state.eeprom.trig_mask_array[3] = 0x80;
        state.eeprom.trig_timer_value = 10;

        let first = state.service_auto_trigger(0);
        assert_eq!(first.len(), 3);
        assert!(first[0].contains("#0:7="));
        assert!(first[1].contains("#0:17="));
        assert_eq!(first[2], "#0:37=170\r\n");
        assert_eq!(state.hw.trigger_led_events.last(), Some(&true));

        let second = state.service_auto_trigger(10);
        assert_eq!(second.len(), 3);
        assert!(second[0].contains("#0:7="));

        state.eeprom.trig_timer_value = 0;
        let none = state.service_auto_trigger(60);
        assert!(none.is_empty());
        assert_eq!(state.hw.trigger_led_events.last(), Some(&false));
    }
