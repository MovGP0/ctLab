#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError
{
    Empty,
    Syntax,
    InvalidChannel,
    InvalidSubchannel,
    InvalidChecksum,
    MissingValue,
}
