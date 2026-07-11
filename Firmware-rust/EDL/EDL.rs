//! Best-effort Rust port of `EDL.pas`.
//!
//! The Pascal source is a large foreground loop wrapped around timer-driven ADC
//! and DAC state. This module restores the audited main-file semantics with an
//! explicit state machine: per-fault latching, calibration derived from the
//! documented EEPROM layout, ripple/off-phase sampling, averaged power, range
//! selection, periodic telemetry, and Ah/Wh integration.

#![allow(dead_code)]

/// Firmware arithmetic precision, matching the AVR Pascal `Single` representation.
pub type Float = f32;

/// CPU clock used to derive timer and serial timing.
pub const PROC_CLOCK: u32 = 16_000_000;

/// Full serial identification returned by `IDN`.
pub const VERS1_STR: &str = "1.784 [EDL by CM/c't 09/2008]";

/// Short front-panel version shown during startup.
pub const VERS3_STR: &str = "EDL 1.78";

/// Protocol subchannel carrying errors and packed status bits.
pub const ERR_SUBCH: u8 = 255;

/// Sentinel selecting automatic shunt range calculation.
pub const AUTO_SHUNT_RANGE: u8 = 4;

/// Number of the lowest-current/highest-resistance shunt.
pub const SHUNT_D: u8 = 3;

/// Foreground measurement/control cadence inherited from the Pascal service loop.
pub const SERVICE_INTERVAL_MS: u32 = 40;

/// Ah/Wh integration cadence, kept separate from faster regulation updates.
pub const INTEGRATION_INTERVAL_MS: u32 = 200;

/// Service cycles between unsolicited measurement frames.
pub const PERIODIC_TELEMETRY_CYCLES: u8 = 10;

/// Service cycles between relatively slow temperature reads.
pub const TEMPERATURE_POLL_CYCLES: u8 = 20;

/// Absolute software over-temperature trip independent of LM75 output polarity.
pub const TEMPERATURE_MAX_C: Float = 70.0;

const LM75_INVERTED_OUTPUT_CONFIGURATION: u8 = 4;
const LM75_HYSTERESIS_C: Float = 3.0;

/// Command identities kept aligned with the Pascal mnemonic table.
#[path = "edl/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;

/// Regulation law and voltage-range modes.
#[path = "edl/mode.rs"]
mod mode;
pub use mode::Mode;

/// Front-panel encoder/menu targets.
#[path = "edl/modify.rs"]
mod modify;
pub use modify::Modify;

/// Protocol error discriminants and string-table indices.
#[path = "edl/error_code.rs"]
mod error_code;
pub use error_code::ErrorCode;

/// Signal/phase tags for converter samples.
#[path = "edl/measure_kind.rs"]
mod measure_kind;
pub use measure_kind::MeasureKind;

/// Installed DAC wire protocol selection.
#[path = "edl/dac_kind.rs"]
mod dac_kind;
pub use dac_kind::DacKind;

/// Fixed positions in the persisted EDL option image.
#[path = "edl/option_slot.rs"]
mod option_slot;
pub use option_slot::OptionSlot;

/// Physical current-measurement and output shunts.
#[path = "edl/shunt.rs"]
mod shunt;
pub use shunt::Shunt;

/// Low/high voltage calibration ranges.
#[path = "edl/voltage_range.rs"]
mod voltage_range;
pub use voltage_range::VoltageRange;

/// Optional hardware flags encoded in the EDL option byte.
#[path = "edl/hardware_option.rs"]
mod hardware_option;
pub use hardware_option::HardwareOption;

/// LM75 pointer-register selectors used by the initialization sequence.
#[path = "edl/lm75_register.rs"]
mod lm75_register;
pub use lm75_register::Lm75Register;

/// Internal and external LM75 board locations.
#[path = "edl/lm75_sensor.rs"]
mod lm75_sensor;
pub use lm75_sensor::Lm75Sensor;

/// Independently latched output protection causes.
#[path = "edl/protection_flags.rs"]
mod protection_flags;
pub use protection_flags::ProtectionFlags;

/// Fault identities keeping status bits and exact wire labels in one exhaustive match.
#[path = "edl/protection_fault.rs"]
mod protection_fault;
pub use protection_fault::ProtectionFault;

/// Persistent calibration and startup layout.
#[path = "edl/eeprom_data.rs"]
mod eeprom_data;
pub use eeprom_data::EepromData;

/// Runtime conversion factors derived from EEPROM.
#[path = "edl/scale_state.rs"]
mod scale_state;
pub use scale_state::ScaleState;

/// Volatile high-nibble protocol status flags.
#[path = "edl/runtime_status.rs"]
mod runtime_status;
pub use runtime_status::RuntimeStatus;

/// Hardware effects required by the foreground control state.
#[path = "edl/edl_hardware.rs"]
mod edl_hardware;
pub use edl_hardware::EdlHardware;

/// Coherent on/off-phase measurement results.
#[path = "edl/measurement_snapshot.rs"]
mod measurement_snapshot;
pub use measurement_snapshot::MeasurementSnapshot;

/// Foreground regulation, protection, UI, and telemetry state machine.
#[path = "edl/device_state.rs"]
mod device_state;
pub use device_state::DeviceState;

#[cfg(test)]
#[path = "EDL_tests.rs"]
mod tests;
