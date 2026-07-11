//! Serial command parser for the EDL electronic-load firmware.
//!
//! The split between device-specific getters/setters and generic channel,
//! mnemonic, token, and checksum parsing mirrors `EDL-Parser.pas`. Mutable
//! fields model the Pascal globals so protocol behavior can be tested without
//! silently replacing hardware-dependent operations.

/// Verbose parser outcome labels.
#[path = "edl_parser/prompt_code.rs"]
mod prompt_code;
pub use prompt_code::PromptCode;

/// Parser-side mode values retaining invalid wire bytes.
#[path = "edl_parser/mode.rs"]
mod mode;
pub use mode::Mode;

/// Parser-side menu values retaining invalid wire bytes.
#[path = "edl_parser/modify.rs"]
mod modify;
pub use modify::Modify;

/// Compiler-checked EDL mnemonic and base-subchannel mapping shared with the foreground state machine.
pub use crate::edl::CmdWhich;

/// Fixed EEPROM option positions shared with the foreground EDL state machine.
pub use crate::edl::OptionSlot;

const DACI_COUNT: usize = 4;
const ADCU_COUNT: usize = 2;
const ADCI_COUNT: usize = 4;
const SHUNT_D: u8 = 3;
const DEFAULT_DAC_MAX: u16 = 4095;
#[rustfmt::skip]
const DEFAULT_OPTION_ARRAY: [f64; OptionSlot::COUNT] = [
    0.0,
    0.02,
    2.5,
    10.0,
    10.0,
    2.5,
    50.0,
    100.0,
    10.0,
    1.0,
    0.1,
    0.002,
    0.020,
    0.200,
    2.0,
    25.0,
    6.1,
    4.0,
    0.0,
    10.0,
    0.0,
    50.0,
];
const ADC10_COUNT: usize = 6;
const ADC_MAX_10: f64 = 1023.0;
const ADC_MAX_16: f64 = 65535.0;

/// Source-faithful protocol state and command dispatch.
#[path = "edl_parser/edl_parser.rs"]
mod implementation;
pub use implementation::EdlParser;

#[cfg(test)]
#[path = "EDL-Parser_tests.rs"]
mod tests;
