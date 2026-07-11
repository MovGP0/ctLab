    use super::{CmdWhich, DcgParser, OptionSlot};

    #[test]
    fn command_lookup_preserves_original_table() {
        let mut parser = DcgParser {
            param_str: "tmp".to_string(),
            ..DcgParser::default()
        };
        assert_eq!(parser.cmd_to_index(), CmdWhich::Tmp);
        assert_eq!(CmdWhich::Pwon.as_str(), "RON");
        assert_eq!(CmdWhich::from_mnemonic("rof"), CmdWhich::Pwoff);
        assert_eq!(CmdWhich::Pwoff.default_subchannel(), 28);
    }

    #[test]
    fn mixed_case_command_mnemonics_parse_like_pascal() {
        let mut parser = DcgParser {
            ser_inp_str: "0:dCv 0?".to_string(),
            dc_volt: 3.3,
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.cmd_which, CmdWhich::Dcv);
        assert!(parser
            .serial_log
            .iter()
            .any(|line| line.contains("0:0=3.3000")));
    }

    #[test]
    fn direct_sub_channel_request_keeps_structure() {
        let mut parser = DcgParser {
            ser_inp_str: "0:10?".to_string(),
            measured_voltage: 12.5,
            ..DcgParser::default()
        };
        parser.parse_sub_ch();
        assert!(parser
            .serial_log
            .iter()
            .any(|line| line.contains("0:10=12.5000")));
    }

    #[test]
    fn set_commands_check_limits_and_report_verbose_limit_errors() {
        let mut parser = DcgParser {
            ser_inp_str: "0:DCV 0=45!".to_string(),
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.dc_volt, parser.u_max);
        assert_eq!(parser.check_limit_err, super::Error::ParamErr);
        assert!(parser.serial_log.iter().any(|line| line == "[PARERR]"));
    }

    #[test]
    fn successful_set_commands_apply_new_dac_level() {
        let mut parser = DcgParser {
            ser_inp_str: "0:DCV 0=5".to_string(),
            dac_lsb_u: [0.5, 0.5],
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.check_limit_err, super::Error::NoErr);
        assert_eq!(parser.dc_volt, 5.0);
        assert_eq!(parser.dac_raw_uon, 11);
        assert_eq!(parser.dac_raw_uoff, 11);
    }

    #[test]
    fn option_writes_reload_calibration_and_honor_settle_delay() {
        let mut parser = DcgParser {
            ser_inp_str: "0:OPT 6=24".to_string(),
            ee_unlocked: true,
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.u_max, 24.0);
        assert_eq!(
            parser.option_array[OptionSlot::MaximumVoltage.index()],
            24.0
        );
        assert_eq!(parser.delay_log, vec![3]);
    }

    #[test]
    fn eeprom_scale_writes_recompute_lsb_calibration() {
        let mut parser = DcgParser {
            ser_inp_str: "0:SCL 0=2".to_string(),
            ee_unlocked: true,
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.dac_u_scales[0], 2.0);
        assert!((parser.dac_lsb_u[0] - 0.00011444092).abs() < 0.00000001);
        assert_eq!(parser.delay_log, vec![3]);
    }

    #[test]
    fn dsp_modify_writes_refresh_display_state() {
        let mut parser = DcgParser {
            ser_inp_str: "0:DSP 0=2".to_string(),
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.modify, super::Modify::Ripple);
        assert_eq!(parser.display_refresh_count, 1);
    }

    #[test]
    fn local_parser_activity_drives_activity_led_low() {
        let mut parser = DcgParser {
            ser_inp_str: "0:DCV 0?".to_string(),
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.activity_timer, 255);
        assert!(parser.led_activity_low);
    }

    #[test]
    fn sub_channel_253_echoes_input_without_reply_prefix() {
        let mut parser = DcgParser {
            ser_inp_str: "0:253?".to_string(),
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.serial_log, vec!["0:253?"]);
    }

    #[test]
    fn sub_channel_254_uses_pascal_reply_framing() {
        let mut parser = DcgParser {
            ser_inp_str: "0:254?".to_string(),
            ..DcgParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(
            parser.serial_log,
            vec![format!("#0:254={}", super::VERS1_STR)]
        );
    }
