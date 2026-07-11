//! Best-effort Rust port of `DCG.pas`.
//!
//! This keeps the original firmware split visible in Rust:
//! command tables, EEPROM-backed calibration, runtime state, serial/LCD
//! formatting, range switching, and the top-level service loop.

#![allow(dead_code)]

/// Firmware-wide floating-point alias retained so calibrated arithmetic has the same precision choice across host and AVR builds.
pub type Float = f32;

/// CPU frequency used to derive UART and timer timing; it must match the programmed AVR clock fuse configuration.
pub const PROC_CLOCK: u32 = 16_000_000;

/// Firmware banner returned by identification requests and shown during startup for service traceability.
pub const VERS1_STR: &str = "2.92 [DCG by CM/c't 05/2010]";

/// Firmware banner returned by identification requests and shown during startup for service traceability.
pub const VERS3_STR: &str = "DCG 2.92";

/// Serial reply label placed before the instrument's configured multidrop address.
pub const ADR_STR: &str = "Adr ";

/// Subchannel sentinel returned by mnemonic lookup when no valid command mapping exists.
pub const ERR_SUB_CH: u8 = 255;

/// Encoder acceleration multiplier indexed by bounded detent speed, beginning with zero movement and saturating at 500x.
#[rustfmt::skip]
const INCR_ACC_ARRAY: [i32; 16] = [
    0,
    1,
    2,
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
];

#[path = "dcg/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;

#[path = "dcg/modify.rs"]
mod modify;
pub use modify::Modify;

#[path = "dcg/error_code.rs"]
mod error_code;
pub use error_code::ErrorCode;

#[path = "dcg/fault_kind.rs"]
mod fault_kind;
pub use fault_kind::FaultKind;

#[path = "dcg/current_range.rs"]
mod current_range;
pub use current_range::CurrentRange;

#[path = "dcg/voltage_range.rs"]
mod voltage_range;
pub use voltage_range::VoltageRange;

#[path = "dcg/option_slot.rs"]
mod option_slot;
pub use option_slot::OptionSlot;

#[path = "dcg/hardware_option.rs"]
mod hardware_option;
pub use hardware_option::HardwareOption;

#[path = "dcg/eeprom_data.rs"]
mod eeprom_data;
pub use eeprom_data::EepromData;

#[path = "dcg/runtime_status.rs"]
mod runtime_status;
pub use runtime_status::RuntimeStatus;

#[path = "dcg/fault_flags.rs"]
mod fault_flags;
pub use fault_flags::FaultFlags;

#[path = "dcg/calibration_scale.rs"]
mod calibration_scale;
pub use calibration_scale::CalibrationScale;

#[path = "dcg/dcg_hardware.rs"]
mod dcg_hardware;
pub use dcg_hardware::DcgHardware;

#[path = "dcg/device_state.rs"]
mod device_state;
pub use device_state::DeviceState;

#[cfg(test)]
#[path = "DCG_tests.rs"]
mod tests;
