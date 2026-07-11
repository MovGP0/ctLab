//! Names the two EDL voltage-divider calibration ranges.

/// Selects the low or high voltage path without exposing calibration-array indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoltageRange
{
    /// Low-voltage divider used for the more sensitive measurement range.
    Low,

    /// High-voltage divider used when the input exceeds the low-range limit.
    High,
}

impl VoltageRange
{
    /// Returns the calibration-array position wired to this voltage range.
    pub const fn index(self) -> usize
    {
        match self
        {
            Self::Low => 0,
            Self::High => 1,
        }
    }

    /// Resolves a calibration-array position without accepting a nonexistent voltage range.
    pub const fn from_index(index: usize) -> Option<Self>
    {
        match index
        {
            0 => Some(Self::Low),
            1 => Some(Self::High),
            _ => None,
        }
    }
}
