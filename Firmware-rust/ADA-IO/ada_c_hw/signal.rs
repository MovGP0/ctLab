#[allow(unused_imports)]
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Signal {
    SDataOut,
    SClk,
    StrDac,
    StrAd16,
    StrSr,
    StrDaMux,
    SDataIn1,
}
