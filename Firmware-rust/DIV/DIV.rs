//! Best-effort Rust port of `DIV.pas`.
//!
//! This preserves the original digital voltmeter structure: range tables,
//! calibration storage, ADC conversion helpers, display/serial formatting, and
//! a polling-style service loop.

#![allow(dead_code)]

pub type Float = f32;

pub const PROC_CLOCK: u32 = 16_000_000;
pub const VERS1_STR: &str = "3.10 [DIV by CM/c't 03/2007]";
pub const VERS3_STR: &str = "DIV 3.10";

pub const PORT_A_INIT: u8 = 0b0000_0011;
pub const PORT_C_INIT: u8 = 0b0000_0011;
pub const ADC24_MID_SCALE: i32 = 0x800000;
pub const EE_INITIALISED_MAGIC: u16 = 0xAA55;
pub const OFFSET_INITIALISED_MAGIC: u16 = 0xAA55;
pub const ERR_SUB_CH: u8 = 255;

pub const RANGE_STR_ARR: [&str; 16] = [
    "DC 250mV",
    "DC  2.5V",
    "DC   25V",
    "DC  250V",
    "AC 250mV",
    "AC  2.5V",
    "AC   25V",
    "AC  250V",
    "DC 250uA",
    "DC  25mA",
    "DC  2.5A",
    "DC   10A",
    "AC 250uA",
    "AC  25mA",
    "AC  2.5A",
    "AC   10A",
];

pub const DIGITS_ARR: [u8; 16] = [3, 1, 2, 3, 3, 1, 2, 3, 3, 2, 1, 1, 3, 2, 1, 1];
pub const NACHKOMMA_ARR: [u8; 16] = [3, 5, 4, 3, 3, 5, 4, 3, 3, 4, 5, 5, 3, 4, 5, 5];

pub const RANGE_ARR_PORT_A: [u8; 16] = [
    0b0000_0000 | PORT_A_INIT,
    0b0000_0000 | PORT_A_INIT,
    0b0010_0000 | PORT_A_INIT,
    0b0010_0000 | PORT_A_INIT,
    0b0000_0000 | PORT_A_INIT,
    0b0000_0000 | PORT_A_INIT,
    0b0000_0000 | PORT_A_INIT,
    0b0000_0000 | PORT_A_INIT,
    0b0000_0000 | PORT_A_INIT,
    0b0100_0000 | PORT_A_INIT,
    0b1000_0000 | PORT_A_INIT,
    0b1000_0000 | PORT_A_INIT,
    0b0000_0000 | PORT_A_INIT,
    0b0100_0000 | PORT_A_INIT,
    0b1000_0000 | PORT_A_INIT,
    0b1000_0000 | PORT_A_INIT,
];

pub const RANGE_ARR_PORT_C: [u8; 16] = [
    0b0000_0000 | PORT_C_INIT,
    0b0000_0000 | PORT_C_INIT,
    0b0001_0000 | PORT_C_INIT,
    0b0010_0000 | PORT_C_INIT,
    0b0100_0100 | PORT_C_INIT,
    0b0100_0000 | PORT_C_INIT,
    0b0100_1100 | PORT_C_INIT,
    0b0100_1000 | PORT_C_INIT,
    0b1000_0000 | PORT_C_INIT,
    0b1000_0000 | PORT_C_INIT,
    0b1000_0000 | PORT_C_INIT,
    0b1000_0000 | PORT_C_INIT,
    0b1100_0100 | PORT_C_INIT,
    0b1100_0100 | PORT_C_INIT,
    0b1100_0100 | PORT_C_INIT,
    0b1100_0100 | PORT_C_INIT,
];

pub const RANGE_ARRAY_24: [Float; 16] = [
    250.0 / 8_388_608.0,
    2.5 / 8_388_608.0,
    25.0 / 8_388_608.0,
    250.0 / 8_388_608.0,
    250.0 / 8_388_608.0,
    2.5 / 8_388_608.0,
    25.0 / 8_388_608.0,
    250.0 / 8_388_608.0,
    250.0 / 8_388_608.0,
    25.0 / 8_388_608.0,
    2.5 / 8_388_608.0,
    2.5 / 8_388_608.0,
    250.0 / 8_388_608.0,
    25.0 / 8_388_608.0,
    2.5 / 8_388_608.0,
    2.5 / 8_388_608.0,
];

