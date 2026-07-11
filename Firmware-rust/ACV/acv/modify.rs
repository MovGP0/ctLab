#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Modify {
    AuxCmdSel,
    RateSel,
    GainSel,
    LevelBarDispl,
    MvDispl,
}
