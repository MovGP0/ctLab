#![allow(dead_code)]

use core::fmt::Write as _;

/*
DDS-Funktionsgenerator mit AD98833
AD9833-DDS an PortB, Pegeleinstellung
in 2-mV-Schritten per AD7541 ueber 4094 SR, Ausgangspegel mit Offset
(c) by c't magazin und Carsten Meyer, cm@ctmagazin.de

29.08.2007 #0.10 SQG-Version, nur Rechteck bis 10 MHz
09.08.2007 #3.62 ERC und SBD eingefuehrt
23.07.2007 #3.60 RxD-Abfrage geaendert in Timeout-Befehl -- mega32 hat kein FIFO
                 Busy-Flag wird bei Bedienung gesetzt, Befehle dann mit Busy-Meldung,
                 Abfragen weiterhin moeglich. Systick=10ms, Timer aufgeraeumt wg. kuerzerem IRQ.
                 Optionale XOR-Pruefsumme eingefuehrt, mit "$XX" dem Befehl hintangestellt,
                 wird berechnet ueber gesamten Befehl, Praefix-"$" zaehlt nicht mehr mit
20.07.2007 #3.53 DSP-Parameter fuer Panel,
                 Parser geaendert: Request wenn kein "=", ausf. Response nur mit "?" oder "!"
                 Einstellung der Aussgangsstufen-Verstaerkung und Abschwaech-Faktor
                 passiver Abschwaecher einstellbar ueber SCL-Parameter, automatische Anpassung
                 der Umschaltpunkte und der Anzeige
26.06.2007 #3.483 Parameter umgestellt fuer Peak-DACLevel, Overload-Flag fuer Input
06.06.2007 #3.48 Inkrementalgeber-Routinen aufgeraeumt
06.06.2007 #3.38 angepasst an ATmega32, andere Ports
27.03.2007 #3.28 Status-Codes eingefuehrt, Uebermittlung der Bedienelemente
19.03.2007 #3.27 Parser-Syntax #!?, Error-Codes, ALL-Request
25.02.2007 #3.20 Parser zweigeteilt, mit Zeitschleifen-Check fuer SerInp
23.01.2007 #3.10 per Define auf neue Platine (zwei 4094 SR) angepasst
11.02.2007 wg. Platzbedarf LongInt fuer Frequenz und DACLevel eingefuehrt
15.01.2007 neuer Standard-Parser wie bei DCG und DIV
14.10.2006 Uebernahme aus MP3source Labor, steuert MP3-Spieler
           Yampp Industrial III von Jesper Hansen, www.jelu.se

This is a best-effort structural Rust port of DDS-SQG.pas. The original
firmware uses AVR/Pascal-specific libraries and inline assembly; those parts are
represented here via a hardware abstraction trait and explicit stubs.
*/

/// Ten-millisecond SQG scheduler period used to advance burst, panel-busy, display, and encoder-release timers.
const SYS_TICK_MS: u8 = 10;

/// AD9833 frequency-register-zero prefix ORed into each fourteen-bit half of the SQG tuning word.
const DDS_FREQ_REG_CMD: u16 = 0b0100_0000_0000_0000;

/// Subchannel sentinel returned by mnemonic lookup when no valid command mapping exists.
const ERR_SUB_CH: u8 = 255;

/// Firmware banner returned by identification requests and shown during startup for service traceability.
const VERS1_STR: &str = "0.10 [SQG by CM/c't 03/2007]";

/// Firmware banner returned by identification requests and shown during startup for service traceability.
const VERS3_STR: &str = "SQG 0.10";

/// Panel text shown while the SQG EEPROM marker indicates that factory defaults must be installed.
const EEPROM_EMPTY_STR: &str = "EEPROM EMPTY! ";

/// Serial reply label placed before the instrument's configured multidrop address.
const ADR_STR: &str = "Adr ";

/// Fixed LCD label for the SQG frequency edit page.
const FREQU_STR: &str = "Frequ Hz";

/// Fixed LCD label for the SQG amplitude edit page.
const LEVEL_STR: &str = "Level ";

/// Fixed LCD label for the measured RMS input page.
const RMS_INPUT_STR: &str = "Input ";

/// Fixed LCD label for waveform selection.
const WAVE_STR: &str = "Function";

/// Fixed LCD label for burst-interval editing.
const BURST_STR: &str = "Burst ms";

