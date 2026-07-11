//! Best-effort Rust port of `DDS.pas`.
//!
//! The Pascal firmware mixes parser handling, panel state, serial framing,
//! measurement range control, and DDS output control in one unit. This Rust
//! version keeps that single-state-machine shape, but expresses the AVR-facing
//! parts through an explicit hardware trait.

#![allow(dead_code)]

/// Firmware-wide floating-point alias retained so calibrated arithmetic has the same precision choice across host and AVR builds.
pub type Float = f32;

/// CPU frequency used to derive UART and timer timing; it must match the programmed AVR clock fuse configuration.
pub const PROC_CLOCK: u32 = 16_000_000;

/// Firmware banner returned by identification requests and shown during startup for service traceability.
pub const VERS1_STR: &str = "3.71 [DDS by CM/c't 03/2007]";

/// Firmware banner returned by identification requests and shown during startup for service traceability.
pub const VERS3_STR: &str = "DDS 3.71";

/// Serial reply label placed before the instrument's configured multidrop address.
pub const ADR_STR: &str = "Adr ";

/// Panel text warning that DDS EEPROM calibration has not yet been initialized.
pub const EE_NOT_PROGRAMMED_STR: &str = "EEPROM EMPTY! ";

/// Subchannel sentinel returned by mnemonic lookup when no valid command mapping exists.
const ERR_SUB_CH: u8 = 255;

/// EEPROM format signature checked at startup before persisted calibration is trusted.
const EEPROM_INITIALIZED: u16 = 0xAA55;

/// Highest DDS frequency accepted by serial and panel setters, expressed in the protocol's tenths-of-hertz unit.
const MAX_FREQUENCY_TENTHS_HZ: i32 = 9_999_999;

/// Largest positive or negative DC offset accepted before the offset DAC would saturate.
const MAX_OFFSET_MV: i32 = 10_000;

/// Full-scale amplitude-DAC code used to clamp calibrated level conversion.
const DAC_LEVEL_MAX: Float = 4_000.0;

/// Lowest logarithmic level accepted before amplitude is treated as effectively muted.
const MIN_DB: Float = -70.0;

/// Millivolt reference used by DDS linear-level to dB conversion.
const DB_REFERENCE_MV: Float = 774.597;

/// Converts the triangle-wave DAC amplitude representation to RMS output level.
const TRIANGLE_RMS_FACTOR: Float = 0.816_496;

/// Inverse triangle-wave factor used to obtain a DAC amplitude from an RMS request.
const TRIANGLE_DAC_FACTOR: Float = 1.224_745;

/// Converts the firmware's square-wave DAC amplitude representation to RMS output level.
const SQUARE_RMS_FACTOR: Float = core::f32::consts::SQRT_2;

/// Inverse square-wave factor used to obtain a DAC amplitude from an RMS request.
const SQUARE_DAC_FACTOR: Float = core::f32::consts::FRAC_1_SQRT_2;

/// Converts the amplitude-DAC representation through the output-stage convention to peak millivolts.
const PEAK_FACTOR: Float = 2.0 * core::f32::consts::SQRT_2;

/// AD9833 tuning contribution of each decimal frequency digit, summed without allocation or runtime exponentiation.
#[rustfmt::skip]
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

/// Analog input gain for each measurement range, indexed by `InputRange` during calibrated RMS conversion.
#[rustfmt::skip]
const INP_GAINS: [Float; 4] = [
    0.1,
    1.0,
    10.0,
    100.0,
];

/// Encoder acceleration multiplier indexed by bounded detent speed, beginning with zero movement and saturating at 500x.
#[rustfmt::skip]
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

/// Preferred one-third-octave frequency setpoints in tenths of a hertz used by coarse panel tuning.
#[rustfmt::skip]
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

/// CGRAM bitmap for the first custom panel symbol loaded during LCD setup.
#[rustfmt::skip]
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

/// CGRAM bitmap for the second custom panel symbol loaded during LCD setup.
#[rustfmt::skip]
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

/// CGRAM bitmap for the third custom panel symbol loaded during LCD setup.
#[rustfmt::skip]
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

#[path = "dds/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;

#[path = "dds/modify.rs"]
mod modify;
pub use modify::Modify;

#[path = "dds/error_code.rs"]
mod error_code;
pub use error_code::ErrorCode;

pub use crate::Waveform;

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
#[path = "DDS_tests.rs"]
mod tests;
