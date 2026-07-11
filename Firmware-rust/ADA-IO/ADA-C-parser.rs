//! Best-effort Rust port of `ADA-C-parser.pas`.
//!
//! This keeps the original parser structure intact:
//! - command lookup via a fixed command enum/table
//! - `parse_get_param` and `parse_set_param` large dispatches
//! - `parse_extract`, `cmd_to_index`, and `parse_sub_ch` flow
//!
//! Hardware-facing helpers are intentionally lightweight stubs so the parser
//! logic remains readable and can be integrated with a real backend later.

use std::{collections::VecDeque, mem};

#[path = "ada_c_parser/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;
#[path = "ada_c_parser/parse_error.rs"]
mod parse_error;
pub use parse_error::ParseError;
#[path = "ada_c_parser/reply.rs"]
mod reply;
pub use reply::Reply;
#[path = "ada_c_parser/parse_context.rs"]
mod parse_context;
pub use parse_context::ParseContext;
#[path = "ada_c_parser/ada_io_parser.rs"]
mod ada_io_parser;
pub use ada_io_parser::AdaIoParser;

#[cfg(test)]
mod tests {
    use super::{AdaIoParser, ParseError, Reply};

    fn checksum(frame: &str) -> String {
        let checksum = frame.bytes().fold(0u8, |acc, ch| acc ^ ch);
        format!("{frame}${checksum:02X}")
    }

    fn assert_float_reply(reply: &Reply, expected_sub_ch: u8, expected_value: f32) {
        match reply {
            Reply::Float { sub_ch, value } => {
                assert_eq!(*sub_ch, expected_sub_ch);
                assert!((*value - expected_value).abs() < f32::EPSILON);
            }
            other => panic!("expected float reply, got {:?}", other),
        }
    }

    #[test]
    fn omni_frames_are_forwarded_and_executed_locally() {
        let mut parser = AdaIoParser::default();
        parser.ctx.ser_inp_str = "*:TRG".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_eq!(replies[0], Reply::Echo("*:TRG".to_string()));
        assert_eq!(
            replies[1],
            Reply::Status {
                error: ParseError::NoErr,
                status: 0,
            }
        );
        assert!(parser.ctx.trigger);
        assert!(parser.ctx.led_activity_low);
        assert_eq!(parser.ctx.activity_timer_ticks, 125);
    }

    #[test]
    fn mnemonic_commands_accept_missing_subchannel() {
        let mut parser = AdaIoParser::default();
        parser.ctx.ser_inp_str = "0:TRT?".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_eq!(
            replies,
            vec![Reply::Int {
                sub_ch: 247,
                value: 0,
            }]
        );
    }

    #[test]
    fn omni_frames_validate_checksum_before_local_execution() {
        let mut parser = AdaIoParser::default();
        parser.ctx.ser_inp_str = "*:TRG$00".to_string();

        let result = parser.parse_sub_ch();

        assert_eq!(result, Err(ParseError::ChecksumErr));
        assert!(!parser.ctx.trigger);
        assert!(!parser.ctx.led_activity_low);
        assert_eq!(parser.ctx.activity_timer_ticks, 0);
    }

    #[test]
    fn omni_frames_with_valid_checksum_refresh_activity_and_execute() {
        let mut parser = AdaIoParser::default();
        parser.ctx.ser_inp_str = checksum("*:TRG");

        let replies = parser.parse_sub_ch().unwrap();

        assert!(matches!(replies.first(), Some(Reply::Echo(_))));
        assert!(parser.ctx.trigger);
        assert!(parser.ctx.led_activity_low);
        assert_eq!(parser.ctx.activity_timer_ticks, 125);
    }

    #[test]
    fn live_adc_reads_use_pascal_scaling_tables() {
        let mut parser = AdaIoParser::default();
        parser.ctx.adc10_raw_array[0] = 123;
        parser.ctx.offset_array[0] = 2;
        parser.ctx.scale_array[0] = 2.0;
        parser.ctx.ser_inp_str = "0:VAL 0?".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_float_reply(&replies[0], 0, 2.5);

        parser.ctx.adc_raw_array[0] = 3225;
        parser.ctx.ser_inp_str = "0:VAL 10?".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_float_reply(&replies[0], 10, 1.0);
    }

    #[test]
    fn raw_adc_aliases_return_integer_samples() {
        let mut parser = AdaIoParser::default();
        parser.ctx.adc_raw_array[0] = 3225;
        parser.ctx.ser_inp_str = "0:RAW 10?".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_eq!(
            replies,
            vec![Reply::Int {
                sub_ch: 60,
                value: 3225,
            }]
        );
    }

