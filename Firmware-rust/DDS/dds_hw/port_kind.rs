#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortKind {
    DdsOut,
    ControlBit,
    Extension,
    LedOut,
}
