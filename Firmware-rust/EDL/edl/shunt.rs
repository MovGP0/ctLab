//! Names the four physical EDL current shunts.

/// Selects a current shunt without exposing calibration-array indices to protocol dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shunt
{
    /// Shunt A, providing the most sensitive and lowest-current range.
    A,

    /// Shunt B, providing the second current range.
    B,

    /// Shunt C, providing the third current range.
    C,

    /// Shunt D, providing the highest-current range.
    D,
}

impl Shunt
{
    /// Returns the calibration-array position wired to this shunt.
    pub const fn index(self) -> usize
    {
        match self
        {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
        }
    }

    /// Resolves a calibration-array position without accepting a nonexistent shunt.
    pub const fn from_index(index: usize) -> Option<Self>
    {
        match index
        {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::C),
            3 => Some(Self::D),
            _ => None,
        }
    }
}
