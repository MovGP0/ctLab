#![allow(dead_code)]

// Programmierbarer Präzisions-Vorverstärker mit AD-Wandler 192 kHz/24 Bit
// 05.05.2010 #1.07 Getrennte Skalierungen L/R auf 200..207 (L) und 210..217 (R)
//                  Option-Parameter 152 für LRswap eingeführt
// 21.02.2008 #1.06 ParseExtract geändert für Integer, wichtig!
//                  Skalierte Anzeige/Ausgabe mV je nach Gain eingeführt
// 21.02.2008 #1.05 SPDIF-Format einstellbar, Bug in Level-Befehl korrigiert
// 16.12.2007 #1.04 kein EEPROM-File mehr notwendig, initialisiert auf Defaults, autom. Bargraph
// 19.11.2007 #1.03 aus Platzgründen umgestellt auf Integer statt Float für Pegel und Param
// 14.10.2007 Parser-Übernahme aus DIV und DDS
//
// Best-effort Rust port of `ACV.pas`. This keeps the original program structure,
// constants, state, and algorithm flow readable, but replaces AVR-specific
// hardware access with mockable helpers. It is not yet a verified embedded build.

use std::{collections::VecDeque, fmt::Write as _};

const PROC_CLOCK: u32 = 16_000_000;
const TWI_PRESC: u8 = 0;

const DDRB_INIT: u8 = 0b0001_1111;
const PORTB_INIT: u8 = 0b0001_0000;
const DDRC_INIT: u8 = 0b1111_0000;
const PORTC_INIT: u8 = 0b1111_0011;
const DDRD_INIT: u8 = 0b0000_0100;
const PORTD_INIT: u8 = 0b1111_1100;

const B_SER_AUX: u8 = 4;

const VERS1_STR: &str = "1.07 [ACV by CM/c't 03/2007]";
const VERS3_STR: &str = "ACV 1.07";
const EE_NOT_PROGRAMMED_STR: &str = "EEPROM EMPTY! ";
const ADR_STR: &str = "Adr ";
const DB_STR: &str = " dB ";
const MV_STR: &str = " mV ";
const GAIN_SEL_STR: &str = "InpGain ";
const AUX_CMD_SEL_STR: &str = "Cmd";
const AUX_CMD_STR: &str = "AuxFunct";
const MEMORIZED_STR: &str = "Memorizd";
const OVERLOAD_STR: &str = " OVERLD ";
const RATE_SEL_STR: &str = "SmplRate";
const ERR_SUB_CH: u8 = 255;
const EE_INITIALIZED_MAGIC: u16 = 0xAA55;
const LCD_COLUMNS: usize = 8;
const LCD_CURSOR_CHAR: char = '\u{5}';
const LCD_OVERLOAD_BLOCK_CHAR: char = '\u{6}';
const LCD_ZERO_DB_MARK_CHAR: char = '\u{7}';
const BUTTON_UNUSED_BITS_MASK: u8 = 0b1100_0111;
const BUTTON_RELEASED: u8 = 0xff;

const ERR_STR_ARR: [&str; 8] = [
    "[OK]",
    "[SRQUSR]",
    "[BUSY]",
    "[OVERLD]",
    "[CMDERR]",
    "[PARERR]",
    "[LOCKED]",
    "[CHKSUM]",
];

const RATE_STR_ARR: [&str; 6] = [
    "C 48kHz",
    "C 96kHz",
    "C192kHz",
    "P 48kHz",
    "P 96kHz",
    "P192kHz",
];

const CMD_STR_ARR: [&str; 13] = [
    "STR",
    "IDN",
    "VAL",
    "SMP",
    "INL",
    "RNG",
    "DSP",
    "ALL",
    "SCL",
    "WEN",
    "ERC",
    "SBD",
    "NOP",
];

const CMD2_SUB_CH_ARR: [u8; 13] = [
    255,
    254,
    0,
    8,
    10,
    19,
    80,
    99,
    200,
    250,
    251,
    252,
    0
];

const SWITCH_ARR: [u8; 8] = [
    0b0000_1000,
    0b0000_1001,
    0b0000_0000,
    0b0000_0001,
    0b0000_0100,
    0b0000_0101,
    0b0000_0110,
    0b0000_0111,
];

const ADC_RANGE_SCALES_DIV: [u16; 8] = [
    100,
    100,
    1000,
    1000,
    10000,
    1000,
    10000,
    10000
];

