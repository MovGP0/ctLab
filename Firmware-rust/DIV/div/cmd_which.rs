#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdWhich {
    Str,
    Idn,
    Trg,
    Val,
    Rng,
    Dsp,
    Ofs,
    Scl,
    All,
    Trm,
    Trt,
    Trl,
    Erc,
    Sbd,
    Wen,
    Nop,
    Err,
}
