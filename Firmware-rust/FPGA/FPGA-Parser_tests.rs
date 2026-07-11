    use super::*;

    #[test]
    fn all_pascal_commands_keep_their_offsets()
    {
        assert_eq!(COMMANDS.len(), 66);
        assert_eq!(COMMAND_OFFSETS.len(), 66);
        assert_eq!(command_subchannel("REG", 9), Some(309));
        assert_eq!(command_subchannel("BRL", 0), Some(1600));
        assert_eq!(command_subchannel("OUT", 63), Some(2063));
        assert_eq!(command_subchannel("NOP", 0), Some(253));
    }

    #[test]
    fn parses_channel_command_argument_and_numeric_value()
    {
        let parsed = parse_frame("9:REG 3=-12.5!").unwrap();

        assert_eq!(parsed.main_channel, Some(9));
        assert_eq!(parsed.subchannel, 303);
        assert_eq!(parsed.parameter, Parameter::Number(-12.5));
        assert!(parsed.verbose);
        assert!(!parsed.is_request);
    }

    #[test]
    fn parses_quoted_file_names_and_direct_subchannels()
    {
        let parsed = parse_frame("9:243=\"DATAFILE.XLS\"").unwrap();

        assert_eq!(parsed.subchannel, 243);
        assert_eq!(parsed.parameter, Parameter::Text("DATAFILE.XLS".to_string()));
    }

    #[test]
    fn validates_pascal_xor_checksum()
    {
        let command = "9:VAL?";
        let frame = format!("{command}${:02X}", xor_checksum(command.as_bytes()));
        assert!(parse_frame(&frame).is_ok());
        assert_eq!(parse_frame("9:VAL?$00"), Err(ParseError::InvalidChecksum));
    }

    #[test]
    fn identifies_results_and_omni_frames()
    {
        assert!(parse_frame("#9:10=123").unwrap().is_result);
        assert!(parse_frame("*:NOP").unwrap().is_omni);
    }
