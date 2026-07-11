    use super::*;

    #[derive(Debug, Default, Clone)]
    struct MockHardware {
        adc10: i16,
        adc_u: u16,
        adc_i: u16,
        serial_in: Vec<char>,
        voltage_on: Vec<u16>,
        voltage_off: Vec<u16>,
        current: Vec<u16>,
        current_ranges: Vec<CurrentRange>,
        voltage_ranges: Vec<VoltageRange>,
        input_relays: Vec<bool>,
        current_limit_sense: bool,
        temp_c: Option<Float>,
        outputs: Vec<bool>,
        delays: Vec<u16>,
        serial: String,
        lcd: Vec<(u8, String)>,
    }

    impl DcgHardware for MockHardware {
        fn read_adc10(&mut self, _channel_1_based: u8) -> i16 {
            self.adc10
        }

        fn read_adc16_voltage(&mut self) -> u16 {
            self.adc_u
        }

        fn read_adc16_current(&mut self) -> u16 {
            self.adc_i
        }

        fn serial_read_timeout(&mut self, _timeout_ms: u16) -> Option<char> {
            if self.serial_in.is_empty() {
                None
            } else {
                Some(self.serial_in.remove(0))
            }
        }

        fn set_voltage_dac_raw(&mut self, raw: u16) {
            self.voltage_on.push(raw);
        }

        fn set_current_dac_raw(&mut self, raw: u16) {
            self.current.push(raw);
        }

        fn set_voltage_dac_off_raw(&mut self, raw: u16) {
            self.voltage_off.push(raw);
        }

        fn delay_ms(&mut self, milliseconds: u16) {
            self.delays.push(milliseconds);
        }

        fn set_current_range(&mut self, range: CurrentRange) {
            self.current_ranges.push(range);
        }

        fn set_voltage_range(&mut self, range: VoltageRange) {
            self.voltage_ranges.push(range);
        }

        fn set_input_relay_high(&mut self, high: bool) {
            self.input_relays.push(high);
        }

        fn current_limit_sense(&mut self) -> bool {
            self.current_limit_sense
        }

        fn set_output_enabled(&mut self, enabled: bool) {
            self.outputs.push(enabled);
        }

        fn read_temp_c(&mut self) -> Option<Float> {
            self.temp_c
        }

        fn serial_write(&mut self, text: &str) {
            self.serial.push_str(text);
        }

        fn lcd_write_line(&mut self, row: u8, text: &str) {
            self.lcd.push((row, text.to_string()));
        }
    }

    fn test_eeprom() -> EepromData {
        let mut eeprom = EepromData {
            dac_u_offsets: [10, 20],
            dac_i_offsets: [1, 2, 3, 4],
            ..EepromData::default()
        };
        eeprom.option_array[OptionSlot::PreamplifierGain.index()] = 1.0;
        eeprom.option_array[OptionSlot::OutputStageGain.index()] = 1.0;
        eeprom.option_array[OptionSlot::CurrentMeasurementGain.index()] = 1.0;
        eeprom.option_array[OptionSlot::ReferenceVoltage.index()] = 1.0;
        eeprom.option_array[OptionSlot::VoltageRangeSwitchpoint.index()] = 2.0;
        eeprom.option_array[OptionSlot::InitialRipplePercent.index()] = 25.0;
        eeprom.option_array[OptionSlot::InitialRippleOnTime.index()] = 4.0;
        eeprom.option_array[OptionSlot::InitialRippleOffTime.index()] = 6.0;
        let sense_resistance = OptionSlot::SenseResistance2mA.index();
        eeprom.option_array[sense_resistance..sense_resistance + 4]
            .copy_from_slice(&[1000.0, 100.0, 10.0, 1.0]);
        let maximum_current = OptionSlot::MaximumCurrent2mA.index();
        eeprom.option_array[maximum_current..maximum_current + 4]
            .copy_from_slice(&[0.002, 0.020, 0.200, 2.000]);
        eeprom
    }

    fn mock_hardware() -> MockHardware {
        MockHardware {
            current_limit_sense: true,
            ..MockHardware::default()
        }
    }

    #[test]
    fn default_eeprom_matches_pascal_ada16_option_layout() {
        let eeprom = EepromData::default();
        assert_eq!(eeprom.option_array[OptionSlot::InitialVoltage.index()], 5.0);
        assert_eq!(eeprom.option_array[OptionSlot::InitialCurrent.index()], 0.02);
        assert_eq!(eeprom.option_array[OptionSlot::CurrentMeasurementGain.index()], 0.25);
        assert_eq!(eeprom.option_array[OptionSlot::ReferenceVoltage.index()], 2.5);
        assert_eq!(eeprom.option_array[OptionSlot::MaximumVoltage.index()], 30.0);
        assert_eq!(eeprom.option_array[OptionSlot::SenseResistance2mA.index()], 470.0);
        assert_eq!(eeprom.option_array[OptionSlot::SenseResistance2A.index()], 0.47);
        assert_eq!(eeprom.option_array[OptionSlot::MaximumCurrent2mA.index()], 0.002);
        assert_eq!(eeprom.option_array[OptionSlot::MaximumCurrent2A.index()], 2.0);
        assert_eq!(eeprom.option_array[OptionSlot::LowVoltageAdcDivider.index()], 2.0);
        assert_eq!(eeprom.option_array[OptionSlot::HighVoltageAdcDivider.index()], 6.0);
        assert_eq!(eeprom.option_array[OptionSlot::InstalledHardware.index()], 7.0);
        assert_eq!(eeprom.option_array[OptionSlot::VoltageRangeSwitchpoint.index()], 12.1);
        assert_eq!(eeprom.option_array[OptionSlot::RelayHysteresisLow.index()], 8.6);
        assert_eq!(eeprom.option_array[OptionSlot::RelayHysteresisHigh.index()], 8.9);
        assert_eq!(eeprom.option_array[OptionSlot::FanOnTemperature.index()], 50.0);
        assert_eq!(eeprom.option_array[OptionSlot::InitialRippleOnTime.index()], 4.0);
        assert_eq!(eeprom.option_array[OptionSlot::InitialRippleOffTime.index()], 6.0);
    }

    #[test]
    fn init_scales_derives_calibration_and_startup_state() {
        let hw = mock_hardware();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_scales();

        assert!(state.scale.dac16_present);
        assert!(state.scale.adc16_present);
        assert!(state.scale.dcp_present);
        assert_eq!(state.scale.dac_max, 65_535);
        assert_eq!(state.scale.switchpoint, 2.0);
        assert_eq!(state.relay_voltage_low, 8.6);
        assert_eq!(state.relay_voltage_high, 8.9);
        assert_eq!(state.ripple_percent, 25.0);
        assert_eq!(state.pw_on_time_ms, 4);
        assert_eq!(state.pw_off_time_ms, 6);
        assert_eq!(state.pw_counter_ms, 4);
    }

    #[test]
    fn status_frame_uses_pascal_error_channel_payload() {
        let hw = mock_hardware();
        let mut state = DeviceState::new(hw);
        state.main_channel = 3;
        state.status.ee_unlocked = true;
        state.status.overload_flag = true;

        let frame = state.status_frame(ErrorCode::ParamErr);

        assert_eq!(frame, "3:255=53 [PARERR] [ICONST]\r\n");
        assert_eq!(state.err_count, 1);
    }

    #[test]
    fn status_frame_reports_fault_labels_in_low_nibble() {
        let hw = mock_hardware();
        let mut state = DeviceState::new(hw);
        state.faults = FaultFlags {
            over_power: true,
            fuse_blown: false,
            over_voltage: true,
            over_temp: false,
        };

        let frame = state.status_frame(ErrorCode::NoErr);

        assert_eq!(frame, "0:255=5 [OVRPOWR] [OVRVOLT]\r\n");
    }

    #[test]
    fn set_level_dac_blanks_changed_ranges_then_applies_offsets_and_ripple_off_level() {
        let hw = mock_hardware();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_scales();
        state.voltage_set = 3.0;
        state.current_set = 0.25;
        state.set_level_dac();

        assert_eq!(state.current_range, CurrentRange::Dc2A);
        assert_eq!(state.voltage_range, VoltageRange::UHigh);
        assert_eq!(state.hw.current.first(), Some(&0));
        assert_eq!(state.hw.voltage_on.first(), Some(&0));
        assert_eq!(state.hw.voltage_off.first(), Some(&0));
        assert_eq!(state.hw.current_ranges, vec![CurrentRange::Dc2A]);
        assert_eq!(state.hw.voltage_ranges, vec![VoltageRange::UHigh]);
        assert_eq!(state.hw.delays, vec![4, 4]);
        assert_eq!(state.dac_raw_i, 8_221);
        assert_eq!(state.dac_raw_u_on, 65_535);
        assert_eq!(state.dac_raw_u_off, 49_151);
        assert_eq!(state.hw.current.last(), Some(&8_221));
        assert_eq!(state.hw.voltage_on.last(), Some(&65_535));
        assert_eq!(state.hw.voltage_off.last(), Some(&49_151));
    }

    #[test]
    fn encoder_delta_waits_for_rast_then_rounds_and_applies_voltage_acceleration() {
        let mut state = DeviceState::with_eeprom(mock_hardware(), test_eeprom());
        state.init_scales();
        state.panel_modify = Modify::Volt;
        state.voltage_set = 1.26;

        assert!(!state.apply_encoder_delta(3));
        assert_eq!(state.voltage_set, 1.26);
        assert!(state.apply_encoder_delta(1));

        assert!((state.voltage_set - 1.4).abs() < 0.000001);
        assert!(!state.first_turn);
        assert_eq!(state.button_number, 4);
        assert!(state.hw.serial.contains("0:255=68 [SRQUSR]\r\n"));
        assert!(state.hw.serial.contains("0:0=1.400\r\n"));
    }

    #[test]
    fn encoder_delta_uses_pascal_acceleration_table_for_fast_voltage_turns() {
        let mut state = DeviceState::with_eeprom(mock_hardware(), test_eeprom());
        state.init_scales();
        state.panel_modify = Modify::Volt;
        state.voltage_set = 1.23;

        assert!(state.apply_encoder_delta(12));

        assert!((state.voltage_set - 1.7).abs() < 0.000001);
        assert_eq!(state.incr_acc_float, 5.0);
    }

    #[test]
    fn fine_current_encoder_step_uses_current_divisor_without_coarse_rounding() {
        let mut state = DeviceState::with_eeprom(mock_hardware(), test_eeprom());
        state.init_scales();
        state.panel_modify = Modify::Ampere;
        state.incr_fine = true;
        state.current_set = 0.126;

        assert!(state.apply_encoder_delta(4));

        assert!((state.current_set - 0.1261).abs() < 0.000001);
        assert_eq!(state.inc_fine_div, 10_000.0);
        assert!(state.first_turn);
    }

    #[test]
    fn new_state_disables_tracking_like_pascal_eeprom_default() {
        let state = DeviceState::new(mock_hardware());

        assert_eq!(state.track_channel, 255);
    }

    #[test]
    fn measurement_conversion_uses_pascal_adc_paths_and_input_scaling() {
        let mut eeprom = test_eeprom();
        eeprom.option_array[OptionSlot::InstalledHardware.index()] = 0.0;
        eeprom.adc_u_offsets = [3, 30];
        eeprom.adc_i_offsets = [4, 40, 400, 4000];
        eeprom.adc_u_scales = [1.0, 1.0];
        eeprom.adc_i_scales = [1.0, 1.0, 1.0, 1.0];
        let hw = MockHardware {
            adc10: 100,
            current_limit_sense: true,
            ..MockHardware::default()
        };
        let mut state = DeviceState::with_eeprom(hw, eeprom);
        state.init_scales();
        state.voltage_range = VoltageRange::ULow;
        state.current_range = CurrentRange::Dc2mA;

        let voltage = state.get_voltage();
        let current = state.get_current();
        state.get_input_voltage();

        assert!((voltage - 0.20117188).abs() < 0.000001);
        assert!((current - 0.00005078).abs() < 0.000001);
        assert!((state.input_voltage - 1.855).abs() < 0.0001);
    }

    #[test]
    fn check_limits_clamps_and_normalizes_ripple_and_tracking() {
        let mut state = DeviceState::with_eeprom(mock_hardware(), test_eeprom());
        state.voltage_set = 40.0;
        state.current_set = 3.0;
        state.pw_on_time_ms = 1;
        state.ripple_percent = 125.0;
        state.track_channel = 9;

        let err = state.check_limits();

        assert_eq!(err, ErrorCode::ParamErr);
        assert_eq!(state.voltage_set, 30.0);
        assert_eq!(state.current_set, 2.0);
        assert_eq!(state.pw_on_time_ms, 2);
        assert_eq!(state.ripple_percent, 100.0);
        assert_eq!(state.track_channel, 7);
        assert_eq!(state.ripple_voltage, 30.0);
        assert!(!state.no_toggle);

        state.track_channel = 128;
        assert_eq!(state.check_limits(), ErrorCode::NoErr);
        assert_eq!(state.track_channel, 255);
    }

    #[test]
    fn tracking_transmit_path_sends_voltage_and_current_commands() {
        let mut state = DeviceState::new(mock_hardware());
        state.track_channel = 4;
        state.voltage_set = 1.25;
        state.current_set = 0.5;

        state.send_track_cmd();

        assert_eq!(state.hw.serial, "4:0=1.250!\r\n4:1=0.500!\r\n");
    }

    #[test]
    fn fault_check_drops_relays_for_overtemp_overvoltage_and_fuse_loss() {
        let hw = MockHardware {
            adc10: 100,
            temp_c: Some(71.0),
            current_limit_sense: true,
            ..MockHardware::default()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_scales();
        state.measured_voltage = 10.0;

        state.fault_check();

        assert!(state.faults.over_temp);
        assert!(!state.faults.over_voltage);
        assert!(state.faults.fuse_blown);
        assert!(state.status.overload_flag);
        assert_eq!(state.hw.input_relays, vec![false, false]);
    }

    #[test]
    fn chores_updates_measurements_overload_and_relay_hysteresis() {
        let hw = MockHardware {
            adc_u: 10_000,
            adc_i: 10_000,
            adc10: 1023,
            current_limit_sense: false,
            ..mock_hardware()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_scales();
        state.voltage_set = 10.0;
        state.old_relay_state_high = false;
        state.relay_voltage_low = 0.0;
        state.relay_voltage_high = 0.01;

        state.chores();

        assert!(state.status.overload_flag);
        assert!(state.measured_voltage > 0.0);
        assert!(state.measured_current > 0.0);
        assert_eq!(state.hw.input_relays, vec![true]);
    }

    #[test]
    fn check_ser_drains_ascii_backspace_and_cr_commands() {
        let mut hw = mock_hardware();
        hw.serial_in = "0:0=12.x\u{8}3!\r".chars().collect();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_scales();

        state.check_ser();

        assert_eq!(state.voltage_set, 12.3);
        assert!(state.hw.serial.contains("0:0=12.300\r\n"));
        assert!(state.ser_input.is_empty());
    }

    #[test]
    fn init_all_follows_pascal_startup_order() {
        let hw = MockHardware {
            adc10: 1023,
            current_limit_sense: true,
            ..mock_hardware()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());

        state.init_all();

        assert_eq!(state.voltage_set, 5.0);
        assert_eq!(state.current_set, 0.02);
        assert_eq!(state.panel_modify, Modify::Volt);
        assert!(state.output_enabled);
        assert_eq!(state.hw.outputs, vec![true]);
        assert!(state
            .hw
            .serial
            .starts_with("0:254=2.92 [DCG by CM/c't 05/2010]\r\n"));
        assert_eq!(state.hw.input_relays, vec![false]);
        assert_eq!(state.capacity_mah, 0.0);
        assert_eq!(state.capacity_mwh, 0.0);
    }
