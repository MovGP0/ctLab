//! Normalized frame metadata consumed by controller dispatch.

use super::*;

/// Normalized c't-Lab frame consumed by the FPGA controller dispatcher.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFrame
{
    /// Addressed module, or `None` for an unaddressed/wildcard frame.
    pub main_channel: Option<u8>,

    /// Numeric protocol operation after mnemonic-plus-argument expansion.
    pub subchannel: u16,

    /// Distinguishes getters from setters without re-parsing the original text.
    pub is_request: bool,

    /// Records the `#` result prefix for routing and echo decisions.
    pub is_result: bool,

    /// Records wildcard forwarding so callers can propagate omni-bus traffic.
    pub is_omni: bool,

    /// Tells response framing whether the sender requested explicit success/error text with `!` or `?`.
    pub verbose: bool,

    /// Typed right-hand side passed to command execution.
    pub parameter: Parameter,
}
