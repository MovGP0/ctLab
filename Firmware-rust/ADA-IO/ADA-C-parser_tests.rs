use crate::test_failures::TestFailures;
use super::{AdaIoParser, CmdWhich, ParseError, Reply};

/// Verifies that command and error enums own the exact wire text without positional string tables.
#[test]
fn enum_text_mappings_preserve_wire_protocol() {
    let mut assert = TestFailures::default();

    assert.eq(CmdWhich::from_mnemonic("  iCs  "), CmdWhich::Ics);
    assert.eq(CmdWhich::Ics.as_str(), Some("ICS"));
    assert.eq(CmdWhich::from_mnemonic("unknown"), CmdWhich::Err);
    assert.eq(CmdWhich::Err.as_str(), None);
    assert.eq(ParseError::ChecksumErr.as_str(), "[CHKSUM]");
    assert.finish();
}

/// Appends the protocol's XOR checksum as two hexadecimal digits for valid-frame parser tests.
fn checksum(frame: &str) -> String {
    let checksum = frame.bytes().fold(0u8, |acc, ch| acc ^ ch);
    format!("{frame}${checksum:02X}")
}

/// Compares floating-point results with the tolerance appropriate for translated calibration arithmetic.
fn assert_float_reply(
    assert: &mut TestFailures,
    reply: Option<&Reply>,
    expected_sub_ch: u8,
    expected_value: f32,
) {
    match reply {
        Some(Reply::Float { sub_ch, value }) => {
            assert.eq(*sub_ch, expected_sub_ch);
            assert.is_true_with_message(
                (*value - expected_value).abs() < f32::EPSILON,
                format_args!("expected {expected_value}, got {value}"),
            );
        }
        other => assert.fail(format_args!("expected float reply, got {other:?}")),
    }
}

/// Verifies that omni frames are forwarded and executed locally remains faithful to the Pascal behavior.
#[test]
fn omni_frames_are_forwarded_and_executed_locally() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.ser_inp_str = "*:TRG".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert.eq(replies.first(), Some(&Reply::Echo("*:TRG".to_string())));
    assert.eq(
        replies.get(1),
        Some(&Reply::Status {
            error: ParseError::NoErr,
            status: 0,
        }),
    );
    assert.is_true(parser.ctx.trigger);
    assert.is_true(parser.ctx.led_activity_low);
    assert.eq(parser.ctx.activity_timer_ticks, 125);
    assert.finish();
}

/// Verifies that mnemonic commands accept missing subchannel remains faithful to the Pascal behavior.
#[test]
fn mnemonic_commands_accept_missing_subchannel() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.ser_inp_str = "0:TRT?".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert.eq(
        replies,
        vec![Reply::Int {
            sub_ch: 247,
            value: 0,
        }],
    );
    assert.finish();
}

/// Verifies that omni frames validate checksum before local execution remains faithful to the Pascal behavior.
#[test]
fn omni_frames_validate_checksum_before_local_execution() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.ser_inp_str = "*:TRG$00".to_string();

    let result = parser.parse_sub_ch();

    assert.eq(result, Err(ParseError::ChecksumErr));
    assert.is_false(parser.ctx.trigger);
    assert.is_false(parser.ctx.led_activity_low);
    assert.eq(parser.ctx.activity_timer_ticks, 0);
    assert.finish();
}

/// Verifies that omni frames with valid checksum refresh activity and execute remains faithful to the Pascal behavior.
#[test]
fn omni_frames_with_valid_checksum_refresh_activity_and_execute() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.ser_inp_str = checksum("*:TRG");

    let replies = parser.parse_sub_ch().unwrap();

    assert.is_true(matches!(replies.first(), Some(Reply::Echo(_))));
    assert.is_true(parser.ctx.trigger);
    assert.is_true(parser.ctx.led_activity_low);
    assert.eq(parser.ctx.activity_timer_ticks, 125);
    assert.finish();
}

/// Verifies that live ADC reads use pascal scaling tables remains faithful to the Pascal behavior.
#[test]
fn live_adc_reads_use_pascal_scaling_tables() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.adc10_raw_array[0] = 123;
    parser.ctx.offset_array[0] = 2;
    parser.ctx.scale_array[0] = 2.0;
    parser.ctx.ser_inp_str = "0:VAL 0?".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert_float_reply(&mut assert, replies.first(), 0, 2.5);

    parser.ctx.adc_raw_array[0] = 3225;
    parser.ctx.ser_inp_str = "0:VAL 10?".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert_float_reply(&mut assert, replies.first(), 10, 1.0);
    assert.finish();
}

/// Verifies that raw ADC aliases return integer samples remains faithful to the Pascal behavior.
#[test]
fn raw_adc_aliases_return_integer_samples() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.adc_raw_array[0] = 3225;
    parser.ctx.ser_inp_str = "0:RAW 10?".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert.eq(
        replies,
        vec![Reply::Int {
            sub_ch: 60,
            value: 3225,
        }],
    );
    assert.finish();
}

