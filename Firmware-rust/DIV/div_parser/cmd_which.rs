#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CmdWhich {
    Str = 0,
    Idn = 1,
    Trg = 2,
    Val = 3,
    Rng = 4,
    Dsp = 5,
    Ofs = 6,
    Scl = 7,
    All = 8,
    Trm = 9,
    Trt = 10,
    Trl = 11,
    Erc = 12,
    Sbd = 13,
    Wen = 14,
    Nop = 15,
    Err = 16,
}
