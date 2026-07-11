//! Parsed state for the AVR release wrapper.
//!
//! The type is isolated from command execution so argument parsing can construct
//! one coherent input shared by all later release stages.

use super::*;

/// Validated inputs shared by the AVR build, size check, and HEX conversion.
///
/// Keeping the parsed values together ensures every release stage operates on
/// the same MCU and artifacts rather than independently reinterpreting arguments.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Options
{
    /// AVR device name used for compiler flags and physical flash capacity.
    pub(super) mcu: String,

    /// Linked ELF whose sections are measured and optionally converted.
    pub(super) elf: PathBuf,

    /// Optional Intel HEX output path for programming flash.
    pub(super) hex: Option<PathBuf>,

    /// Cargo manifest selecting the firmware package to build.
    pub(super) manifest: PathBuf,

    /// Optional project limit that may be stricter than physical flash.
    pub(super) budget: Option<u64>,

    /// Optional known-good flash use against which growth is checked.
    pub(super) baseline: Option<u64>,

    /// Number of bytes by which a release may exceed its baseline.
    pub(super) allowed_regression: u64,

    /// Arguments following `--`, forwarded verbatim to Cargo.
    pub(super) cargo_args: Vec<String>,
}