/// Verifies that DAC set updates raw output value remains faithful to the Pascal behavior.
#[test]
fn dac_set_updates_raw_output_value() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.dac12_present = true;
    parser.ctx.ser_inp_str = "0:VAL 20=1.0".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert.eq(
        replies,
        vec![Reply::Status {
            error: ParseError::NoErr,
            status: 0,
        }],
    );
    assert.eq(parser.ctx.dac_value_array[0], 1.0);
    assert.eq(parser.ctx.dac_raw_array[0], 0x0800 - 200);
    assert.finish();
}

/// Verifies that port outputs update shift register state without I2C expander remains faithful to the Pascal behavior.
#[test]
fn port_outputs_update_shift_register_state_without_i2c_expander() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.ser_inp_str = "0:PIO 3=85".to_string();

    parser.parse_sub_ch().unwrap();

    assert.eq(parser.ctx.port_array[3], 85);
    assert.eq(parser.ctx.shift_register_writes.last().unwrap()[3], 85);
    assert.is_true(parser.ctx.i2c_writes.is_empty());
    assert.finish();
}

/// Verifies that port outputs use pascal I2C expander command when present remains faithful to the Pascal behavior.
#[test]
fn port_outputs_use_pascal_i2c_expander_command_when_present() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.io_present = true;
    parser.ctx.ser_inp_str = "0:PIO 3=85".to_string();

    parser.parse_sub_ch().unwrap();

    assert.eq(parser.ctx.port_array[3], 85);
    assert.eq(parser.ctx.i2c_slave_adr, 0x3b);
    assert.eq(parser.ctx.param_int, 0x0155);
    assert.eq(parser.ctx.i2c_writes, vec![(0x3b, 0x0155)]);
    assert.is_true(parser.ctx.shift_register_writes.is_empty());
    assert.finish();
}

/// Verifies that I2C commands read and write pascal payloads remains faithful to the Pascal behavior.
#[test]
fn i2c_commands_read_and_write_pascal_payloads() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.i2c_slave_adr = 0x48;
    parser.ctx.i2c_byte_reads.push_back(0x5a);
    parser.ctx.ser_inp_str = "0:ICB?".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert.eq(
        replies,
        vec![Reply::Int {
            sub_ch: 230,
            value: 0x5a,
        }],
    );

    parser.ctx.i2c_word_reads.push_back(0x1234);
    parser.ctx.ser_inp_str = "0:ICS?".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert.eq(
        replies,
        vec![Reply::Int {
            sub_ch: 232,
            value: 0x3412,
        }],
    );

    parser.ctx.ser_inp_str = "0:ICW=4660".to_string();
    parser.parse_sub_ch().unwrap();
    parser.ctx.ser_inp_str = "0:ICS=4660".to_string();
    parser.parse_sub_ch().unwrap();

    assert.eq(parser.ctx.i2c_writes, vec![(0x48, 0x1234), (0x48, 0x3412)]);
    assert.finish();
}

/// Verifies that EEPROM backed options update runtime and default mirrors remains faithful to the Pascal behavior.
#[test]
fn eeprom_backed_options_update_runtime_and_default_mirrors() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.ser_inp_str = "0:WEN=1".to_string();
    parser.parse_sub_ch().unwrap();
    parser.ctx.ser_inp_str = "0:OPT 9=6".to_string();

    parser.parse_sub_ch().unwrap();

    assert.eq(parser.ctx.inc_rast, 6);
    assert.eq(parser.ctx.inc_rast_def, 6);
    assert.is_false(parser.ctx.ee_unlocked);

    parser.ctx.ser_inp_str = "0:WEN=1".to_string();
    parser.parse_sub_ch().unwrap();
    parser.ctx.ser_inp_str = "0:OPT 7=1".to_string();

    parser.parse_sub_ch().unwrap();

    assert.is_true(parser.ctx.integrate_ad16);
    assert.is_true(parser.ctx.init_integrate_ad16);
    assert.finish();
}

/// Verifies that idn reply includes pascal feature suffix remains faithful to the Pascal behavior.
#[test]
fn idn_reply_includes_pascal_feature_suffix() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.vers1_str = "1.742 [ADA by CM/c't 04/2007; ".to_string();
    parser.ctx.dac12_present = true;
    parser.ctx.dac16_present = true;
    parser.ctx.adc16_present = true;
    parser.ctx.io_present = true;
    parser.ctx.lcd_present = true;
    parser.ctx.ser_inp_str = "0:IDN?".to_string();

    let replies = parser.parse_sub_ch().unwrap();

    assert.eq(
        replies,
        vec![Reply::Text(
            "0:1.742 [ADA by CM/c't 04/2007; DA12 DA16 AD16 IO32 LCD ]".to_string()
        )],
    );
    assert.finish();
}

/// Verifies that erc set requires value like pascal command table remains faithful to the Pascal behavior.
#[test]
fn erc_set_requires_value_like_pascal_command_table() {
    let mut assert = TestFailures::default();

    let mut parser = AdaIoParser::default();
    parser.ctx.ser_inp_str = "0:ERC=".to_string();

    let result = parser.parse_sub_ch();

    assert.eq(result, Err(ParseError::ParamErr));
    assert.eq(parser.ctx.err_count, 0);
    assert.finish();
}
