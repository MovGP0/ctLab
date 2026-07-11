//! Defines DIV state retained across parser, polling-loop, or interrupt operations.

#[allow(unused_imports)]
use super::*;

/// Collects div hardware state that must survive across polling-loop or interrupt updates.
#[derive(Debug, Clone, Default)]
pub struct DivHardwareState {
    /// Stores ad24 temp until calibration, limit checking, and response formatting have consumed it.
    pub ad24_temp: u32,

    /// Stores ad24 temp fast integrated until calibration, limit checking, and response formatting have consumed it.
    pub ad24_temp_fast_integrated: u32,

    /// Stores ad24 temp slow integrated until calibration, limit checking, and response formatting have consumed it.
    pub ad24_temp_slow_integrated: u32,

    /// Stores the previous fast-filter state used by the two-sample AD24 average.
    pub ad24_integrate0: u32,

    /// Stores the newest slow-filter history term.
    pub ad24_integrate1: u32,

    /// Stores the second slow-filter history term.
    pub ad24_integrate2: u32,

    /// Stores the oldest slow-filter history term.
    pub ad24_integrate3: u32,

    /// Latches negative flag from the same converter status bits as the associated sample.
    pub negative_flag: bool,

    /// Latches over voltage flag from the same converter status bits as the associated sample.
    pub over_voltage_flag: bool,

    /// Requests one manual LTC2400 clock pulse to cancel a pending conversion on the next systick.
    pub abort_flag: bool,

    /// Latches an external, automatic, or command trigger until the polling loop services it.
    pub trigger: bool,

    /// Signals that the systick handler published a fresh LTC2400 sample and integration values.
    pub ad24_ready: bool,

    /// Signals that the systick boundary allows the next AVR ADC10 read to proceed.
    pub ad10_ready: bool,
}
