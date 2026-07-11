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
#[path = "DDS-SQG_tests.rs"]
mod tests;
