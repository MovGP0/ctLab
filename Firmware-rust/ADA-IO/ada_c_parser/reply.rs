//! Defines ADA typed parser results before they are formatted as serial frames.

#[allow(unused_imports)]
use super::*;

/// Carries a typed ADA parser result until the serial layer chooses its wire formatting.
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    /// Returns a calibrated floating-point channel value.
    Float {
        /// Identifies the addressed value channel in the response frame.
        sub_ch: u8,

        /// Carries the calibrated floating-point result for that channel.
        value: f32,
    },

    /// Returns a raw or integral channel value.
    Int {
        /// Identifies the addressed value channel in the response frame.
        sub_ch: u8,

        /// Carries the raw or integral result for that channel.
        value: i32,
    },

    /// Returns an owned textual payload such as identity or feature data.
    Text(String),

    /// Forwards an omni-addressed frame unchanged to downstream devices.
    Echo(String),

    /// Returns packed protocol error and runtime status flags.
    Status {
        /// Names the parser outcome encoded by the status response.
        error: ParseError,

        /// Carries the packed busy, request, overload, and unlock flags.
        status: u8,
    },
}
