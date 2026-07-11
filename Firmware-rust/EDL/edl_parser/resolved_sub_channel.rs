//! Couples exact EDL wire identity with its semantic operation.

use super::{EdlSubChannel, InvalidSubChannel};

/// Preserves the numeric subchannel for replies while dispatching through a typed operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedSubChannel
{
    /// Original validated protocol number used when formatting response frames.
    pub wire_value: u16,

    /// Semantic operation used by getter and setter dispatch.
    pub operation: EdlSubChannel,
}

impl TryFrom<i32> for ResolvedSubChannel
{
    type Error = InvalidSubChannel;

    /// Validates the signed parser result and decodes it without losing alias wire identity.
    fn try_from(wire_value: i32) -> Result<Self, Self::Error>
    {
        let unsigned = u16::try_from(wire_value).map_err(|_| InvalidSubChannel { wire_value })?;
        let operation = EdlSubChannel::from_wire(unsigned)
            .ok_or(InvalidSubChannel { wire_value })?;

        Ok(Self
        {
            wire_value: unsigned,
            operation,
        })
    }
}
