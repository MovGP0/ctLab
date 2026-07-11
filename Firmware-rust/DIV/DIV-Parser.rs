//! Implements the standalone DIV parser and its adapter to live voltmeter state for protocol regression tests.

// Best-effort Rust port of ctLab/Firmware/DIV/DIV-Parser.pas.
//
// This file keeps the original parser structure and lookup tables readable,
// while moving board-specific I/O and ADC behavior behind a hook trait.

use crate::div::{
    DeviceState as DivDeviceState, DivFault, DivHardware as DivRuntimeHardware, DivRange,
};

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
mod parser_impl;
pub use parser_impl::DivParser;

/// Provides the full identification string returned by the `IDN` command.
pub const VERS1_STR: &str = "3.10 [DIV by CM/c't 03/2007] ";

/// Parses u8 default and updates only the state owned by that protocol phase.
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

/// Parses f32 default and updates only the state owned by that protocol phase.
fn parse_f32_default(value: &str, default: f32) -> f32 {
    value.trim().parse::<f32>().unwrap_or(default)
}

/// Parses hex u8 default and updates only the state owned by that protocol phase.
fn parse_hex_u8_default(value: &str, default: u8) -> u8 {
    u8::from_str_radix(value.trim(), 16).unwrap_or(default)
}

/// Maps div range from u8 into the typed state used internally, rejecting or defaulting unsupported wire values as the implementation specifies.
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

/// Chooses the engineering exponent appended to serial values so milli- and micro-ranges keep the original wire units.
fn range_exponent_suffix(range: DivRange) -> Option<&'static str> {
    match range {
        DivRange::Dc250mV | DivRange::Ac250mV | DivRange::Dc25mA | DivRange::Ac25mA => Some("E-3"),
        DivRange::Dc250uA | DivRange::Ac250uA => Some("E-6"),
        _ => None,
    }
}

/// Converts serial parameter into the representation used on the wire or display.
fn format_serial_param(value: f32) -> String {
    format!("{value:.6}")
}

#[cfg(test)]
#[path = "DIV-Parser_tests.rs"]
mod tests;
