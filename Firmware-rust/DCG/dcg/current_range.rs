//! Output-current ranges selected by the DC generator's shunt network.

/// Selects the full-scale direct-current range used for shunt switching and calibration.
///
/// The order runs from the most sensitive to the highest-current shunt and is
/// also used to index the corresponding EEPROM scale and offset arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CurrentRange {
    /// Measures and regulates direct current up to 2 milliamperes.
    Dc2mA,

    /// Measures and regulates direct current up to 20 milliamperes.
    Dc20mA,

    /// Measures and regulates direct current up to 200 milliamperes.
    Dc200mA,

    /// Measures and regulates direct current up to 2 amperes.
    Dc2A,
}
impl CurrentRange {
    /// Maps a calibration-table index to its current range.
    ///
    /// Indices above the three sensitive ranges select the 2 A range, matching
    /// the bounded fallback used by automatic range selection.
    pub(super) fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Dc2mA,
            1 => Self::Dc20mA,
            2 => Self::Dc200mA,
            _ => Self::Dc2A,
        }
    }
}