pub const RANGE_ARRAY_10: [Float; 16] = [
    250.0 / 512.0,
    2.5 / 512.0,
    25.0 / 512.0,
    250.0 / 512.0,
    250.0 / 1024.0,
    25.0 / 1024.0,
    2.5 / 1024.0,
    2.5 / 1024.0,
    250.0 / 512.0,
    25.0 / 512.0,
    2.5 / 512.0,
    2.5 / 512.0,
    250.0 / 1024.0,
    25.0 / 1024.0,
    2.5 / 1024.0,
    2.5 / 1024.0,
];

pub const CMD_STR_ARR: [&str; 16] = [
    "STR",
    "IDN",
    "TRG",
    "VAL",
    "RNG",
    "DSP",
    "OFS",
    "SCL",
    "ALL",
    "TRM",
    "TRT",
    "TRL",
    "ERC",
    "SBD",
    "WEN",
    "NOP",
];

pub const ERR_STR_ARR: [&str; 8] = [
    "[OK]",
    "[SRQUSR]",
    "[BUSY]",
    "[OVRLD]",
    "[CMDERR]",
    "[PARERR]",
    "[LOCKED]",
    "[CHKSUM]",
];

pub const FAULT_STR_ARR: [&str; 4] = [
    "[OVRNEG]",
    "[OVRPOS]",
    "[]",
    "[]"
];

#[path = "div/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;
#[path = "div/error_code.rs"]
mod error_code;
pub use error_code::ErrorCode;
#[path = "div/div_range.rs"]
mod div_range;
pub use div_range::DivRange;
#[path = "div/range_relay_config.rs"]
mod range_relay_config;
pub use range_relay_config::RangeRelayConfig;
#[path = "div/eeprom_data.rs"]
mod eeprom_data;
pub use eeprom_data::EepromData;
#[path = "div/div_hardware.rs"]
mod div_hardware;
pub use div_hardware::DivHardware;
#[path = "div/device_state.rs"]
mod device_state;
pub use device_state::DeviceState;


fn div_range_from_u8(value: u8) -> DivRange {
    limit_raw_range(value).0
}

fn limit_raw_range(value: u8) -> (DivRange, bool) {
    let limited = value > 15;
    let value = if value > 127 {
        0
    } else if value > 15 {
        15
    } else {
        value
    };

    match value {
        0 => (DivRange::Dc250mV, limited),
        1 => (DivRange::Dc2V5, limited),
        2 => (DivRange::Dc25V, limited),
        3 => (DivRange::Dc250V, limited),
        4 => (DivRange::Ac250mV, limited),
        5 => (DivRange::Ac2V5, limited),
        6 => (DivRange::Ac25V, limited),
        7 => (DivRange::Ac250V, limited),
        8 => (DivRange::Dc250uA, limited),
        9 => (DivRange::Dc25mA, limited),
        10 => (DivRange::Dc2A5, limited),
        11 => (DivRange::Dc10A, limited),
        12 => (DivRange::Ac250uA, limited),
        13 => (DivRange::Ac25mA, limited),
        14 => (DivRange::Ac2A5, limited),
        _ => (DivRange::Ac10A, limited),
    }
}

