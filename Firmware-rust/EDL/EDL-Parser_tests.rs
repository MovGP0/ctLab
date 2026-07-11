use crate::test_failures::TestFailures;
use super::*;

#[test]
fn resolved_subchannels_preserve_alias_wire_values()
{
    let mut assert = TestFailures::default();

    let first_alias = ResolvedSubChannel::try_from(21);
    let second_alias = ResolvedSubChannel::try_from(22);

    assert.eq(
        first_alias,
        Ok(ResolvedSubChannel
        {
            wire_value: 21,
            operation: EdlSubChannel::CurrentModulationPercent,
        }),
    );
    assert.eq(
        second_alias,
        Ok(ResolvedSubChannel
        {
            wire_value: 22,
            operation: EdlSubChannel::CurrentModulationPercent,
        }),
    );

    let mut parser = EdlParser {
        ser_inp_str: "PCA 1?".to_owned(),
        dc_amp_mod: 0.25,
        slave_ch: 0,
        ..EdlParser::default()
    };
    assert.eq(parser.parse_sub_ch(), vec!["22=25.0000"]);
    assert.finish();
}

#[test]
fn indexed_subchannels_decode_to_domain_types()
{
    let mut assert = TestFailures::default();

    assert.eq(
        ResolvedSubChannel::try_from(105).map(|resolved| resolved.operation),
        Ok(EdlSubChannel::CurrentDacOffset(Shunt::D)),
    );
    assert.eq(
        ResolvedSubChannel::try_from(153).map(|resolved| resolved.operation),
        Ok(EdlSubChannel::Option(OptionSlot::HighVoltageDivider)),
    );
    assert.eq(
        ResolvedSubChannel::try_from(211).map(|resolved| resolved.operation),
        Ok(EdlSubChannel::VoltageAdcScale(VoltageRange::High)),
    );
    assert.eq(
        ResolvedSubChannel::try_from(234).map(|resolved| resolved.operation),
        Ok(EdlSubChannel::Temperature(Lm75Sensor::External)),
    );
    assert.eq(
        ResolvedSubChannel::try_from(-1),
        Err(InvalidSubChannel { wire_value: -1 }),
    );
    assert.eq(
        ResolvedSubChannel::try_from(6),
        Err(InvalidSubChannel { wire_value: 6 }),
    );
    assert.finish();
}

#[test]
fn reserved_pascal_calibration_channels_remain_unlocked_no_ops()
{
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        sub_ch: 216,
        param: 12.5,
        param_int: 12,
        ee_unlocked: true,
        ..EdlParser::default()
    };

    parser.parse_set_param();

    assert.is_true(parser.output_lines.is_empty());
    assert.is_false(parser.ee_unlocked);
    assert.finish();
}

#[test]
fn default_command_table_accepts_pascal_text_setter_syntax() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "DCA 0=1.25".to_owned(),
        slave_ch: 0,
        ..EdlParser::default()
    };

    let output = parser.parse_sub_ch();

    assert.is_true(output.is_empty());
    assert.eq(parser.sub_ch, 1);
    assert.eq(parser.dc_amp, 1.25);
    assert.finish();
}

#[test]
fn val_command_uses_pascal_offset_and_gets_same_channel_as_short_form() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "VAL 5?".to_owned(),
        dc_ohm: 123.4,
        slave_ch: 0,
        ..EdlParser::default()
    };

    let output = parser.parse_sub_ch();

    assert.eq(parser.sub_ch, 5);
    assert.eq(output, vec!["5=123.4"]);
    assert.finish();
}

#[test]
fn zero_parameter_text_command_uses_command_offset_as_sub_channel() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "IDN?".to_owned(),
        vers1_str: "EDL test",
        slave_ch: 0,
        ..EdlParser::default()
    };

    let output = parser.parse_sub_ch();

    assert.eq(parser.sub_ch, 254);
    assert.eq(output, vec!["0:EDL test"]);
    assert.finish();
}

