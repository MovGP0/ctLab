//! Translates the ACV preamplifier controller, including command dispatch, S/PDIF setup, front-panel handling, and host regression tests.

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

/// Declares the 16 MHz AVR clock used to derive UART, TWI, ADC, and systick timing.
const PROC_CLOCK: u32 = 16_000_000;

/// Selects TWI prescaler 1 so the bitrate register is interpreted without an extra divider.
const TWI_PRESC: u8 = 0;

/// Configures `DDRB_INIT` output bits for the board's strobes, clocks, LEDs, and serial lines.
const DDRB_INIT: u8 = 0b0001_1111;

/// Sets `PORTB_INIT` pull-ups and idle output levels before peripherals are accessed.
const PORTB_INIT: u8 = 0b0001_0000;

/// Configures `DDRC_INIT` output bits for the board's strobes, clocks, LEDs, and serial lines.
const DDRC_INIT: u8 = 0b1111_0000;

/// Sets `PORTC_INIT` pull-ups and idle output levels before peripherals are accessed.
const PORTC_INIT: u8 = 0b1111_0011;

/// Configures `DDRD_INIT` output bits for the board's strobes, clocks, LEDs, and serial lines.
const DDRD_INIT: u8 = 0b0000_0100;

/// Sets `PORTD_INIT` pull-ups and idle output levels before peripherals are accessed.
const PORTD_INIT: u8 = 0b1111_1100;

/// Selects Port B bit 4, the active-high auxiliary serial output driven by `set_aux_serial_line`.
const B_SER_AUX: u8 = 4;

/// Provides the full identification string returned by the `IDN` command.
const VERS1_STR: &str = "1.07 [ACV by CM/c't 03/2007]";

/// Provides the compact firmware name shown on the front-panel startup screen.
const VERS3_STR: &str = "ACV 1.07";

/// Warns the host during startup that the ACV EEPROM sentinel was absent and defaults were loaded.
const EE_NOT_PROGRAMMED_STR: &str = "EEPROM EMPTY! ";

/// Prefixes the one-digit slave address on the second startup LCD row.
const ADR_STR: &str = "Adr ";

/// Separates the selected input gain from its decibel unit on the gain-edit screen.
const DB_STR: &str = " dB ";

/// Appends the millivolt unit to each scaled channel reading on the level display.
const MV_STR: &str = " mV ";

/// Labels the LCD screen used to edit the input preamplifier gain.
const GAIN_SEL_STR: &str = "InpGain ";

/// Prefixes the hexadecimal auxiliary-function byte while that value is edited.
const AUX_CMD_SEL_STR: &str = "Cmd";

/// Labels the LCD screen used to edit the auxiliary-function command byte.
const AUX_CMD_STR: &str = "AuxFunct";

/// Confirms on the LCD that the current panel settings were written to EEPROM.
const MEMORIZED_STR: &str = "Memorizd";

/// Replaces a channel's numeric millivolt reading when its ADC overload flag is set.
const OVERLOAD_STR: &str = " OVERLD ";

/// Labels the LCD screen used to choose the S/PDIF sample-rate and clock mode.
const RATE_SEL_STR: &str = "SmplRate";

/// Reserves err sub ch as the wire-level subchannel used by existing ctLab clients.
const ERR_SUB_CH: u8 = 255;

/// Marks an ACV EEPROM image as initialized with the 0xAA55 sentinel.
const EE_INITIALIZED_MAGIC: u16 = 0xAA55;

/// Fixes ACV display rows at the physical eight-character LCD width.
const LCD_COLUMNS: usize = 8;

/// Uses custom LCD glyph 5 for the active edit cursor.
const LCD_CURSOR_CHAR: char = '\u{5}';

/// Uses custom LCD glyph 6 to fill an overloaded level-bar cell.
const LCD_OVERLOAD_BLOCK_CHAR: char = '\u{6}';

/// Uses custom LCD glyph 7 to mark the 0 dB point on the bar graph.
const LCD_ZERO_DB_MARK_CHAR: char = '\u{7}';

/// Forces unused active-low button bits 0..2 and 6..7 high before decoding keys on bits 3..5.
const BUTTON_UNUSED_BITS_MASK: u8 = 0b1100_0111;

/// Represents the active-low button port when every front-panel key is released.
const BUTTON_RELEASED: u8 = 0xff;

/// Provides the LCD labels for consumer (`C`) and professional (`P`) S/PDIF sample rates.
///
/// The [`Spdif`] discriminant selects the label, preserving the compact
/// seven-character presentation used by the original front panel.
#[rustfmt::skip]
const RATE_STR_ARR: [&str; 6] = [
    "C 48kHz",
    "C 96kHz",
    "C192kHz",
    "P 48kHz",
    "P 96kHz",
    "P192kHz",
];

/// Maps each programmable gain step from -20 dB through +50 dB to its relay bit pattern.
///
/// Applying the table entry to Port B switches the analogue input network to
/// the same gain step used by the conversion and display calculations.
#[rustfmt::skip]
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

/// Provides the integer divisors that convert ACV level counts for each gain range without floating point.
#[rustfmt::skip]
const ADC_RANGE_SCALES_DIV: [u16; 8] = [
    100,
    100,
    1000,
    1000,
    10000,
    1000,
    10000,
    10000,
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
#[path = "ACV_tests.rs"]
mod tests;