    #[test]
    fn dac_set_updates_raw_output_value() {
        let mut parser = AdaIoParser::default();
        parser.ctx.dac12_present = true;
        parser.ctx.ser_inp_str = "0:VAL 20=1.0".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_eq!(
            replies,
            vec![Reply::Status {
                error: ParseError::NoErr,
                status: 0,
            }]
        );
        assert_eq!(parser.ctx.dac_value_array[0], 1.0);
        assert_eq!(parser.ctx.dac_raw_array[0], 0x0800 - 200);
    }

    #[test]
    fn port_outputs_update_shift_register_state_without_i2c_expander() {
        let mut parser = AdaIoParser::default();
        parser.ctx.ser_inp_str = "0:PIO 3=85".to_string();

        parser.parse_sub_ch().unwrap();

        assert_eq!(parser.ctx.port_array[3], 85);
        assert_eq!(parser.ctx.shift_register_writes.last().unwrap()[3], 85);
        assert!(parser.ctx.i2c_writes.is_empty());
    }

    #[test]
    fn port_outputs_use_pascal_i2c_expander_command_when_present() {
        let mut parser = AdaIoParser::default();
        parser.ctx.io_present = true;
        parser.ctx.ser_inp_str = "0:PIO 3=85".to_string();

        parser.parse_sub_ch().unwrap();

        assert_eq!(parser.ctx.port_array[3], 85);
        assert_eq!(parser.ctx.i2c_slave_adr, 0x3b);
        assert_eq!(parser.ctx.param_int, 0x0155);
        assert_eq!(parser.ctx.i2c_writes, vec![(0x3b, 0x0155)]);
        assert!(parser.ctx.shift_register_writes.is_empty());
    }

    #[test]
    fn i2c_commands_read_and_write_pascal_payloads() {
        let mut parser = AdaIoParser::default();
        parser.ctx.i2c_slave_adr = 0x48;
        parser.ctx.i2c_byte_reads.push_back(0x5a);
        parser.ctx.ser_inp_str = "0:ICB?".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_eq!(
            replies,
            vec![Reply::Int {
                sub_ch: 230,
                value: 0x5a,
            }]
        );

        parser.ctx.i2c_word_reads.push_back(0x1234);
        parser.ctx.ser_inp_str = "0:ICS?".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_eq!(
            replies,
            vec![Reply::Int {
                sub_ch: 232,
                value: 0x3412,
            }]
        );

        parser.ctx.ser_inp_str = "0:ICW=4660".to_string();
        parser.parse_sub_ch().unwrap();
        parser.ctx.ser_inp_str = "0:ICS=4660".to_string();
        parser.parse_sub_ch().unwrap();

        assert_eq!(parser.ctx.i2c_writes, vec![(0x48, 0x1234), (0x48, 0x3412)]);
    }

    #[test]
    fn eeprom_backed_options_update_runtime_and_default_mirrors() {
        let mut parser = AdaIoParser::default();
        parser.ctx.ser_inp_str = "0:WEN=1".to_string();
        parser.parse_sub_ch().unwrap();
        parser.ctx.ser_inp_str = "0:OPT 9=6".to_string();

        parser.parse_sub_ch().unwrap();

        assert_eq!(parser.ctx.inc_rast, 6);
        assert_eq!(parser.ctx.inc_rast_def, 6);
        assert!(!parser.ctx.ee_unlocked);

        parser.ctx.ser_inp_str = "0:WEN=1".to_string();
        parser.parse_sub_ch().unwrap();
        parser.ctx.ser_inp_str = "0:OPT 7=1".to_string();

        parser.parse_sub_ch().unwrap();

        assert!(parser.ctx.integrate_ad16);
        assert!(parser.ctx.init_integrate_ad16);
    }

    #[test]
    fn idn_reply_includes_pascal_feature_suffix() {
        let mut parser = AdaIoParser::default();
        parser.ctx.vers1_str = "1.742 [ADA by CM/c't 04/2007; ".to_string();
        parser.ctx.dac12_present = true;
        parser.ctx.dac16_present = true;
        parser.ctx.adc16_present = true;
        parser.ctx.io_present = true;
        parser.ctx.lcd_present = true;
        parser.ctx.ser_inp_str = "0:IDN?".to_string();

        let replies = parser.parse_sub_ch().unwrap();

        assert_eq!(
            replies,
            vec![Reply::Text(
                "0:1.742 [ADA by CM/c't 04/2007; DA12 DA16 AD16 IO32 LCD ]".to_string()
            )]
        );
    }

    #[test]
    fn erc_set_requires_value_like_pascal_command_table() {
        let mut parser = AdaIoParser::default();
        parser.ctx.ser_inp_str = "0:ERC=".to_string();

        let result = parser.parse_sub_ch();

        assert_eq!(result, Err(ParseError::ParamErr));
        assert_eq!(parser.ctx.err_count, 0);
    }
}