/// EEPROM format signature checked at startup before persisted calibration is trusted.
const EEPROM_INITIALIZED: u16 = 0xAA55;

// Frequency decade weights for the AD9833 tuning-word calculation at a 20 MHz
// reference clock. The Pascal code builds the DDS word digit by digit.

/// Decimal-decade factors used by SQG tuning-word generation to reproduce the Pascal floating-point sum.
#[rustfmt::skip]
const FHZ: [f64; 9] = [
    134_217_728.0,
    13_421_772.8,
    1_342_177.28,
    134_217.728,
    13_421.772_8,
    1_342.177_28,
    134.217728,
    13.4217728,
    1.34217728,
];

/// Preferred one-third-octave frequency setpoints in tenths of a hertz used by coarse panel tuning.
#[rustfmt::skip]
const TERZ_ARRAY: [i32; 32] = [
    10,
    20,
    50,
    100,
    200,
    500,
    1_000,
    2_000,
    5_000,
    10_000,
    20_000,
    50_000,
    100_000,
    200_000,
    500_000,
    1_000_000,
    2_000_000,
    5_000_000,
    10_000_000,
    20_000_000,
    50_000_000,
    80_000_000,
    100_000_000,
    18_432_000,
    24_576_000,
    35_795_450,
    41_943_040,
    44_336_817,
    49_152_000,
    65_536_000,
    73_728_000,
    0,
];

#[path = "dds_sqg/cmd_which.rs"]
mod cmd_which;
use cmd_which::CmdWhich;

#[path = "dds_sqg/modify.rs"]
mod modify;
use modify::Modify;

#[path = "dds_sqg/panel_button.rs"]
mod panel_button;
use panel_button::PanelButton;

#[path = "dds_sqg/panel_event.rs"]
mod panel_event;
use panel_event::PanelEvent;

#[path = "dds_sqg/panel_request_code.rs"]
mod panel_request_code;
use panel_request_code::PanelRequestCode;

#[path = "dds_sqg/switch_output.rs"]
mod switch_output;
use switch_output::SwitchOutput;

#[path = "dds_sqg/error_code.rs"]
mod error_code;
use error_code::ErrorCode;

#[path = "dds_sqg/eeprom_defaults.rs"]
mod eeprom_defaults;
use eeprom_defaults::EepromDefaults;

#[path = "dds_sqg/status_flags.rs"]
mod status_flags;
use status_flags::StatusFlags;

#[path = "dds_sqg/firmware_state.rs"]
mod firmware_state;

#[cfg(test)]
use firmware_state::FirmwareState;

#[path = "dds_sqg/hardware_interface.rs"]
mod hardware_interface;
use hardware_interface::HardwareInterface;

