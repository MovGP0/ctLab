// Best-effort Rust port of ctLab/Firmware/DIV/DIV-Parser.pas.
//
// This file keeps the original parser structure and lookup tables readable,
// while moving board-specific I/O and ADC behavior behind a hook trait.

use crate::div::{DeviceState as DivDeviceState, DivHardware as DivRuntimeHardware, DivRange};

#[path = "div_parser/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;
#[path = "div_parser/parser_error.rs"]
mod parser_error;
pub use parser_error::ParserError;
#[path = "div_parser/parser_state.rs"]
mod parser_state;
pub use parser_state::ParserState;
#[path = "div_parser/div_parser_hooks.rs"]
mod div_parser_hooks;
pub use div_parser_hooks::DivParserHooks;
#[path = "div_parser/div_runtime_adapter.rs"]
mod div_runtime_adapter;
pub use div_runtime_adapter::DivRuntimeAdapter;
#[path = "div_parser/div_parser.rs"]
mod div_parser;
pub use div_parser::DivParser;


pub const VERS1_STR: &str = "3.10 [DIV by CM/c't 03/2007] ";

pub const CMD_STR_ARR: [&str; 16] = [
    "STR", "IDN", "TRG", "VAL", "RNG", "DSP", "OFS", "SCL", "ALL", "TRM", "TRT", "TRL", "ERC",
    "SBD", "WEN", "NOP",
];

pub const CMD_TO_SUBCH_ARR: [u8; 16] = [
    255, 254, 249, 0, 19, 80, 100, 200, 99, 240, 247, 248, 251, 252, 250, 0,
];

const COMMANDS: [CmdWhich; 16] = [
    CmdWhich::Str,
    CmdWhich::Idn,
    CmdWhich::Trg,
    CmdWhich::Val,
    CmdWhich::Rng,
    CmdWhich::Dsp,
    CmdWhich::Ofs,
    CmdWhich::Scl,
    CmdWhich::All,
    CmdWhich::Trm,
    CmdWhich::Trt,
    CmdWhich::Trl,
    CmdWhich::Erc,
    CmdWhich::Sbd,
    CmdWhich::Wen,
    CmdWhich::Nop,
];


fn parse_u8_default(value: &str, default: u8) -> u8 {
    value
        .trim()
        .parse::<i32>()
        .ok()
        .and_then(|parsed| {
            if (0..=u8::MAX as i32).contains(&parsed) {
                Some(parsed as u8)
            } else {
                None
            }
        })
        .unwrap_or(default)
}

fn parse_f32_default(value: &str, default: f32) -> f32 {
    value.trim().parse::<f32>().unwrap_or(default)
}

fn parse_hex_u8_default(value: &str, default: u8) -> u8 {
    u8::from_str_radix(value.trim(), 16).unwrap_or(default)
}

fn div_range_from_u8(value: u8) -> DivRange {
    match value {
        0 => DivRange::Dc250mV,
        1 => DivRange::Dc2V5,
        2 => DivRange::Dc25V,
        3 => DivRange::Dc250V,
        4 => DivRange::Ac250mV,
        5 => DivRange::Ac2V5,
        6 => DivRange::Ac25V,
        7 => DivRange::Ac250V,
        8 => DivRange::Dc250uA,
        9 => DivRange::Dc25mA,
        10 => DivRange::Dc2A5,
        11 => DivRange::Dc10A,
        12 => DivRange::Ac250uA,
        13 => DivRange::Ac25mA,
        14 => DivRange::Ac2A5,
        _ => DivRange::Ac10A,
    }
}

fn range_exponent_suffix(range: DivRange) -> Option<&'static str> {
    match range {
        DivRange::Dc250mV | DivRange::Ac250mV | DivRange::Dc25mA | DivRange::Ac25mA => Some("E-3"),
        DivRange::Dc250uA | DivRange::Ac250uA => Some("E-6"),
        _ => None,
    }
}

