use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
/// One complete on/off-phase acquisition used for regulation, telemetry, and integration.
pub struct MeasurementSnapshot {
    /// Scaled input voltage while the load pulse is active.
    pub voltage_on: Float,

    /// Scaled load current while the pulse is active.
    pub current_on: Float,

    /// Input voltage sampled during the ripple-off phase.
    pub voltage_off: Float,

    /// Residual current sampled during the ripple-off phase.
    pub current_off: Float,

    /// Instantaneous active-phase power.
    pub power_on: Float,

    /// Instantaneous off-phase power retained for diagnostics.
    pub power_off: Float,

    /// Duty-cycle-weighted power consumed by telemetry and energy accumulation.
    pub power_avg: Float,
}
