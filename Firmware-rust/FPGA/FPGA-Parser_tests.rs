use crate::test_failures::TestFailures;
use super::*;

#[test]
fn all_pascal_commands_keep_their_offsets()
{
    let mut assert = TestFailures::default();

    assert.eq(COMMANDS.len(), 66);
    assert.eq(COMMAND_OFFSETS.len(), 66);
    assert.eq(command_subchannel("REG", 9), Some(309));
    assert.eq(command_subchannel("BRL", 0), Some(1600));
    assert.eq(command_subchannel("OUT", 63), Some(2063));
    assert.eq(command_subchannel("NOP", 0), Some(253));
    assert.finish();
}

#[test]
fn parses_channel_command_argument_and_numeric_value()
{
    let mut assert = TestFailures::default();

    let parsed = parse_frame("9:REG 3=-12.5!").unwrap();

    assert.eq(parsed.main_channel, Some(9));
    assert.eq(parsed.subchannel, 303);
    assert.eq(parsed.parameter, Parameter::Number(-12.5));
    assert.is_true(parsed.verbose);
    assert.is_false(parsed.is_request);
    assert.finish();
}

#[test]
fn parses_quoted_file_names_and_direct_subchannels()
{
    let mut assert = TestFailures::default();

    let parsed = parse_frame("9:243=\"DATAFILE.XLS\"").unwrap();

    assert.eq(parsed.subchannel, 243);
    assert.eq(parsed.parameter, Parameter::Text("DATAFILE.XLS".to_string()));
    assert.finish();
}

#[test]
fn validates_pascal_xor_checksum()
{
    let mut assert = TestFailures::default();

    let command = "9:VAL?";
    let frame = format!("{command}${:02X}", xor_checksum(command.as_bytes()));
    assert.is_true(parse_frame(&frame).is_ok());
    assert.eq(parse_frame("9:VAL?$00"), Err(ParseError::InvalidChecksum));
    assert.finish();
}

#[test]
fn identifies_results_and_omni_frames()
{
    let mut assert = TestFailures::default();

    assert.is_true(parse_frame("#9:10=123").unwrap().is_result);
    assert.is_true(parse_frame("*:NOP").unwrap().is_omni);
    assert.finish();
}
