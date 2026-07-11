//! Best-effort Rust port of `DDS.pas`.
//!
//! The Pascal firmware mixes parser handling, panel state, serial framing,
//! measurement range control, and DDS output control in one unit. This Rust
//! version keeps that single-state-machine shape, but expresses the AVR-facing
//! parts through an explicit hardware trait.

#![allow(dead_code)]

pub type Float = f32;

pub const PROC_CLOCK: u32 = 16_000_000;
pub const VERS1_STR: &str = "3.71 [DDS by CM/c't 03/2007]";
pub const VERS3_STR: &str = "DDS 3.71";
pub const ADR_STR: &str = "Adr ";
pub const EE_NOT_PROGRAMMED_STR: &str = "EEPROM EMPTY! ";

pub const CMD_STR_ARR: [&str; 22] = [
    "STR",
    "IDN",
    "TRG",
    "VAL",
    "FRQ",
    "LVL",
    "LVP",
    "DBU",
    "WAV",
    "BST",
    "AUX",
    "INL",
    "RNG",
    "DCO",
    "DSP",
    "ALL",
    "OPT",
    "SCL",
    "WEN",
    "ERC",
    "SBD",
    "NOP",
];

pub const CMD2_SUB_CH_ARR: [u8; 22] = [
    255,
    254,
    249,
    0,
    0,
    1,
    2,
    3,
    4,
    5,
    9,
    10,
    19,
    20,
    80,
    99,
    150,
    200,
    250,
    251,
    252,
    0,
];

const ERR_SUB_CH: u8 = 255;
const EEPROM_INITIALIZED: u16 = 0xAA55;
const MAX_FREQUENCY_TENTHS_HZ: i32 = 9_999_999;
const MAX_OFFSET_MV: i32 = 10_000;
const DAC_LEVEL_MAX: Float = 4_000.0;
const MIN_DB: Float = -70.0;
const DB_REFERENCE_MV: Float = 774.597;
const TRIANGLE_RMS_FACTOR: Float = 0.816_496;
const TRIANGLE_DAC_FACTOR: Float = 1.224_745;
const SQUARE_RMS_FACTOR: Float = 1.414_21;
const SQUARE_DAC_FACTOR: Float = 0.707_11;
const PEAK_FACTOR: Float = 2.828_427_1;
const DDS_FACTORS: [u32; 8] = [
    64_000_000,
    6_400_000,
    640_000,
    64_000,
    6_400,
    640,
    64,
    6,
];
const INP_GAINS: [Float; 4] = [
    0.1,
    1.0,
    10.0,
    100.0,
];
const INCR_ACC_ARRAY: [i32; 16] = [
    0,
    1,
    5,
    10,
    25,
    50,
    100,
    250,
    500,
    1_000,
    2_500,
    5_000,
    10_000,
    25_000,
    25_000,
    25_000,
];
const TERZ_ARRAY: [i32; 32] = [
    200,
    250,
    315,
    400,
    500,
    630,
    800,
    1000,
    1250,
    1600,
    2000,
    2500,
    3150,
    4000,
    5000,
    6300,
    8000,
    10000,
    12500,
    16000,
    20000,
    25000,
    31500,
    40000,
    50000,
    63000,
    80000,
    100000,
    125000,
    160000,
    200000,
    0,
];
const LCD_CHARSET_0: [u8; 8] = [
    0x01,
    0x03,
    0x07,
    0x0f,
    0x07,
    0x03,
    0x01,
    0x00,
];
const LCD_CHARSET_1: [u8; 8] = [
    0x01,
    0x03,
    0x05,
    0x09,
    0x05,
    0x03,
    0x01,
    0x00,
];
const LCD_CHARSET_2: [u8; 8] = [
    0x01,
    0x02,
    0x05,
    0x0a,
    0x05,
    0x02,
    0x01,
    0x00,
];

const ERR_LABELS: [&str; 8] = [
    "[OK]",
    "[SRQUSR]",
    "[BUSY]",
    "[OVERLD]",
    "[CMDERR]",
    "[PARERR]",
    "[LOCKED]",
    "[CHKSUM]",
];

#[path = "dds/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;

#[path = "dds/modify.rs"]
mod modify;
pub use modify::Modify;

#[path = "dds/error_code.rs"]
mod error_code;
pub use error_code::ErrorCode;

#[path = "dds/waveform.rs"]
mod waveform;
pub use waveform::Waveform;

#[path = "dds/input_range.rs"]
mod input_range;
pub use input_range::InputRange;

#[path = "dds/runtime_status.rs"]
mod runtime_status;
pub use runtime_status::RuntimeStatus;

#[path = "dds/eeprom_data.rs"]
mod eeprom_data;
pub use eeprom_data::EepromData;

#[path = "dds/dds_hardware.rs"]
mod dds_hardware;
pub use dds_hardware::DdsHardware;