fn format_serial_param(value: f32) -> String {
    format!("{value:.6}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default)]
    struct MockHardware {
        serial: String,
        lcd_lines: Vec<(u8, String)>,
        last_range: Option<DivRange>,
        ad24: i32,
        ad10: [i16; 8],
        ad10_ready_clears: usize,
        ad24_ready_clears: usize,
    }

    impl DivRuntimeHardware for MockHardware {
        fn read_adc10(&mut self, channel_1_based: u8) -> i16 {
            self.ad10[channel_1_based as usize]
        }

        fn read_adc24(&mut self) -> i32 {
            self.ad24
        }

        fn read_adc24_fast_integrated(&mut self) -> i32 {
            self.ad24
        }

        fn read_adc24_slow_integrated(&mut self) -> i32 {
            self.ad24
        }

        fn adc24_overload_negative(&self) -> bool {
            false
        }

        fn adc24_overload_positive(&self) -> bool {
            false
        }

        fn clear_adc10_ready(&mut self) {
            self.ad10_ready_clears += 1;
        }

        fn adc10_ready(&mut self) -> bool {
            true
        }

        fn clear_adc24_ready(&mut self) {
            self.ad24_ready_clears += 1;
        }

        fn adc24_ready(&mut self) -> bool {
            true
        }

        fn set_range_config(&mut self, config: crate::div::RangeRelayConfig) {
            self.last_range = Some(config.range);
        }

        fn set_trigger_edge(&mut self, _positive_edge: bool) {}

        fn poll_serial_byte(&mut self) -> Option<u8> {
            None
        }

        fn serial_write(&mut self, text: &str) {
            self.serial.push_str(text);
        }

        fn lcd_write_line(&mut self, row: u8, text: &str) {
            self.lcd_lines.push((row, text.to_string()));
        }
    }

    fn run_frame(parser: &mut DivParser<DivRuntimeAdapter<'_, MockHardware>>, frame: &str) {
        parser.state.ser_inp_str = frame.to_string();
        parser.parse_sub_ch();
    }

    fn new_parser() -> DivParser<DivRuntimeAdapter<'static, MockHardware>> {
        let device = Box::new(DivDeviceState::new(MockHardware::default()));
        let leaked = Box::leak(device);
        let hooks = DivRuntimeAdapter::new(leaked);
        let mut parser = DivParser::new(hooks);
        parser.state.slave_ch = 1;
        parser.state.current_ch = 1;
        parser
    }

    #[test]
    fn busy_commands_fail_before_execution() {
        let mut parser = new_parser();
        parser.hooks.busy = true;

        run_frame(&mut parser, "1:RNG?");

        assert_eq!(parser.hooks.device.hw.serial, "#1:255=130 [BUSY]\r\n");
        assert_eq!(parser.hooks.activity_timer_ticks, None);
    }

    #[test]
    fn runtime_adapter_waits_use_device_irq_handshakes() {
        let mut parser = new_parser();

        parser.hooks.wait_ad10(&mut parser.state);
        parser.hooks.wait_ad24(&mut parser.state);

        assert_eq!(parser.hooks.device.hw.ad10_ready_clears, 1);
        assert_eq!(parser.hooks.device.hw.ad24_ready_clears, 1);
    }

    #[test]
    fn calibration_and_range_writes_hit_live_device_state() {
        let mut parser = new_parser();

        run_frame(&mut parser, "1:WEN=1");
        run_frame(&mut parser, "1:RNG=5");
        run_frame(&mut parser, "1:WEN=1");
        run_frame(&mut parser, "1:OFS 0=42");
        run_frame(&mut parser, "1:WEN=1");
        run_frame(&mut parser, "1:OFS 20=7");
        run_frame(&mut parser, "1:WEN=1");
        run_frame(&mut parser, "1:SCL 0=1.5");
        run_frame(&mut parser, "1:WEN=1");
        run_frame(&mut parser, "1:SCL 20=2.5");

        assert_eq!(parser.hooks.device.range, DivRange::Ac2V5);
        assert_eq!(parser.hooks.device.hw.last_range, Some(DivRange::Ac2V5));
        assert_eq!(parser.hooks.device.eeprom.ad24_offsets[0], 42);
        assert_eq!(parser.hooks.device.eeprom.ad10_offsets[0], 7);
        assert_eq!(parser.hooks.device.eeprom.ad24_scales[0], 1.5);
        assert_eq!(parser.hooks.device.eeprom.ad10_scales[0], 2.5);
        assert!(!parser.state.ee_unlocked);
    }

    #[test]
    fn trigger_commands_update_runtime_state() {
        let mut parser = new_parser();

        run_frame(&mut parser, "1:WEN=1");
        run_frame(&mut parser, "1:TRM=3");
        run_frame(&mut parser, "1:WEN=1");
        run_frame(&mut parser, "1:TRT=25");
        run_frame(&mut parser, "1:TRG?");

        assert_eq!(parser.hooks.device.eeprom.trigger_mode, 3);
        assert_eq!(parser.hooks.device.eeprom.trigger_timer_ms, 25);
        assert!(parser.hooks.device.trigger_pending);
        assert!(parser.hooks.device.hw.serial.ends_with("#1:255=0 [OK]\r\n"));
    }

    #[test]
    fn forwarded_frames_preserve_pascal_wire_format() {
        let mut parser = new_parser();

        run_frame(&mut parser, "#2:19=5");
        run_frame(&mut parser, "2:IDN?");

        assert_eq!(parser.hooks.device.hw.serial, "#2:19=5\r\n2:IDN?\r\n");
    }

    #[test]
    fn replies_use_prefixed_pascal_framing() {
        let mut parser = new_parser();

        run_frame(&mut parser, "1:IDN?");
        run_frame(&mut parser, "1:RNG?");

        assert_eq!(
            parser.hooks.device.hw.serial,
            "#1:254=3.10 [DIV by CM/c't 03/2007] \r\n#1:19=1\r\n"
        );
    }
}
