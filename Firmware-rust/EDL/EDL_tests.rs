    use super::*;

    #[derive(Debug, Default, Clone)]
    struct MockHardware {
        voltage_on: u16,
        voltage_off: u16,
        current_on: u16,
        current_off: u16,
        voltage10: i16,
        current10: i16,
        temp_c: Option<Float>,
        trigger_in: bool,
        shunts: Vec<u8>,
        outputs: Vec<bool>,
        dacs: Vec<u16>,
        lm75_writes: Vec<(u8, u8, Vec<u8>)>,
        serial: Vec<String>,
        lcd: Vec<(u8, String)>,
    }

    impl MockHardware {
        fn serial_joined(&self) -> String {
            self.serial.join("")
        }
    }

    impl EdlHardware for MockHardware {
        fn read_voltage_adc16(&mut self, on_phase: bool) -> u16 {
            if on_phase {
                self.voltage_on
            } else {
                self.voltage_off
            }
        }

        fn read_current_adc16(&mut self, on_phase: bool) -> u16 {
            if on_phase {
                self.current_on
            } else {
                self.current_off
            }
        }

        fn read_voltage_adc10(&mut self) -> i16 {
            self.voltage10
        }

        fn read_current_adc10(&mut self) -> i16 {
            self.current10
        }

        fn set_shunt(&mut self, shunt_index: u8) {
            self.shunts.push(shunt_index);
        }

        fn set_output_enabled(&mut self, enabled: bool) {
            self.outputs.push(enabled);
        }

        fn set_dac_raw(&mut self, raw: u16) {
            self.dacs.push(raw);
        }

        fn read_temp_c(&mut self) -> Option<Float> {
            self.temp_c
        }

        fn lm75_write(&mut self, address: u8, register: u8, data: &[u8]) {
            self.lm75_writes.push((address, register, data.to_vec()));
        }

        fn serial_write(&mut self, text: &str) {
            self.serial.push(text.to_string());
        }

        fn lcd_write_line(&mut self, row: u8, text: &str) {
            self.lcd.push((row, text.to_string()));
        }

        fn read_trigger_in(&mut self) -> bool {
            self.trigger_in
        }
    }

    fn test_eeprom() -> EepromData {
        let mut eeprom = EepromData {
            adc_u_offsets: [0, 0],
            adc_u_scales: [1.0, 1.0],
            adc_i_offsets: [0, 0, 0, 0],
            adc_i_scales: [1.0, 1.0, 1.0, 1.0],
            dac_i_offsets: [0, 0, 0, 0],
            dac_i_scales: [1.0, 1.0, 1.0, 1.0],
            ..EepromData::default()
        };
        eeprom.option_array[OptionSlot::LowVoltageDivider.index()] = 1.0;
        eeprom.option_array[OptionSlot::HighVoltageDivider.index()] = 1.0;
        eeprom.option_array[OptionSlot::CurrentMeasurementGain.index()] = 1.0;
        eeprom.option_array[OptionSlot::ReferenceVoltage.index()] = 1.0;
        eeprom.option_array[OptionSlot::MaximumPower.index()] = 2.0;
        let sense_resistance = OptionSlot::SenseResistanceA.index();
        eeprom.option_array[sense_resistance..sense_resistance + 4]
            .copy_from_slice(&[10.0, 1.0, 0.5, 0.1]);
        let maximum_current = OptionSlot::MaximumCurrentA.index();
        eeprom.option_array[maximum_current..maximum_current + 4]
            .copy_from_slice(&[0.01, 0.1, 0.5, 2.0]);
        eeprom.option_array[OptionSlot::HighVoltageLimit.index()] = 2.0;
        eeprom.option_array[OptionSlot::LowVoltageLimit.index()] = 1.0;
        eeprom.option_array[OptionSlot::InitialCurrentPercent.index()] = 100.0;
        eeprom.option_array[OptionSlot::InitialRippleOnTime.index()] = 10.0;
        eeprom.option_array[OptionSlot::InitialRippleOffTime.index()] = 0.0;
        eeprom.option_array[OptionSlot::InstalledHardware.index()] = 4.0;
        eeprom
    }

    #[test]
    fn default_eeprom_matches_pascal_layout() {
        let eeprom = EepromData::default();
        assert_eq!(eeprom.dac_i_offsets, [0, 0, 0, 0]);
        assert_eq!(eeprom.adc_u_offsets, [-260, -260]);
        assert_eq!(eeprom.option_array[OptionSlot::InitialCurrent.index()], 0.02);
        assert_eq!(eeprom.option_array[OptionSlot::SenseResistanceA.index()], 100.0);
        assert_eq!(eeprom.option_array[OptionSlot::SenseResistanceD.index()], 0.1);
        assert_eq!(eeprom.option_array[OptionSlot::MaximumCurrentA.index()], 0.002);
        assert_eq!(eeprom.option_array[OptionSlot::MaximumCurrentD.index()], 2.0);
        assert_eq!(eeprom.option_array[OptionSlot::HighVoltageLimit.index()], 25.0);
        assert_eq!(eeprom.option_array[OptionSlot::LowVoltageLimit.index()], 6.1);
        assert_eq!(eeprom.option_array[OptionSlot::InstalledHardware.index()], 4.0);
        assert_eq!(eeprom.option_array[OptionSlot::InitialRippleOnTime.index()], 10.0);
        assert_eq!(eeprom.option_array[OptionSlot::InitialRippleOffTime.index()], 0.0);
        assert_eq!(eeprom.option_array[OptionSlot::FanOnTemperature.index()], 50.0);
    }

    #[test]
    fn set_lm75_temp_programs_pascal_threshold_hysteresis_and_pointer_sequence() {
        let hw = MockHardware::default();
        let mut eeprom = test_eeprom();
        eeprom.option_array[OptionSlot::InstalledHardware.index()] = Float::from(
            HardwareOption::InternalLm75.mask() | HardwareOption::ExternalLm75.mask(),
        );
        eeprom.option_array[OptionSlot::FanOnTemperature.index()] = 50.0;
        let mut state = DeviceState::with_eeprom(hw, eeprom);
        state.hw.lm75_writes.clear();

        state.set_lm75_temp();

        let one_sensor = |address| {
            vec![
                (
                    address,
                    Lm75Register::Configuration.address(),
                    vec![LM75_INVERTED_OUTPUT_CONFIGURATION],
                ),
                (
                    address,
                    Lm75Register::Overtemperature.address(),
                    vec![0x32, 0x00],
                ),
                (
                    address,
                    Lm75Register::Hysteresis.address(),
                    vec![0x2f, 0x00],
                ),
                (address, Lm75Register::Temperature.address(), vec![]),
            ]
        };
        let mut expected = one_sensor(Lm75Sensor::Internal.address());
        expected.extend(one_sensor(Lm75Sensor::External.address()));
        assert_eq!(state.hw.lm75_writes, expected);
    }

    #[test]
    fn encoder_rounding_and_acceleration_match_pascal_current_adjustment() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.mode = Mode::IHiVolt;
        state.modify = Modify::LowerMainMenu;
        state.current_set = 0.12346;
        state.first_turn = true;

        state.inc_fac_i();
        state.round_inc_param();

        assert_eq!(state.inc_coarse_div, 100.0);
        assert_eq!(state.inc_fine_div, 10_000.0);
        assert!((state.current_set - 0.12).abs() < 0.000_001);
        assert!(!state.first_turn);

        state.incr_acc_float = 5.0;
        state.set_acc_param();
        assert!((state.incr_acc_float - 0.05).abs() < 0.000_001);
        assert!((state.current_set - 0.17).abs() < 0.000_001);
    }

    #[test]
    fn encoder_fine_adjustment_skips_coarse_rounding_and_uses_fine_divisor() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.mode = Mode::PHiVolt;
        state.modify = Modify::LowerMainMenu;
        state.power_set = 12.345;
        state.incr_fine = true;
        state.first_turn = true;

        state.inc_fac_p();
        state.round_inc_param();
        state.incr_acc_float = -2.0;
        state.set_acc_param();

        assert_eq!(state.inc_coarse_div, 10.0);
        assert_eq!(state.inc_fine_div, 100.0);
        assert!(state.first_turn);
        assert!((state.incr_acc_float + 0.02).abs() < 0.000_001);
        assert!((state.power_set - 12.325).abs() < 0.000_001);
    }

    #[test]
    fn check_limits_restores_clamps_and_ripple_normalization() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.mode = Mode::RLoVolt;
        state.init_scales();
        state.resistance_set = 0.01;
        state.current_set = 5.0;
        state.i_percent = 150;
        state.pw_on_time_ms = 0;
        state.pw_off_time_ms = -5;
        state.power_set = 5.0;
        state.voltage_cutoff = 5.0;
        state.track_channel = 9;

        let err = state.check_limits();

        assert_eq!(err, ErrorCode::ParamErr);
        assert_eq!(state.resistance_set, state.scale.dc_ohm_min);
        assert_eq!(state.current_set, 2.0);
        assert_eq!(state.i_percent, 100);
        assert_eq!(state.pw_on_time_ms, 1);
        assert_eq!(state.pw_off_time_ms, 0);
        assert_eq!(state.power_set, 2.0);
        assert_eq!(state.voltage_cutoff, 1.0);
        assert_eq!(state.track_channel, 7);
        assert!(state.no_toggle);
    }

    #[test]
    fn status_prompt_reports_each_fault_instead_of_single_overload_bit() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.faults = ProtectionFlags {
            over_power: true,
            fuse_blown: false,
            over_voltage: true,
            over_temp: true,
            low_volt: true,
        };
        state.status.overload_flag = true;
        let frame = state.status_frame(ErrorCode::NoErr);

        assert!(frame.contains("[OVRPOWR]"));
        assert!(frame.contains("[OVRVOLT]"));
        assert!(frame.contains("[OVRTEMP]"));
        assert!(frame.contains("[LOWVOLT]"));
        assert!(frame.starts_with("#0:255=61 "));
    }

    #[test]
    fn current_and_resistance_modes_restore_auto_range_behavior() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());

        state.mode = Mode::IHiVolt;
        state.current_set = 0.2;
        state.i_percent = 50;
        state.pw_off_time_ms = 10;
        state.check_limits();
        state.set_level_dac_i();
        assert_eq!(state.shunt_select, 2);
        assert!(state.dac_raw_on > state.dac_raw_off);

        state.mode = Mode::RLoVolt;
        state.init_scales();
        state.resistance_set = 0.05;
        state.check_limits();
        state.set_level_dac_r();
        assert_eq!(state.shunt_select, 3);
        assert_eq!(state.dac_raw_on, state.dac_raw_off);
        assert_eq!(state.hw.shunts.last().copied(), Some(3));
    }

    #[test]
    fn init_all_restores_startup_constants_and_banner() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());

        state.init_all();

        assert_eq!(state.mode, Mode::IHiVolt);
        assert!(state.output_enabled);
        assert_eq!(state.current_set, state.eeprom.init_amp());
        assert_eq!(state.shunt_range, AUTO_SHUNT_RANGE);
        assert_eq!(state.capacity_ah, 0.0);
        assert_eq!(state.capacity_wh, 0.0);
        assert!(state.hw.serial_joined().contains(VERS1_STR));
        assert!(state
            .hw
            .lcd
            .iter()
            .any(|(row, text)| *row == 0 && text == VERS3_STR));
    }

    #[test]
    fn service_cycle_restores_off_phase_sampling_average_power_and_telemetry() {
        let hw = MockHardware {
            voltage_on: 32_768,
            voltage_off: 65_535,
            current_on: 65_535,
            current_off: 32_768,
            ..MockHardware::default()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_all();
        state.mode = Mode::IHiVolt;
        state.output_enabled = true;
        state.i_percent = 50;
        state.pw_on_time_ms = 3;
        state.pw_off_time_ms = 1;
        state.set_trigger_mask(0x02);
        state.check_limits();
        state.service_cycle();

        assert!((state.last_measurement.voltage_on - 32_768.0 / 65_535.0).abs() < 0.0001);
        assert!((state.last_measurement.voltage_off - 1.0).abs() < 0.0001);
        assert!((state.measured_power - 0.5).abs() < 0.0001);
        assert!((state.internal_resistance - 1.0).abs() < 0.0002);
        let serial = state.hw.serial_joined();
        assert!(serial.contains("#0:10="));
        assert!(serial.contains("#0:11="));
        assert!(serial.contains("#0:15="));
        assert!(serial.contains("#0:16="));
    }

    #[test]
    fn output_off_zeroes_current_and_normalizes_off_phase_when_ripple_is_disabled() {
        let hw = MockHardware {
            voltage_on: 40_000,
            voltage_off: 50_000,
            current_on: 30_000,
            current_off: 10_000,
            ..MockHardware::default()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_all();
        state.output_enabled = false;
        state.i_percent = 100;
        state.pw_off_time_ms = 0;
        state.check_limits();

        let on_voltage = state.read_voltage_phase(true);
        let off_voltage = state.read_voltage_phase(false);
        let off_current = state.read_current_phase(false);
        state.service_cycle();

        assert!((on_voltage - off_voltage).abs() < 0.0001);
        assert_eq!(off_current, 0.0);
        assert_eq!(state.measured_current, 0.0);
        assert_eq!(state.measured_power, 0.0);
    }

    #[test]
    fn energy_integration_accumulates_and_resets_like_pascal() {
        let hw = MockHardware {
            voltage_on: 65_535,
            voltage_off: 65_535,
            current_on: 65_535,
            current_off: 65_535,
            ..MockHardware::default()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_all();
        state.output_enabled = true;
        state.i_percent = 100;
        state.pw_off_time_ms = 0;
        state.check_limits();
        state.service_step(200);

        assert!(state.capacity_ah > 0.0);
        assert!(state.capacity_wh > 0.0);

        state.i_percent = 50;
        state.pw_off_time_ms = 10;
        state.check_limits();
        state.service_step(200);
        assert_eq!(state.capacity_ah, 0.0);
        assert_eq!(state.capacity_wh, 0.0);

        state.capacity_ah = 1.0;
        state.capacity_wh = 2.0;
        state.reset_energy_counters();
        assert_eq!(state.capacity_ah, 0.0);
        assert_eq!(state.capacity_wh, 0.0);
    }
