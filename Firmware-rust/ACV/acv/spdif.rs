#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Spdif {
    C48Khz,
    C96Khz,
    C192Khz,
    P48Khz,
    P96Khz,
    P192Khz,
}
