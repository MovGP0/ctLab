/// Reasons a frame is rejected before controller state can change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError
{
    /// No command remained after line-ending removal.
    Empty,

    /// The mnemonic or direct-subchannel syntax was not recognized.
    Syntax,

    /// The channel prefix was present but not an unsigned channel number or wildcard.
    InvalidChannel,

    /// A resolved command fell outside the protocol's subchannel space.
    InvalidSubchannel,

    /// The optional two-digit XOR checksum did not match the frame body.
    InvalidChecksum,

    /// A setter contained `=` but no value, which must not be treated as zero.
    MissingValue,
}