use crate::{Ad9833Control, Waveform};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Default)]
    struct DummyHardware {
        serial: String,
        serial_in: VecDeque<char>,
        dds_words: Vec<u16>,
        shift_ops: Vec<(i32, u8)>,
        offset_ops: Vec<i16>,
        delays: Vec<u16>,
        baud_calls: Vec<(u8, bool)>,
        slave_channel: u8,
        activity_leds: Vec<bool>,
        lcd_setup_result: bool,
        lcd_chars: Vec<(u8, [u8; 8])>,
        lcd_lines: Vec<(u8, String)>,
        serial_pending: bool,
        systicks: u8,
        panel_events: Vec<PanelEvent>,
    }

    impl HardwareInterface for DummyHardware {
        fn serout_byte(&mut self, byte: u8) {
            self.serial.push(byte as char);
        }

        fn write_serial(&mut self, text: &str) {
            self.serial.push_str(text);
        }

        fn send_dds_word(&mut self, word: u16) {
            self.dds_words.push(word);
        }

        fn shift_out_level_sr(&mut self, level: i32, switch_state: u8) {
            self.shift_ops.push((level, switch_state));
        }

        fn shift_out_offset_dac(&mut self, dac_counts: i16) {
            self.offset_ops.push(dac_counts);
        }

        fn serial_timeout_char(&mut self, _timeout_ticks: u8) -> Option<char> {
            self.serial_in.pop_front()
        }

        fn set_serial_baud_register(&mut self, register: u8, double_speed: bool) {
            self.baud_calls.push((register, double_speed));
        }

        fn read_slave_channel(&mut self) -> u8 {
            self.slave_channel
        }

        fn serial_read_immediate(&mut self) -> Option<char> {
            self.serial_in.pop_front()
        }

        fn lcd_setup(&mut self) -> bool {
            self.lcd_setup_result
        }

        fn lcd_define_custom_char(&mut self, slot: u8, bitmap: [u8; 8]) {
            self.lcd_chars.push((slot, bitmap));
        }

        fn lcd_write_line(&mut self, row: u8, text: &str) {
            self.lcd_lines.push((row, text.to_string()));
        }

        fn serial_pending(&self) -> bool {
            self.serial_pending
        }

        fn take_systick(&mut self) -> bool {
            if self.systicks == 0 {
                false
            } else {
                self.systicks -= 1;
                true
            }
        }

        fn next_panel_event(&mut self) -> PanelEvent {
            if self.panel_events.is_empty() {
                PanelEvent::None
            } else {
                self.panel_events.remove(0)
            }
        }

        fn delay_ms(&mut self, ms: u16) {
            self.delays.push(ms);
        }

        fn set_activity_led(&mut self, active_low: bool) {
            self.activity_leds.push(active_low);
        }
    }

    #[test]
    fn set_level_dds_emits_three_dds_words() {
        let mut state = FirmwareState::default();
        let mut hw = DummyHardware::default();

        state.frequenz = 10_000;
        state.wave = Waveform::Square;
        state.set_level_dds(&mut hw);

        assert_eq!(hw.dds_words.len(), 3);
        assert_eq!(hw.dds_words[2], Ad9833Control::Square.as_word());
        assert_eq!(state.dds_frequ, 13_421);
    }

    #[test]
    fn set_level_dds_restores_level_offset_and_relay_setup() {
        let mut state = FirmwareState::default();
        let mut hw = DummyHardware::default();

        state.dac_level = 1000.0;
        state.offset_mv = 250;
        state.wave = Waveform::Square;
        state.set_level_dds(&mut hw);

        assert_eq!(hw.offset_ops, vec![50]);
        assert_eq!(
            hw.shift_ops,
            vec![(
                40_000,
                SwitchOutput::Attenuator.mask() | SwitchOutput::Square.mask(),
            )]
        );
        assert!(!state.level_range);
    }

    #[test]
    fn set_level_dds_mutes_before_switching_from_high_to_low_range() {
        let mut state = FirmwareState {
            dac_level: 1000.0,
            level_range: true,
            wave: Waveform::Square,
            ..Default::default()
        };
        let mut hw = DummyHardware::default();

        state.set_level_dds(&mut hw);

        assert_eq!(hw.dds_words[0], Ad9833Control::Reset.as_word());
        assert_eq!(
            hw.shift_ops[0],
            (
                0,
                SwitchOutput::Offset.mask() | SwitchOutput::Attenuator.mask(),
            )
        );
        assert_eq!(hw.delays, vec![5]);
        assert!(!state.level_range);
    }

    #[test]
    fn run_main_loop_iteration_drives_burst_from_systick() {
        let mut state = FirmwareState {
            burst_mode: 2,
            burst_count: 1,
            wave_cmd: Ad9833Control::Square,
            ..Default::default()
        };
        let mut hw = DummyHardware {
            systicks: 2,
            ..Default::default()
        };

        state.run_main_loop_iteration(&mut hw);

        assert_eq!(
            hw.dds_words,
            vec![
                Ad9833Control::Square.as_word(),
                Ad9833Control::Reset.as_word(),
            ]
        );
        assert_eq!(state.burst_count, 2);
    }

    #[test]
    fn run_main_loop_iteration_skips_panel_while_serial_is_pending() {
        let mut state = FirmwareState {
            lcd_present: true,
            current_ch: 0,
            ..Default::default()
        };
        let mut hw = DummyHardware {
            serial_pending: true,
            panel_events: vec![PanelEvent::Button(PanelButton::Left)],
            ..Default::default()
        };

        state.run_main_loop_iteration(&mut hw);

        assert_eq!(state.modify, Modify::Frequency);
        assert!(hw
            .panel_events
            .contains(&PanelEvent::Button(PanelButton::Left)));
    }

    #[test]
    fn panel_button_uses_busy_status_and_updates_display_selection_when_uart_idle() {
        let mut state = FirmwareState {
            lcd_present: true,
            current_ch: 0,
            ..Default::default()
        };
        let mut hw = DummyHardware {
            panel_events: vec![PanelEvent::Button(PanelButton::Left)],
            ..Default::default()
        };

        state.run_main_loop_iteration(&mut hw);

        assert_eq!(state.modify, Modify::Amplitude);
        assert!(state.status.busy);
        assert!(hw.serial.contains("#0:1=5000"));
    }

    #[test]
    fn parse_get_param_returns_version() {
        let mut state = FirmwareState::default();
        let mut hw = DummyHardware::default();

        state.sub_ch = 254;
        state.parse_get_param(&mut hw);

        assert!(hw.serial.contains(VERS1_STR));
    }

    #[test]
    fn patch_copy_from_ee_restores_sqg_eeprom_backed_reset_values() {
        let mut state = FirmwareState {
            wave: Waveform::Off,
            frequenz: 42,
            dac_level: 1000.0,
            terz_num: 1,
            burst_mode: 9,
            inc_rast: 1,
            attn_fac: 1.0,
            pwr_gain: 1.0,
            level_scale_low: 2.0,
            level_scale_hi: 3.0,
            ..Default::default()
        };
        state.defaults.init_wave = Waveform::Square.as_byte();
        state.defaults.init_frequenz = 20_000;
        state.defaults.init_level = 5000.0;
        state.defaults.init_terz_num = 12;
        state.defaults.init_burst = 7;
        state.defaults.init_inc_rast = 6;
        state.defaults.init_attn_fac = 33.0;
        state.defaults.init_pwr_gain = 1.5;
        state.defaults.level_scale_low = 0.95;
        state.defaults.level_scale_hi = 1.05;

        state.patch_copy_from_ee();

        assert_eq!(state.wave, Waveform::Square);
        assert_eq!(state.frequenz, 20_000);
        assert_eq!(state.dac_level, 5000.0);
        assert_eq!(state.terz_num, 12);
        assert_eq!(state.burst_mode, 7);
        assert_eq!(state.inc_rast, 6);
        assert_eq!(state.attn_fac, 33.0);
        assert_eq!(state.pwr_gain, 1.5);
        assert_eq!(state.level_scale_low, 0.95);
        assert_eq!(state.level_scale_hi, 1.05);
    }

    #[test]
    fn init_all_restores_pascal_startup_state_and_banner() {
        let mut state = FirmwareState::default();
        let mut hw = DummyHardware {
            serial_in: "stale".chars().collect(),
            slave_channel: 2,
            lcd_setup_result: true,
            ..Default::default()
        };

        state.defaults.ee_initialized = 0;
        state.defaults.ee_ser_baud_reg = 5;
        state.frequenz = 42;
        state.status.busy = true;
        state.err_count = 9;
        state.err_flag = true;
        state.incr_fine = true;
        state.incr_diff = 3;
        state.ser_inp_str = "partial".to_string();

        state.init_all(&mut hw);

        assert_eq!(state.defaults.ee_ser_baud_reg, 51);
        assert_eq!(hw.baud_calls, vec![(51, true)]);
        assert_eq!(state.slave_ch, 2);
        assert!(state.lcd_present);
        assert_eq!(state.status, StatusFlags::default());
        assert_eq!(state.burst_count, 1);
        assert_eq!(state.modify, Modify::Frequency);
        assert_eq!(state.current_ch, 255);
        assert_eq!(state.err_count, 0);
        assert!(!state.err_flag);
        assert!(state.changed_flag);
        assert!(state.first_turn);
        assert!(!state.incr_fine);
        assert_eq!(state.incr_diff, 0);
        assert!(state.ser_inp_str.is_empty());
        assert!(hw.serial_in.is_empty());
        assert_eq!(
            hw.lcd_lines,
            vec![
                (0, VERS3_STR.to_string()),
                (1, EEPROM_EMPTY_STR.to_string()),
            ]
        );
        assert_eq!(hw.delays, vec![1000, 150, 150, 150, 150, 500]);
        assert_eq!(
            hw.activity_leds,
            vec![true, false, true, false, true, false]
        );
        assert_eq!(
            hw.serial,
            format!("#2:254={VERS1_STR}{EEPROM_EMPTY_STR}\r\n")
        );
        assert_eq!(hw.dds_words.len(), 3);
    }

    #[test]
    fn parse_extract_accepts_pascal_numeric_token_range() {
        let state = FirmwareState::default();

        let (token, end, is_param) = state.parse_extract("  -1,5/2?", 0);

        assert!(is_param);
        assert_eq!(token, "-1,5/2");
        assert_eq!(end, 8);
    }

    #[test]
    fn command_table_includes_pascal_nop_mapping() {
        assert_eq!(FirmwareState::cmd_to_index("NOP"), CmdWhich::Nop);
        assert_eq!(FirmwareState::cmd_to_index("nop"), CmdWhich::Nop);
        assert_eq!(CmdWhich::Nop.as_str(), "NOP");
        assert_eq!(CmdWhich::Nop.default_subchannel(), 0);
        assert_eq!(CmdWhich::Ofs.as_str(), "DCO");
        assert_eq!(CmdWhich::from_mnemonic("dbu"), CmdWhich::Dbu);
        assert_eq!(CmdWhich::Dbu.default_subchannel(), 3);
    }

    #[test]
    fn shared_waveform_owns_wire_codes_labels_and_sqg_limits() {
        assert_eq!(Waveform::Off.as_byte(), 0);
        assert_eq!(Waveform::Sine.as_str(), "Sin");
        assert_eq!(Waveform::Triangle.as_str(), "Tri");
        assert_eq!(Waveform::Square.as_str(), "Squ");
        assert_eq!(Waveform::Logic.as_str(), "Lgc");
        assert_eq!(Waveform::External(0).as_str(), "Ext");
        assert_eq!(Waveform::External(2).as_byte(), 7);

        assert_eq!(Waveform::from_sqg_byte(3), (Waveform::Square, false));
        assert_eq!(Waveform::from_sqg_byte(4), (Waveform::Square, true));
        assert_eq!(Waveform::from_sqg_byte(127), (Waveform::Square, true));
        assert_eq!(Waveform::from_sqg_byte(128), (Waveform::Off, true));
        assert_eq!(Waveform::from_sqg_byte(255), (Waveform::Off, true));
    }

    #[test]
    fn typed_hardware_and_panel_values_preserve_pascal_encodings() {
        assert_eq!(Ad9833Control::Reset.as_word(), 0b0010_0001_0000_0000);
        assert_eq!(Ad9833Control::Sine.as_word(), 0b0010_0000_0000_0000);
        assert_eq!(Ad9833Control::Triangle.as_word(), 0b0010_0000_0000_0010);
        assert_eq!(Ad9833Control::Square.as_word(), 0b0010_0000_0010_1000);

        assert_eq!(SwitchOutput::Square.mask(), 1 << 4);
        assert_eq!(SwitchOutput::Attenuator.mask(), 1 << 5);
        assert_eq!(SwitchOutput::External.mask(), 1 << 6);
        assert_eq!(SwitchOutput::Offset.mask(), 1 << 7);

        assert_eq!(PanelRequestCode::Released.as_byte(), 64);
        assert_eq!(PanelRequestCode::Left.as_byte(), 65);
        assert_eq!(PanelRequestCode::Right.as_byte(), 66);
        assert_eq!(PanelRequestCode::PanelActive.as_byte(), 67);
    }

    #[test]
    fn parse_sub_ch_accepts_unprefixed_command_after_current_channel_matches() {
        let mut state = FirmwareState::default();
        let mut hw = DummyHardware::default();

        state.current_ch = state.slave_ch;
        state.ser_inp_str = "FRQ?".to_string();
        state.parse_sub_ch(&mut hw);

        assert_eq!(state.sub_ch, 0);
        assert!(hw.serial.contains("#0:0=1000"));
    }

    #[test]
    fn parse_sub_ch_accepts_unprefixed_direct_subchannel_after_current_channel_matches() {
        let mut state = FirmwareState::default();
        let mut hw = DummyHardware::default();

        state.current_ch = state.slave_ch;
        state.ser_inp_str = "4?".to_string();
        state.parse_sub_ch(&mut hw);

        assert_eq!(state.sub_ch, 4);
        assert!(hw.serial.contains("#0:4=3"));
    }

    #[test]
    fn parse_set_param_persists_encoder_detent_default() {
        let mut state = FirmwareState::default();
        let mut hw = DummyHardware::default();

        state.ser_inp_str = "0:250=1".to_string();
        state.parse_sub_ch(&mut hw);
        state.ser_inp_str = "0:89=7".to_string();
        state.parse_sub_ch(&mut hw);
        state.inc_rast = 1;

        state.patch_copy_from_ee();

        assert_eq!(state.inc_rast, 7);
        assert_eq!(state.defaults.init_inc_rast, 7);
        assert!(!state.status.ee_unlocked);
    }
}
