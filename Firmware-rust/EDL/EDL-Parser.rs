//! Serial command parser for the EDL electronic-load firmware.
//!
//! The split between device-specific getters/setters and generic channel,
//! mnemonic, token, and checksum parsing mirrors `EDL-Parser.pas`. Mutable
//! fields model the Pascal globals so protocol behavior can be tested without
//! silently replacing hardware-dependent operations.

/// Verbose parser outcome labels.
#[path = "edl_parser/prompt_code.rs"]
mod prompt_code;
pub use prompt_code::PromptCode;

/// Parser-side mode values retaining invalid wire bytes.
#[path = "edl_parser/mode.rs"]
mod mode;
pub use mode::Mode;

/// Parser-side menu values retaining invalid wire bytes.
#[path = "edl_parser/modify.rs"]
mod modify;
pub use modify::Modify;

/// Compiler-checked EDL mnemonic and base-subchannel mapping shared with the foreground state machine.
pub use crate::edl::CmdWhich;

/// Fixed EEPROM option positions shared with the foreground EDL state machine.
pub use crate::edl::OptionSlot;

const DACI_COUNT: usize = 4;
const ADCU_COUNT: usize = 2;
const ADCI_COUNT: usize = 4;
const SHUNT_D: u8 = 3;
const DEFAULT_DAC_MAX: u16 = 4095;
#[rustfmt::skip]
const DEFAULT_OPTION_ARRAY: [f64; OptionSlot::COUNT] = [
    0.0,
    0.02,
    2.5,
    10.0,
    10.0,
    2.5,
    50.0,
    100.0,
    10.0,
    1.0,
    0.1,
    0.002,
    0.020,
    0.200,
    2.0,
    25.0,
    6.1,
    4.0,
    0.0,
    10.0,
    0.0,
    50.0,
];
const ADC10_COUNT: usize = 6;
const ADC_MAX_10: f64 = 1023.0;
const ADC_MAX_16: f64 = 65535.0;

/// Source-faithful protocol state and command dispatch.
#[path = "edl_parser/edl_parser.rs"]
mod implementation;
pub use implementation::EdlParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_command_table_accepts_pascal_text_setter_syntax() {
        let mut parser = EdlParser {
            ser_inp_str: "DCA 0=1.25".to_owned(),
            slave_ch: 0,
            ..EdlParser::default()
        };

        let output = parser.parse_sub_ch();

        assert!(output.is_empty());
        assert_eq!(parser.sub_ch, 1);
        assert_eq!(parser.dc_amp, 1.25);
    }

    #[test]
    fn val_command_uses_pascal_offset_and_gets_same_channel_as_short_form() {
        let mut parser = EdlParser {
            ser_inp_str: "VAL 5?".to_owned(),
            dc_ohm: 123.4,
            slave_ch: 0,
            ..EdlParser::default()
        };

        let output = parser.parse_sub_ch();

        assert_eq!(parser.sub_ch, 5);
        assert_eq!(output, vec!["5=123.4"]);
    }

    #[test]
    fn zero_parameter_text_command_uses_command_offset_as_sub_channel() {
        let mut parser = EdlParser {
            ser_inp_str: "IDN?".to_owned(),
            vers1_str: "EDL test",
            slave_ch: 0,
            ..EdlParser::default()
        };

        let output = parser.parse_sub_ch();

        assert_eq!(parser.sub_ch, 254);
        assert_eq!(output, vec!["0:EDL test"]);
    }

    #[test]
    fn setters_enforce_pascal_limits_and_report_actual_limit_error_when_verbose() {
        let mut parser = EdlParser {
            ser_inp_str: "DCA 0=5!".to_owned(),
            slave_ch: 0,
            ..EdlParser::default()
        };

        let output = parser.parse_sub_ch();

        assert_eq!(parser.dc_amp, 2.0);
        assert_eq!(parser.check_limit_err, PromptCode::ParamErr);
        assert_eq!(output, vec!["ParamErr"]);
    }

    #[test]
    fn mode_numbers_match_pascal_and_drive_the_selected_dac_path() {
        let mut parser = EdlParser {
            ser_inp_str: "RNG 0=1".to_owned(),
            dc_amp: 1.0,
            dac_lsb_i: [0.5; DACI_COUNT],
            slave_ch: 0,
            ..EdlParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.mode_select, Mode::IhiVolt);
        assert!(parser.mode_mpx);
        assert!(parser.output_enable);
        assert!(parser.mpxena);
        assert_eq!(parser.dac_temp_on, 1638);
    }

    #[test]
    fn power_mode_setpoint_recomputes_current_and_current_dac() {
        let mut parser = EdlParser {
            ser_inp_str: "DCP 0=4".to_owned(),
            mode_select: Mode::PhiVolt,
            voltage_on: 2.0,
            dac_lsb_i: [0.5; DACI_COUNT],
            slave_ch: 0,
            ..EdlParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.dc_watt, 4.0);
        assert_eq!(parser.dc_amp, 2.0);
        assert!(parser.mode_mpx);
        assert_eq!(parser.dac_temp_on, 3276);
    }

    #[test]
    fn raw_adc10_reads_return_backing_samples() {
        let mut parser = EdlParser {
            ser_inp_str: "52?".to_owned(),
            adc10: [0, 0, 0, 777, 888, 0],
            slave_ch: 0,
            ..EdlParser::default()
        };

        let output = parser.parse_sub_ch();

        assert_eq!(output, vec!["52=777"]);

        parser.ser_inp_str = "53?".to_owned();
        let output = parser.parse_sub_ch();

        assert_eq!(output, vec!["53=888"]);
    }

    #[test]
    fn modify_writes_refresh_display_state_and_preserve_menu_value() {
        let mut parser = EdlParser {
            ser_inp_str: "DSP 0=8".to_owned(),
            slave_ch: 0,
            ..EdlParser::default()
        };

        parser.parse_sub_ch();

        assert_eq!(parser.modify, Modify::CapMenu);
        assert_eq!(parser.display_refresh_count, 1);

        parser.ser_inp_str = "DSP 0?".to_owned();
        let output = parser.parse_sub_ch();

        assert_eq!(output, vec!["80=8"]);
    }

    #[test]
    fn unlocked_calibration_and_option_writes_recalculate_scales() {
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

        assert_eq!(
            parser.option_array[OptionSlot::CurrentMeasurementGain.index()],
            20.0
        );
        assert_eq!(parser.dac_lsb_i[0], initial_lsb / 2.0);
        assert_eq!(parser.dc_ohm_max, initial_ohm_max * 2.0);
        assert!(!parser.ee_unlocked);

        parser.ser_inp_str = "SCL 2=2".to_owned();
        parser.ee_unlocked = true;
        parser.parse_sub_ch();

        assert_eq!(parser.daci_scales[0], 2.0);
        assert_eq!(parser.dac_lsb_i[0], initial_lsb / 4.0);
    }
}
