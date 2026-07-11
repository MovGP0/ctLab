//! Error returned when a raw EDL subchannel has no defined protocol operation.

/// Retains an invalid wire value so parser diagnostics and tests can identify the rejected input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSubChannel
{
    /// Signed value parsed from the incoming frame before validation.
    pub wire_value: i32,
}
