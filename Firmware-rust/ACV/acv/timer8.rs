//! Defines ACV the systick-driven countdown used by nonblocking firmware delays.

#[allow(unused_imports)]
use super::*;

/// Models the eight-bit SysTimer countdown used for debounce, display refresh, and activity timeouts without relying on wall-clock time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Timer8 {
    /// Counts ticks in systicks until the corresponding nonblocking action is due.
    pub(super) ticks: u8,
}

impl Timer8 {
    /// Reloads the countdown timer; callers use it to postpone an action until the requested number of systicks elapses.
    pub(super) fn set(&mut self, ticks: u8) {
        self.ticks = ticks;
    }

    /// Reports whether zero without mutating device state.
    pub(super) fn is_zero(&self) -> bool {
        self.ticks == 0
    }

    /// Decrements a nonzero countdown once, matching the saturating SysTimer behavior used by the polling loop.
    pub(super) fn tick(&mut self) {
        self.ticks = self.ticks.saturating_sub(1);
    }
}