pub fn range_exponent_suffix(range: DivRange) -> Option<&'static str> {
    match range {
        DivRange::Dc250mV | DivRange::Ac250mV | DivRange::Dc25mA | DivRange::Ac25mA => Some("E-3"),
        DivRange::Dc250uA | DivRange::Ac250uA => Some("E-6"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, Clone, Default)]
    struct MockHardware {
        ad10: i16,
        ad24: i32,
        ad24_fast: i32,
        ad24_slow: i32,
        adc24_overload_negative: bool,
        adc24_overload_positive: bool,
        rx: VecDeque<u8>,
        range_configs: Vec<RangeRelayConfig>,
        trigger_edges: Vec<bool>,
        ad10_ready_polls_before_ready: usize,
        ad24_ready_polls_before_ready: usize,
        ad10_ready_polls: usize,
        ad24_ready_polls: usize,
        readiness_events: Vec<&'static str>,
        serial: String,
        lcd_lines: Vec<(u8, String)>,
    }

    impl DivHardware for MockHardware {
        fn read_adc10(&mut self, _channel_1_based: u8) -> i16 {
            self.ad10
        }

        fn read_adc24(&mut self) -> i32 {
            self.ad24
        }

        fn read_adc24_fast_integrated(&mut self) -> i32 {
            self.ad24_fast
        }

        fn read_adc24_slow_integrated(&mut self) -> i32 {
            self.ad24_slow
        }

        fn adc24_overload_negative(&self) -> bool {
            self.adc24_overload_negative
        }

        fn adc24_overload_positive(&self) -> bool {
            self.adc24_overload_positive
        }

        fn clear_adc10_ready(&mut self) {
            self.ad10_ready_polls = 0;
            self.readiness_events.push("clear-ad10");
        }

        fn adc10_ready(&mut self) -> bool {
            self.ad10_ready_polls += 1;
            self.readiness_events.push("poll-ad10");
            self.ad10_ready_polls > self.ad10_ready_polls_before_ready
        }

        fn clear_adc24_ready(&mut self) {
            self.ad24_ready_polls = 0;
            self.readiness_events.push("clear-ad24");
        }

        fn adc24_ready(&mut self) -> bool {
            self.ad24_ready_polls += 1;
            self.readiness_events.push("poll-ad24");
            self.ad24_ready_polls > self.ad24_ready_polls_before_ready
        }

        fn set_range_config(&mut self, config: RangeRelayConfig) {
            self.range_configs.push(config);
        }

        fn set_trigger_edge(&mut self, positive_edge: bool) {
            self.trigger_edges.push(positive_edge);
        }

        fn poll_serial_byte(&mut self) -> Option<u8> {
            self.rx.pop_front()
        }

        fn serial_write(&mut self, text: &str) {
            self.serial.push_str(text);
        }

        fn lcd_write_line(&mut self, row: u8, text: &str) {
            self.lcd_lines.push((row, text.to_string()));
        }
    }

    fn assert_close(actual: Float, expected: Float) {
        assert!(
            (actual - expected).abs() < 0.00001,
            "expected {expected}, got {actual}"
        );
    }

    fn serial_rx(text: &str) -> VecDeque<u8> {
        text.as_bytes().iter().copied().collect()
    }

    #[test]
    fn switch_range_applies_pascal_relay_gain_and_display_tables() {
        let mut state = DeviceState::new(MockHardware::default());

        state.switch_range(DivRange::Ac25V);

        let config = state.hw.range_configs.last().copied().unwrap();
        assert_eq!(config.port_a, 0b0000_0011);
        assert_eq!(config.port_c, 0b0100_1111);
        assert!(!config.dc_gain_10);
        assert_eq!(config.digits, 2);
        assert_eq!(config.decimals, 4);
        assert_eq!(state.integrate_24_fast, i64::from(ADC24_MID_SCALE));
        assert_eq!(state.integrate_24_slow, i64::from(ADC24_MID_SCALE));

        state.switch_range(DivRange::Dc10A);
        let config = state.hw.range_configs.last().copied().unwrap();
        assert_eq!(config.port_a, 0b1000_0011);
        assert_eq!(config.port_c, 0b1000_0011);
        assert!(config.dc_gain_10);
    }

    #[test]
    fn wait_ad10_clears_stale_flag_and_waits_for_next_irq_update() {
        let mut state = DeviceState::new(MockHardware {
            ad10_ready_polls_before_ready: 2,
            ..MockHardware::default()
        });

        state.wait_ad10();

        assert_eq!(state.hw.ad10_ready_polls, 3);
        assert_eq!(
            state.hw.readiness_events,
            vec!["clear-ad10", "poll-ad10", "poll-ad10", "poll-ad10"]
        );
    }

    #[test]
    fn wait_ad24_clears_stale_flag_and_waits_for_next_irq_update() {
        let mut state = DeviceState::new(MockHardware {
            ad24_ready_polls_before_ready: 1,
            ..MockHardware::default()
        });

        state.wait_ad24();

        assert_eq!(state.hw.ad24_ready_polls, 2);
        assert_eq!(
            state.hw.readiness_events,
            vec!["clear-ad24", "poll-ad24", "poll-ad24"]
        );
    }

    #[test]
    fn scaling_uses_pascal_per_range_factors_and_calibration_scales() {
        let mut state = DeviceState::new(MockHardware {
            ad10: 612,
            ad24: ADC24_MID_SCALE + 1000,
            ad24_fast: ADC24_MID_SCALE,
            ad24_slow: ADC24_MID_SCALE,
            ..MockHardware::default()
        });
        state.eeprom.ad24_scales[DivRange::Dc25V as usize] = 2.0;
        state.eeprom.ad10_scales[DivRange::Dc25V as usize] = 0.5;
        state.eeprom.ad24_offsets[DivRange::Dc25V as usize] = 10;
        state.eeprom.ad10_offsets[DivRange::Dc25V as usize] = 2;

        state.switch_range(DivRange::Dc25V);
        state.get_ad24(0);
        state.get_ad10(5);

        assert_close(state.measured_value, 1010.0 * (25.0 / 8_388_608.0) * 2.0);
        assert_close(state.measured_aux, 102.0 * (25.0 / 512.0) * 0.5);

        state.switch_range(DivRange::Ac2V5);
        assert_close(state.param_scale_24(-1000), 1000.0 * (2.5 / 8_388_608.0));
    }

    #[test]
    fn display_and_serial_format_follow_range_tables() {
        let mut state = DeviceState::new(MockHardware::default());
        state.switch_range(DivRange::Dc250mV);
        state.measured_value = 0.01234;

        assert_eq!(state.param_to_str(true), "+000.012");
        assert_eq!(state.param_to_str(false), "0.01234");

        state.switch_range(DivRange::Ac25V);
        state.measured_value = 1.234567;

        assert_eq!(state.param_to_str(true), "\x0301.2346");
        assert_eq!(state.param_to_str(false), "1.234567");
    }

    #[test]
    fn trigger_edges_timer_and_mask_select_pascal_subchannels() {
        let mut state = DeviceState::new(MockHardware::default());
        state.eeprom.trigger_mode = 0b0000_0111;
        state.set_trigger_edge_level(true);

        state.int2_trigger_edge(false);
        assert!(!state.trigger_pending);

        state.int2_trigger_edge(true);
        assert_eq!(state.service_trigger(), &[0, 10, 11]);
        assert!(!state.trigger_pending);

        state.eeprom.trigger_mode = 0b0000_0010;
        state.eeprom.trigger_timer_ms = 25;
        state.tick_auto_trigger(24);
        assert!(!state.trigger_pending);
        state.tick_auto_trigger(1);
        assert_eq!(state.service_trigger(), &[10]);
        assert_eq!(state.hw.trigger_edges, vec![true]);
    }

    #[test]
    fn ad24_integration_mode_selects_pascal_sources_and_fault_flags() {
        let mut state = DeviceState::new(MockHardware {
            ad24: ADC24_MID_SCALE + 100,
            ad24_fast: ADC24_MID_SCALE + 200,
            ad24_slow: ADC24_MID_SCALE + 300,
            adc24_overload_negative: true,
            ..MockHardware::default()
        });

        state.switch_range(DivRange::Dc2V5);
        state.get_ad24(0);
        assert_close(state.measured_value, 100.0 * (2.5 / 8_388_608.0));

        state.get_ad24(1);
        assert_close(state.measured_value, 200.0 * (2.5 / 8_388_608.0));

        state.get_ad24(2);
        assert_close(state.measured_value, 300.0 * (2.5 / 8_388_608.0));
        assert_eq!(state.fault_flags(), 0b0000_0001);
        assert!(state.overload_flag());
    }

    #[test]
    fn status_prompt_uses_pascal_prefix_status_byte_and_fault_labels() {
        let mut state = DeviceState::new(MockHardware::default());
        state.slave_ch = 2;
        state.busy_flag = true;
        state.ee_unlocked = true;

        state.ser_prompt(ErrorCode::BusyErr);
        assert_eq!(state.hw.serial, "#2:255=146 [BUSY]\r\n");
        assert_eq!(state.err_count, 1);

        state.hw.serial.clear();
        state.busy_flag = false;
        state.ee_unlocked = false;
        state.overload_negative = true;
        state.overload_positive = true;

        state.ser_prompt(ErrorCode::OvlErr);
        assert_eq!(state.hw.serial, "#2:255=35 [OVRNEG] [OVRPOS]\r\n");
    }

    #[test]
    fn check_limits_clamps_pascal_byte_range_and_reports_param_error() {
        let mut state = DeviceState::new(MockHardware::default());

        assert!(!state.check_limits_raw_range(15));
        assert_eq!(state.range, DivRange::Ac10A);
        assert_eq!(state.check_limit_err, ErrorCode::NoErr);

        assert!(state.check_limits_raw_range(16));
        assert_eq!(state.range, DivRange::Ac10A);
        assert_eq!(state.check_limit_err, ErrorCode::ParamErr);

        assert!(state.check_limits_raw_range(255));
        assert_eq!(state.range, DivRange::Dc250mV);
        assert_eq!(state.check_limit_err, ErrorCode::ParamErr);
    }

    #[test]
    fn rng_set_uses_pascal_check_limits_before_switching_range() {
        let mut state = DeviceState::new(MockHardware {
            rx: serial_rx("0:RNG=16\r0:RNG=255\r"),
            ..MockHardware::default()
        });
        state.slave_ch = 0;

        state.check_ser();

        assert_eq!(state.range, DivRange::Dc250mV);
        assert_eq!(state.hw.range_configs[0].range, DivRange::Ac10A);
        assert_eq!(state.hw.range_configs[1].range, DivRange::Dc250mV);
        assert_eq!(
            state.hw.serial,
            "#0:255=5 [PARERR]\r\n#0:255=5 [PARERR]\r\n"
        );
    }

    #[test]
    fn init_all_restores_eeprom_defaults_and_initialises_zero_offsets() {
        let mut state = DeviceState::new(MockHardware {
            ad24_fast: ADC24_MID_SCALE + 123,
            ..MockHardware::default()
        });
        state.eeprom.init_range = DivRange::Ac10A;
        state.eeprom.init_lcd_integrate = 2;
        state.eeprom.init_inc_rast = 7;
        state.eeprom.trigger_edge_level = true;
        state.eeprom.offset_initialised = 0;

        state.init_all();

        assert_eq!(state.range, DivRange::Ac10A);
        assert_eq!(state.lcd_integrate, 2);
        assert_eq!(state.inc_rast, 7);
        assert_eq!(state.eeprom.ad24_offsets, [-123; 16]);
        assert_eq!(state.eeprom.offset_initialised, OFFSET_INITIALISED_MAGIC);
        assert_eq!(state.hw.trigger_edges, vec![true]);
        assert!(state
            .hw
            .serial
            .contains("#0:254=3.10 [DIV by CM/c't 03/2007]"));
        assert!(state.hw.serial.contains("#0:255=130 [BUSY]\r\n"));
        assert!(state.hw.serial.ends_with("#0:255=0 [OK]\r\n"));
    }

    #[test]
    fn check_ser_buffers_ascii_handles_backspace_and_parses_cr_frames() {
        let mut state = DeviceState::new(MockHardware {
            rx: serial_rx("0:RNG=5x\x08\r0:RNG?\r#9:19=3\r"),
            ..MockHardware::default()
        });
        state.slave_ch = 0;

        state.check_ser();

        assert_eq!(state.range, DivRange::Ac2V5);
        assert_eq!(state.hw.serial, "#0:19=5\r\n#9:19=3\r\n");
        assert!(state.ser_input.is_empty());
    }
}