#[test]
fn setters_enforce_pascal_limits_and_report_actual_limit_error_when_verbose() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "DCA 0=5!".to_owned(),
        slave_ch: 0,
        ..EdlParser::default()
    };

    let output = parser.parse_sub_ch();

    assert.eq(parser.dc_amp, 2.0);
    assert.eq(parser.check_limit_err, PromptCode::ParamErr);
    assert.eq(output, vec!["ParamErr"]);
    assert.finish();
}

#[test]
fn mode_numbers_match_pascal_and_drive_the_selected_dac_path() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "RNG 0=1".to_owned(),
        dc_amp: 1.0,
        dac_lsb_i: [0.5; DACI_COUNT],
        slave_ch: 0,
        ..EdlParser::default()
    };

    parser.parse_sub_ch();

    assert.eq(parser.mode_select, Mode::IhiVolt);
    assert.is_true(parser.mode_mpx);
    assert.is_true(parser.output_enable);
    assert.is_true(parser.mpxena);
    assert.eq(parser.dac_temp_on, 1638);
    assert.finish();
}

#[test]
fn power_mode_setpoint_recomputes_current_and_current_dac() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "DCP 0=4".to_owned(),
        mode_select: Mode::PhiVolt,
        voltage_on: 2.0,
        dac_lsb_i: [0.5; DACI_COUNT],
        slave_ch: 0,
        ..EdlParser::default()
    };

    parser.parse_sub_ch();

    assert.eq(parser.dc_watt, 4.0);
    assert.eq(parser.dc_amp, 2.0);
    assert.is_true(parser.mode_mpx);
    assert.eq(parser.dac_temp_on, 3276);
    assert.finish();
}

#[test]
fn raw_adc10_reads_return_backing_samples() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "52?".to_owned(),
        adc10: [0, 0, 0, 777, 888, 0],
        slave_ch: 0,
        ..EdlParser::default()
    };

    let output = parser.parse_sub_ch();

    assert.eq(output, vec!["52=777"]);

    parser.ser_inp_str = "53?".to_owned();
    let output = parser.parse_sub_ch();

    assert.eq(output, vec!["53=888"]);
    assert.finish();
}

#[test]
fn modify_writes_refresh_display_state_and_preserve_menu_value() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "DSP 0=8".to_owned(),
        slave_ch: 0,
        ..EdlParser::default()
    };

    parser.parse_sub_ch();

    assert.eq(parser.modify, Modify::CapMenu);
    assert.eq(parser.display_refresh_count, 1);

    parser.ser_inp_str = "DSP 0?".to_owned();
    let output = parser.parse_sub_ch();

    assert.eq(output, vec!["80=8"]);
    assert.finish();
}

#[test]
fn unlocked_calibration_and_option_writes_recalculate_scales() {
    let mut assert = TestFailures::default();

    let mut parser = EdlParser {
        ser_inp_str: "OPT 4=20".to_owned(),
        ee_unlocked: true,
        slave_ch: 0,
        ..EdlParser::default()
    };

    parser.init_scales();
    let initial_lsb = parser.dac_lsb_i[0];
    let initial_ohm_max = parser.dc_ohm_max;

    parser.parse_sub_ch();

    assert.eq(parser.option_array[OptionSlot::CurrentMeasurementGain.index()], 20.0);
    assert.eq(parser.dac_lsb_i[0], initial_lsb / 2.0);
    assert.eq(parser.dc_ohm_max, initial_ohm_max * 2.0);
    assert.is_false(parser.ee_unlocked);

    parser.ser_inp_str = "SCL 2=2".to_owned();
    parser.ee_unlocked = true;
    parser.parse_sub_ch();

    assert.eq(parser.daci_scales[0], 2.0);
    assert.eq(parser.dac_lsb_i[0], initial_lsb / 4.0);
    assert.finish();
}
