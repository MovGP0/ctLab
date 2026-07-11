#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Timer8 {
    pub(super) ticks: u8,
}

impl Timer8 {
    pub(super) fn set(&mut self, ticks: u8) {
        self.ticks = ticks;
    }

    pub(super) fn is_zero(&self) -> bool {
        self.ticks == 0
    }

    pub(super) fn tick(&mut self) {
        self.ticks = self.ticks.saturating_sub(1);
    }
}
