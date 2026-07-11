#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Identity attached to each pipelined LTC1864 conversion.
pub enum MeasurementPhase {
    /// Current sampled while the load pulse is inactive.
    Ioff,

    /// Voltage sampled while the load pulse is inactive.
    Uoff,

    /// Current sampled while the load pulse is active.
    Ion,

    /// Voltage sampled while the load pulse is active.
    Uon,
}
