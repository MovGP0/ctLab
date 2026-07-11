#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlBit {
    Sclk,
    SDataOut,
    StrDac,
    StrAd16,
    SDataIn1,
}
