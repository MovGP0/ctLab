//! Defines cycle-level DDS pin operations needed to keep converter frames uninterrupted.

use super::*;

/// Cycle-level DDS pin interface used to preserve clock, latch, and critical-section ordering across AVR variants and host tests.
pub trait DdsHardwareIo {
    /// Raises one logical output bit without exposing target register addresses to the shift routines.
    fn set_bit(&mut self, port: PortKind, bit: u8);

    /// Lowers one logical output bit without exposing target register addresses to the shift routines.
    fn clear_bit(&mut self, port: PortKind, bit: u8);

    /// Emits one AVR no-operation cycle used to satisfy converter setup and hold times without touching registers.
    fn nop(&mut self);

    /// Expands a board timing unit into deterministic no-operation cycles between serial edges.
    fn delay_units(&mut self, units: u8);

    /// Provides board-timed settling while keeping the delay mechanism replaceable in host tests.
    fn delay_ms(&mut self, milliseconds: u16);

    /// Saves interrupt state and excludes timer service until the complete DDS or DAC frame has been clocked.
    fn begin_critical_section(&mut self);

    /// Restores the interrupt state captured before the contiguous DDS or DAC frame.
    fn end_critical_section(&mut self);
}