#[path = "dds/panel_event.rs"]
mod panel_event;
pub use panel_event::PanelEvent;

#[path = "dds/device_state.rs"]
mod device_state;
pub use device_state::DeviceState;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Debug, Default, Clone)]
    struct MockHardware {
        serial_out: String,
        serial_in: VecDeque<char>,
        waveforms: Vec<Waveform>,
        ranges: Vec<InputRange>,
        aux_configs: Vec<u8>,
        frequency_words: Vec<u32>,
        amplitude_words: Vec<u16>,
        lcd_lines: Vec<(u8, String)>,
        lcd_custom_chars: Vec<(u8, [u8; 8])>,
        lcd_setup_result: bool,
        serial_baud_calls: Vec<(u8, bool)>,
        activity_led_states: Vec<bool>,
        delay_calls: Vec<u16>,
        slave_channel: u8,
        input_level_mv: Float,
        input_overload: bool,
    }

    impl MockHardware {
        fn take_serial_output(&mut self) -> String {
            std::mem::take(&mut self.serial_out)
        }

        fn push_serial(&mut self, text: &str) {
            self.serial_in.extend(text.chars());
        }
    }

    impl DdsHardware for MockHardware {
        fn send_dds_frequency_word(&mut self, word: u32) {
            self.frequency_words.push(word);
        }

        fn send_amplitude_word(&mut self, word: u16) {
            self.amplitude_words.push(word);
        }

        fn set_waveform(&mut self, waveform: Waveform) {
            self.waveforms.push(waveform);
        }

        fn set_input_range(&mut self, range: InputRange) {
            self.ranges.push(range);
        }

        fn send_aux_config(&mut self, value: u8) {
            self.aux_configs.push(value);
        }

        fn read_input_level(&mut self) -> Float {
            self.input_level_mv
        }

        fn read_input_overload(&mut self) -> bool {
            self.input_overload
        }

        fn serial_write(&mut self, text: &str) {
            self.serial_out.push_str(text);
        }

        fn serial_read(&mut self) -> Option<char> {
            self.serial_in.pop_front()
        }

        fn set_serial_baud_register(&mut self, register: u8, double_speed: bool) {
            self.serial_baud_calls.push((register, double_speed));
        }

        fn read_slave_channel(&mut self) -> u8 {
            self.slave_channel
        }

        fn set_activity_led(&mut self, enabled: bool) {
            self.activity_led_states.push(enabled);
        }

        fn delay_ms(&mut self, milliseconds: u16) {
            self.delay_calls.push(milliseconds);
        }

        fn lcd_setup(&mut self) -> bool {
            self.lcd_setup_result
        }

        fn lcd_define_custom_char(&mut self, slot: u8, bitmap: [u8; 8]) {
            self.lcd_custom_chars.push((slot, bitmap));
        }

        fn lcd_write_line(&mut self, row: u8, text: &str) {
            self.lcd_lines.push((row, text.to_string()));
        }
    }

    fn xor_checksum(text: &str) -> String {
        format!("{:02X}", text.bytes().fold(0_u8, |acc, byte| acc ^ byte))
    }

    #[test]
    fn frequency_protocol_uses_tenths_hz_on_the_wire() {
        let mut state = DeviceState::new(MockHardware::default());

        state.process_serial_command("FRQ=1234.5!");
        assert_eq!(state.frequency_tenths_hz, 12_345);
        assert_eq!(state.hw.take_serial_output(), "#0:255=0 [OK]\r\n");

        state.process_serial_command("FRQ");
        assert_eq!(state.hw.take_serial_output(), "#0:0=1234.5\r\n");
    }

    #[test]
    fn tuning_word_uses_fixed_decimal_decades_without_formatting_storage() {
        let mut state = DeviceState::new(MockHardware::default());

        state.frequency_tenths_hz = 10_000;
        assert_eq!(state.dds_tuning_word(), 64_000);

        state.frequency_tenths_hz = 12_345_670;
        assert_eq!(state.dds_tuning_word(), 79_012_288);
    }

    #[test]
    fn burst_mode_gates_waveform_on_systick() {
        let mut state = DeviceState::new(MockHardware::default());
        state.waveform = Waveform::Square;
        state.burst_mode = 3;
        state.burst_count = 1;
        state.burst_gate_open = true;

        state.on_sys_tick();
        assert_eq!(state.hw.waveforms.last(), Some(&Waveform::Square));

        state.on_sys_tick();
        assert_eq!(state.hw.waveforms.last(), Some(&Waveform::Off));

        state.on_sys_tick();
        state.on_sys_tick();
        state.on_sys_tick();
        assert_eq!(state.hw.waveforms.last(), Some(&Waveform::Square));
    }

    #[test]
    fn waveform_external_selection_preserves_external_index() {
        let mut state = DeviceState::new(MockHardware::default());

        state.process_serial_command("WAV=7!");
        assert_eq!(state.waveform, Waveform::External(2));
        assert_eq!(state.hw.aux_configs.last(), Some(&2));
        assert_eq!(state.hw.waveforms.last(), Some(&Waveform::External(2)));
        assert_eq!(state.hw.take_serial_output(), "#0:255=0 [OK]\r\n");

        state.process_serial_command("WAV");
        assert_eq!(state.hw.take_serial_output(), "#0:4=7\r\n");
    }

    #[test]
    fn calibration_semantics_follow_pascal_defaults() {
        let mut state = DeviceState::new(MockHardware::default());

        assert!((state.dac_level_to_rms(state.dac_level) - 774.6).abs() < 0.2);
        assert!(state.level_to_db(774.597).abs() < 0.01);

        state.waveform = Waveform::Triangle;
        let triangle_dac = state.rms_to_dac_level(774.597);
        assert!((state.dac_level_to_rms(triangle_dac) - 774.597).abs() < 0.5);

        state.process_serial_command("WAV=4!");
        assert!((state.dac_level_to_peak_mv() - state.eeprom.init_logic_level_mv).abs() < 0.5);
    }

    #[test]
    fn range_control_is_explicit_input_range_not_output_bucket() {
        let mut state = DeviceState::new(MockHardware::default());

        state.process_serial_command("RNG=2!");
        assert_eq!(state.range, InputRange::Ac10V);
        assert_eq!(state.hw.ranges.last(), Some(&InputRange::Ac10V));
        state.hw.take_serial_output();

        state.process_serial_command("LVL=10.0!");
        assert_eq!(state.range, InputRange::Ac10V);
        assert_eq!(state.hw.ranges.last(), Some(&InputRange::Ac10V));
    }

    #[test]
    fn parser_supports_numeric_subchannels_omni_and_checksum() {
        let mut state = DeviceState::new(MockHardware::default());
        let raw = "*:0=4321.1!";
        let checksum = xor_checksum(raw);
        let framed = format!("{raw}${checksum}");

        state.process_serial_command(&framed);
        assert_eq!(state.frequency_tenths_hz, 43_211);
        assert_eq!(
            state.hw.take_serial_output(),
            format!("{raw}${checksum}\r\n#0:255=0 [OK]\r\n")
        );
    }

    #[test]
    fn serial_framing_matches_pascal_verbose_rules() {
        let mut state = DeviceState::new(MockHardware::default());

        state.process_serial_command("FRQ=1000.0");
        assert_eq!(state.hw.take_serial_output(), "");

        state.process_serial_command("STR?");
        assert_eq!(state.hw.take_serial_output(), "#0:255=0 [OK]\r\n");

        state.process_serial_command("#1:0=1234.5");
        assert_eq!(state.hw.take_serial_output(), "#1:0=1234.5\r\n");
    }

    #[test]
    fn serial_receive_loop_parses_backspaced_lines() {
        let mut state = DeviceState::new(MockHardware::default());
        state.hw.push_serial("FRQ=1000.6\u{08}5!\r");

        state.check_ser();

        assert_eq!(state.frequency_tenths_hz, 10_005);
        assert_eq!(state.hw.take_serial_output(), "#0:255=0 [OK]\r\n");
    }

    #[test]
    fn panel_loop_restores_coarse_fine_frequency_and_busy_semantics() {
        let mut state = DeviceState::new(MockHardware::default());
        state.inc_rast = 2;
        state.lcd_present = true;

        state.handle_panel_event(PanelEvent::EncoderDelta(2));
        assert_eq!(state.frequency_tenths_hz, 12_500);
        assert!(state.status.busy_flag);
        assert_eq!(
            state.hw.take_serial_output(),
            "#0:255=67 [OK]\r\n#0:0=1250.0\r\n"
        );

        state.process_serial_command("FRQ");
        assert_eq!(state.hw.take_serial_output(), "#0:0=1250.0\r\n");

        state.process_serial_command("FRQ=1300.0!");
        assert_eq!(state.frequency_tenths_hz, 12_500);
        assert_eq!(state.hw.take_serial_output(), "#0:255=2 [BUSY]\r\n");

        state.handle_panel_event(PanelEvent::Buttons {
            enter: true,
            left: false,
            right: false,
        });
        assert!(state.incr_fine);
        assert_eq!(state.hw.take_serial_output(), "#0:255=67 [OK]\r\n");

        state.handle_panel_event(PanelEvent::IncrTimerElapsed);
        assert_eq!(state.hw.take_serial_output(), "#0:255=64 [OK]\r\n");

        state.frequency_tenths_hz = 12_345;
        state.first_turn = true;
        state.handle_panel_event(PanelEvent::EncoderDelta(2));
        assert_eq!(state.frequency_tenths_hz, 12_350);
        assert_eq!(
            state.hw.take_serial_output(),
            "#0:255=67 [OK]\r\n#0:0=1235.0\r\n"
        );

        state.handle_panel_event(PanelEvent::DisplayTimerElapsed);
        assert!(!state.status.busy_flag);
        assert!(!state.incr_fine);
    }

    #[test]
    fn panel_loop_restores_amplitude_wave_and_service_transitions() {
        let mut state = DeviceState::new(MockHardware::default());
        state.inc_rast = 1;
        state.panel_modify = Modify::AmplSel;
        state.incr_fine = true;
        state.dac_level = 123.7;
        state.first_turn = true;

        state.handle_panel_event(PanelEvent::EncoderDelta(1));
        assert!((state.dac_level - 124.0).abs() < 0.01);
        assert_eq!(
            state.hw.take_serial_output(),
            format!(
                "#0:255=67 [OK]\r\n#0:1={}\r\n",
                DeviceState::<MockHardware>::format_param(
                    state.dac_level_to_rms(state.dac_level),
                    1
                )
            )
        );

        state.handle_panel_event(PanelEvent::IncrTimerElapsed);
        state.hw.take_serial_output();
        state.incr_fine = false;
        state.first_turn = true;
        state.db = 1.8;
        state.dac_level = state.db_to_dac_level(state.db);
        state.handle_panel_event(PanelEvent::EncoderDelta(2));
        assert!((state.db - 6.0).abs() < 0.01);

        state.panel_modify = Modify::WaveSel;
        state.first_turn = true;
        state.waveform = Waveform::Square;
        state.handle_panel_event(PanelEvent::EncoderDelta(1));
        assert_eq!(state.waveform, Waveform::Logic);
        assert!((state.dac_level_to_peak_mv() - state.eeprom.init_logic_level_mv).abs() < 0.5);

        state.handle_panel_event(PanelEvent::IncrTimerElapsed);
        state.hw.take_serial_output();
        state.first_turn = true;
        state.handle_panel_event(PanelEvent::EncoderDelta(1));
        assert_eq!(state.waveform, Waveform::External(0));
        assert_eq!(state.hw.aux_configs.last(), Some(&0));
        state.hw.take_serial_output();

        state.handle_panel_event(PanelEvent::Buttons {
            enter: false,
            left: true,
            right: true,
        });
        assert_eq!(
            state.hw.take_serial_output(),
            "#0:255=65 [OK]\r\n#0:255=66 [OK]\r\n"
        );
    }

    #[test]
    fn init_all_restores_startup_setup_and_banner_semantics() {
        let mut state = DeviceState::new(MockHardware {
            lcd_setup_result: true,
            slave_channel: 2,
            ..Default::default()
        });
        state.eeprom.ee_initialized = 0;
        state.eeprom.ee_ser_baud_reg = 5;
        state.frequency_tenths_hz = 55_555;
        state.hw.push_serial("stale-input");

        state.init_all();

        assert_eq!(state.eeprom.ee_ser_baud_reg, 51);
        assert_eq!(state.serial_baud_reg, 51);
        assert_eq!(state.slave_channel, 2);
        assert!(state.lcd_present);
        assert_eq!(state.range, InputRange::Ac1V);
        assert_eq!(state.panel_modify, Modify::FreqSel);
        assert_eq!(state.current_channel, 255);
        assert_eq!(state.err_count, 0);
        assert_eq!(state.burst_count, 1);
        assert_eq!(state.burst_timer_ticks, 1);
        assert!(state.changed_flag);
        assert!(state.first_turn);
        assert!(!state.incr_fine);
        assert!(state.hw.serial_in.is_empty());
        assert_eq!(state.hw.serial_baud_calls, vec![(51, true)]);
        assert_eq!(
            state.hw.lcd_custom_chars,
            vec![(0, LCD_CHARSET_0), (1, LCD_CHARSET_1), (2, LCD_CHARSET_2),]
        );
        assert_eq!(
            state.hw.lcd_lines,
            vec![
                (0, VERS3_STR.to_string()),
                (1, EE_NOT_PROGRAMMED_STR.to_string()),
            ]
        );
        assert_eq!(
            state.hw.delay_calls,
            vec![1000, 150, 150, 150, 150, 500, 250]
        );
        assert_eq!(
            state.hw.activity_led_states,
            vec![true, false, true, false, true, false]
        );
        assert_eq!(state.hw.frequency_words.len(), 2);
        assert_eq!(state.hw.amplitude_words.len(), 2);
        assert_eq!(
            state.hw.take_serial_output(),
            format!("#2:254={VERS1_STR}{EE_NOT_PROGRAMMED_STR}\r\n")
        );
    }
}