#[path = "acv/cmd_which.rs"]
mod cmd_which;
use cmd_which::CmdWhich;
#[path = "acv/modify.rs"]
mod modify;
use modify::Modify;
#[path = "acv/spdif.rs"]
mod spdif;
use spdif::Spdif;
#[path = "acv/error.rs"]
mod error;
use error::Error;
#[path = "acv/timer8.rs"]
mod timer8;
use timer8::Timer8;
#[path = "acv/eeprom_image.rs"]
mod eeprom_image;
use eeprom_image::EepromImage;
#[path = "acv/mock_hardware.rs"]
mod mock_hardware;
use mock_hardware::MockHardware;
#[path = "acv/acv_state.rs"]
mod acv_state;
pub use acv_state::AcvState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_commands_use_pascal_subchannel_table_order() {
        let expectations = [
            ("STR", 255),
            ("IDN", 254),
            ("VAL", 0),
            ("SMP", 8),
            ("INL", 10),
            ("RNG", 19),
            ("DSP", 80),
            ("ALL", 99),
            ("SCL", 200),
            ("WEN", 250),
            ("ERC", 251),
            ("SBD", 252),
            ("NOP", 0),
        ];

        let mut state = AcvState::default();
        for (command, sub_ch) in expectations {
            state.param_str = command.to_string();
            let cmd_which = state.cmd_to_index();

            assert_ne!(cmd_which, CmdWhich::Err);
            assert_eq!(CMD2_SUB_CH_ARR[cmd_which as usize], sub_ch);
        }
    }

    #[test]
    fn ser_aux_bit_bangs_pb4_lsb_first_and_leaves_line_high() {
        let mut state = AcvState::default();
        state.hw.port_b = PORTB_INIT;

        state.ser_aux(0b1010_0101);

        assert_eq!(state.hw.aux_serial_log, vec![0b1010_0101]);
        assert_eq!(
            state.hw.aux_serial_bits,
            vec![false, true, false, true, false, false, true, false, true, true]
        );
        assert_ne!(state.hw.port_b & (1 << B_SER_AUX), 0);
    }

    #[test]
    fn init_spdif_updates_adc_board_config_for_sample_rate() {
        let mut state = AcvState::default();

        state.spdif_rate = Spdif::C48Khz;
        state.init_spdif();
        assert_eq!(state.hw.adc_config, 0b0100_0100);
        assert_eq!(state.hw.i2c_registers[0x04], 0b0110_0000);

        state.spdif_rate = Spdif::P96Khz;
        state.init_spdif();
        assert_eq!(state.hw.adc_config, 0b0100_0101);
        assert_eq!(state.hw.i2c_registers[0x04], 0b0100_0000);

        state.spdif_rate = Spdif::C192Khz;
        state.init_spdif();
        assert_eq!(state.hw.adc_config, 0b0100_0110);
        assert_eq!(state.hw.i2c_registers[0x04], 0b0111_0000);
    }

    #[test]
    fn init_all_applies_stored_uart_baud_register() {
        let mut state = AcvState::default();
        state.eeprom.ee_ser_baud_reg = 100;

        state.init_all();

        assert_eq!(state.hw.uart_baud_reg, 100);
        assert!(state.hw.uart_double_speed);
    }

    #[test]
    fn init_all_restores_default_uart_baud_register_when_stored_value_is_invalid() {
        let mut state = AcvState::default();
        state.eeprom.ee_ser_baud_reg = 8;

        state.init_all();

        assert_eq!(state.eeprom.ee_ser_baud_reg, 51);
        assert_eq!(state.hw.uart_baud_reg, 51);
        assert!(state.hw.uart_double_speed);
    }

    #[test]
    fn check_delay_services_queued_serial_input() {
        let mut state = AcvState::default();

        for ch in "8\r".chars() {
            state.push_serial_char(ch);
        }
        assert!(state.hw.serial_output.is_empty());

        state.check_delay(1);

        assert_eq!(state.hw.serial_output, "#0:8=0\r\n");
        assert!(state.hw.serial_input.is_empty());
    }

    #[test]
    fn front_panel_buttons_require_debounce_and_release_before_repeating() {
        let mut state = AcvState::default();
        state.hw.lcd_present = true;
        state.display_timer.set(10);
        state.incr_timer.set(20);
        state.modify = Modify::GainSel;

        let left_pressed = 0b1101_1111;
        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::GainSel);

        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::LevelBarDispl);

        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::LevelBarDispl);

        state.main_loop_step(0, Some(BUTTON_RELEASED));
        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::LevelBarDispl);

        state.main_loop_step(0, Some(left_pressed));
        assert_eq!(state.modify, Modify::MvDispl);
    }

    #[test]
    fn level_bar_display_preserves_channel_columns_and_pascal_markers() {
        let mut state = AcvState::default();
        state.hw.lcd_present = true;
        state.modify = Modify::LevelBarDispl;
        state.hw.adc_values[3] = 320;
        state.hw.adc_values[4] = 800;

        state.soll_werte_on_lcd();

        assert_eq!(state.hw.lcd_lines[0], "L##   \u{7} ");
        assert_eq!(state.hw.lcd_lines[1], "R######\u{6}");
    }

    #[test]
    fn lcd_menu_pages_include_pascal_cursor_glyph_and_fixed_width_rows() {
        let mut state = AcvState::default();
        state.modify = Modify::GainSel;
        state.gain = 2;

        state.soll_werte_on_lcd();

        assert_eq!(state.hw.lcd_lines[0], " +0 dB \u{5}");
        assert_eq!(state.hw.lcd_lines[1], GAIN_SEL_STR);
    }
}
