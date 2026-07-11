#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VoltageRange {
    ULow,
    UHigh,
}
