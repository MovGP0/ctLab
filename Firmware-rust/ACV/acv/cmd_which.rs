#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CmdWhich {
    Str,
    Idn,
    Val,
    Smp,
    Inl,
    Rng,
    Dsp,
    All,
    Scl,
    Wen,
    Erc,
    Sbd,
    Nop,
    Err,
}
