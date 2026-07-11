/// Identifies one of the four AVR GPIO register banks used by the firmware.
///
/// It lets family-specific adapters describe schematic pin assignments while
/// [`AvrdPortIo`](super::AvrdPortIo) performs the matching register access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterPort {
    /// GPIO bank A.
    A,

    /// GPIO bank B.
    B,

    /// GPIO bank C.
    C,

    /// GPIO bank D.
    D,
}
