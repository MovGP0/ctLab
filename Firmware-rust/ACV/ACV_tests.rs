    use super::*;

    /// Verifies that named commands use pascal subchannel table order remains faithful to the Pascal behavior.
    #[test]
    fn named_commands_use_pascal_subchannel_table_order() {
        let expectations = [
            ("STR", 255),
            ("IDN", 254),
            ("VAL", 0),
            ("SMP", 8),
            ("INL", 10),
            ("RNG", 19),
            ("DSP", 80),
            ("ALL", 99),
            ("SCL", 200),
            ("WEN", 250),
            ("ERC", 251),
            ("SBD", 252),
            ("NOP", 0),
        ];

        let mut state = AcvState::default();
        for (command, sub_ch) in expectations {
            state.param_str = command.to_string();
            let cmd_which = state.cmd_to_index();

            assert_ne!(cmd_which, CmdWhich::Err);
            assert_eq!(cmd_which.as_str(), Some(command));
            assert_eq!(cmd_which.sub_channel_offset(), Some(sub_ch));
            assert_eq!(
                CmdWhich::from_mnemonic(&command.to_ascii_lowercase()),
                cmd_which
            );
        }

        assert_eq!(CmdWhich::from_mnemonic("unknown"), CmdWhich::Err);
        assert_eq!(CmdWhich::Err.as_str(), None);
        assert_eq!(Error::ChecksumErr.as_str(), "[CHKSUM]");
    }

    /// Verifies that serial aux bit bangs pb4 lsb first and leaves line high remains faithful to the Pascal behavior.
    #[test]
    fn ser_aux_bit_bangs_pb4_lsb_first_and_leaves_line_high() {
        let mut state = AcvState::default();
        state.hw.port_b = PORTB_INIT;

        state.ser_aux(0b1010_0101);

        assert_eq!(state.hw.aux_serial_log, vec![0b1010_0101]);
        assert_eq!(
            state.hw.aux_serial_bits,
            vec![false, true, false, true, false, false, true, false, true, true]
        );
        assert_ne!(state.hw.port_b & (1 << B_SER_AUX), 0);
    }

    /// Verifies that init spdif updates ADC board config for sample rate remains faithful to the Pascal behavior.
    #[test]
    fn init_spdif_updates_adc_board_config_for_sample_rate() {
        let mut state = AcvState {
            spdif_rate: Spdif::C48,
            ..Default::default()
        };

        state.init_spdif();
        assert_eq!(state.hw.adc_config, 0b0100_0100);
        assert_eq!(state.hw.i2c_registers[0x04], 0b0110_0000);

        state.spdif_rate = Spdif::P96;
        state.init_spdif();
        assert_eq!(state.hw.adc_config, 0b0100_0101);
        assert_eq!(state.hw.i2c_registers[0x04], 0b0100_0000);

        state.spdif_rate = Spdif::C192;
        state.init_spdif();
        assert_eq!(state.hw.adc_config, 0b0100_0110);
        assert_eq!(state.hw.i2c_registers[0x04], 0b0111_0000);
    }

    /// Verifies that init all applies stored uart baud register remains faithful to the Pascal behavior.
    #[test]
    fn init_all_applies_stored_uart_baud_register() {
        let mut state = AcvState::default();
        state.eeprom.ee_ser_baud_reg = 100;

        state.init_all();

        assert_eq!(state.hw.uart_baud_reg, 100);
        assert!(state.hw.uart_double_speed);
    }

    /// Verifies that init all restores default uart baud register when stored value is invalid remains faithful to the Pascal behavior.
    #[test]
    fn init_all_restores_default_uart_baud_register_when_stored_value_is_invalid() {
        let mut state = AcvState::default();
        state.eeprom.ee_ser_baud_reg = 8;

        state.init_all();

        assert_eq!(state.eeprom.ee_ser_baud_reg, 51);
        assert_eq!(state.hw.uart_baud_reg, 51);
        assert!(state.hw.uart_double_speed);
    }

    /// Verifies that check delay services queued serial input remains faithful to the Pascal behavior.
    #[test]
    fn check_delay_services_queued_serial_input() {
        let mut state = AcvState::default();

        for ch in "8\r".chars() {
            state.push_serial_char(ch);
        }
        assert!(state.hw.serial_output.is_empty());

        state.check_delay(1);

        assert_eq!(state.hw.serial_output, "#0:8=0\r\n");
        assert!(state.hw.serial_input.is_empty());
    }

    /// Verifies that front panel buttons require debounce and release before repeating remains faithful to the Pascal behavior.
    #[test]
    fn front_panel_buttons_require_debounce_and_release_before_repeating() {
        let mut state = AcvState::default();
        state.hw.lcd_present = true;
        state.display_timer.set(10);
        state.incr_timer.set(20);
        state.modify = Modify::GainSel;

        let left_pressed = 0b1101_1111;
        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::GainSel);

        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::LevelBarDispl);

        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::LevelBarDispl);

        state.main_loop_step(0, Some(BUTTON_RELEASED));
        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::LevelBarDispl);

        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::MvDispl);
    }

    /// Verifies that level bar display preserves channel columns and pascal markers remains faithful to the Pascal behavior.
    #[test]
    fn level_bar_display_preserves_channel_columns_and_pascal_markers() {
        let mut state = AcvState::default();
        state.hw.lcd_present = true;
        state.modify = Modify::LevelBarDispl;
        state.hw.adc_values[3] = 320;
        state.hw.adc_values[4] = 800;

        state.soll_werte_on_lcd();

        assert_eq!(state.hw.lcd_lines[0], "L##   \u{7} ");
        assert_eq!(state.hw.lcd_lines[1], "R######\u{6}");
    }

    /// Verifies that LCD menu pages include pascal cursor glyph and fixed width rows remains faithful to the Pascal behavior.
    #[test]
    fn lcd_menu_pages_include_pascal_cursor_glyph_and_fixed_width_rows() {
        let mut state = AcvState {
            modify: Modify::GainSel,
            gain: 2,
            ..Default::default()
        };

        state.soll_werte_on_lcd();

        assert_eq!(state.hw.lcd_lines[0], " +0 dB \u{5}");
        assert_eq!(state.hw.lcd_lines[1], GAIN_SEL_STR);
    }
