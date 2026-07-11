#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DivRange {
    Dc250mV = 0,
    Dc2V5 = 1,
    Dc25V = 2,
    Dc250V = 3,
    Ac250mV = 4,
    Ac2V5 = 5,
    Ac25V = 6,
    Ac250V = 7,
    Dc250uA = 8,
    Dc25mA = 9,
    Dc2A5 = 10,
    Dc10A = 11,
    Ac250uA = 12,
    Ac25mA = 13,
    Ac2A5 = 14,
    Ac10A = 15,
}
