#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Signal and ripple phase associated with a pipelined ADC result.
pub enum MeasureKind {
    /// Current while the load is inactive.
    Ioff,

    /// Voltage while the load is inactive.
    Uoff,

    /// Current while the load is active.
    Ion,

    /// Voltage while the load is active.
    Uon,
}
