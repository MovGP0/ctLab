// Best-effort Rust port of ctLab/Firmware/DCG/DCG-Parser.pas.
//
// This keeps the original parser structure, command tables, sub-channel
// mapping, and stateful serial parsing flow. Hardware-facing routines are
// modeled as placeholders so the parser remains readable without pulling in
// the rest of the firmware.

#[path = "dcg_parser/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;

#[path = "dcg_parser/modify.rs"]
mod modify;
pub use modify::Modify;

#[path = "dcg_parser/error.rs"]
mod error;
pub use error::Error;

#[path = "dcg_parser/dcg_parser.rs"]
mod parser_impl;
pub use parser_impl::DcgParser;

/// Fixed EEPROM option positions shared with the foreground DCG state machine.
pub use crate::dcg::OptionSlot;

/// Firmware banner returned by identification requests and shown during startup for service traceability.
pub const VERS1_STR: &str = "2.92 [DCG by CM/c't 05/2010]";

/// Parser-model DCG full-scale voltage used when no EEPROM option image is supplied.
pub const DEFAULT_U_MAX: f32 = 30.0;

/// Parser-model maximum current used when no EEPROM option image is supplied.
pub const DEFAULT_I_MAX: f32 = 2.0;

/// Parser-model 12.1-volt relay transition matching the Pascal factory calibration.
pub const DEFAULT_SWITCHPOINT: f32 = 12.1;

/// Parser-model full-scale code of the factory 12-bit voltage/current DAC.
pub const DEFAULT_DAC_MAX: u16 = 4095;

/// Factory full-scale amperage for each of the four current shunts used by parser-only tests.
#[rustfmt::skip]
pub const DEFAULT_I_MAX_ARRAY: [f32; 4] = [
    0.002,
    0.020,
    0.200,
    2.000,
];

/// Complete factory DCG option image used by the parser model when real EEPROM is unavailable.
#[rustfmt::skip]
pub const DEFAULT_OPTION_ARRAY: [f32; OptionSlot::COUNT] = [
    5.0,
    0.02,
    3.0,
    3.0,
    0.25,
    2.5,
    DEFAULT_U_MAX,
    470.0,
    47.0,
    4.7,
    0.47,
    0.002,
    0.020,
    0.200,
    DEFAULT_I_MAX,
    2.0,
    6.0,
    7.0,
    DEFAULT_SWITCHPOINT,
    8.6,
    8.9,
    50.0,
    0.0,
    4.0,
    6.0,
];

#[cfg(test)]
#[path = "DCG-Parser_tests.rs"]
mod tests;
