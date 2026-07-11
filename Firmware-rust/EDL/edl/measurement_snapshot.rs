use super::*;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct MeasurementSnapshot {
    pub voltage_on: Float,
    pub current_on: Float,
    pub voltage_off: Float,
    pub current_off: Float,
    pub power_on: Float,
    pub power_off: Float,
    pub power_avg: Float,
}
