// Best-effort Rust port of ctLab/Firmware/EDL/EDL-Parser.pas.
//
// This keeps the original parser split:
// - device-specific handlers: parse_get_param / parse_set_param
// - generic parser helpers: cmd_to_index / parse_extract / parse_sub_ch
//
// The original Pascal parser talks directly to firmware globals and serial I/O.
// This Rust version keeps the same control flow but represents hardware access
// through explicit state fields and placeholder hook methods.

#[path = "edl_parser/prompt_code.rs"]
mod prompt_code;
pub use prompt_code::PromptCode;

#[path = "edl_parser/mode.rs"]
mod mode;
pub use mode::Mode;

#[path = "edl_parser/modify.rs"]
mod modify;
pub use modify::Modify;

#[path = "edl_parser/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;

const OPTION_ARRAY_LEN: usize = 22;
const DACI_COUNT: usize = 4;
const ADCU_COUNT: usize = 2;
const ADCI_COUNT: usize = 4;
const SHUNT_D: u8 = 3;
const DEFAULT_DAC_MAX: u16 = 4095;
const DEFAULT_CMD_STR_ARR: [&str; 31] = [
    "STR",
    "IDN",
    "CHN",
    "VAL",
    "ENA",
    "DCA",
    "DCP",
    "DCV",
    "DCR",
    "MAH",
    "MWH",
    "MSV",
    "MSA",
    "RNG",
    "MSW",
    "PCA",
    "RON",
    "ROF",
    "RIP",
    "RAW",
    "DSP",
    "ALL",
    "OFS",
    "SCL",
    "OPT",
    "TMP",
    "TRM",
    "WEN",
    "ERC",
    "SBD",
    "NOP",
];
const DEFAULT_CMD2_SUB_CH_ARR: [u8; 31] = [
    255,
    254,
    253,
    0,
    0,
    1,
    3,
    4,
    5,
    7,
    8,
    10,
    11,
    19,
    18,
    21,
    27,
    28,
    29,
    50,
    80,
    99,
    100,
    200,
    150,
    233,
    240,
    250,
    251,
    252,
    0,
];
const DEFAULT_OPTION_ARRAY: [f64; OPTION_ARRAY_LEN] = [
    0.0, 0.02, 2.5, 10.0, 10.0, 2.5, 50.0, 100.0, 10.0, 1.0, 0.1, 0.002, 0.020, 0.200, 2.0, 25.0,
    6.1, 4.0, 0.0, 10.0, 0.0, 50.0,
];
const OPT_GAIN_I: usize = 4;
const OPT_U_REF: usize = 5;
const OPT_PMAX: usize = 6;
const OPT_RSENSE_BASE: usize = 7;
const OPT_IMAX_BASE: usize = 11;
const OPT_UMAX_HI: usize = 15;
const OPT_UMAX_LO: usize = 16;
const OPT_INIT_OPTIONS: usize = 17;
const ADC10_COUNT: usize = 6;
const ADC_MAX_10: f64 = 1023.0;
const ADC_MAX_16: f64 = 65535.0;

#[path = "edl_parser/edl_parser.rs"]
mod edl_parser;
pub use edl_parser::EdlParser;

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

        assert_eq!(parser.option_array[OPT_GAIN_I], 20.0);
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
