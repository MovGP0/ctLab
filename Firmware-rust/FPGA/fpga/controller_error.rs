//! Controller-level errors kept separate from parser and storage implementation details.

use super::*;

/// Failure modes surfaced by the controller after transport-independent parsing.
///
/// Keeping storage errors generic lets the same command state machine drive the
/// original FAT16 implementation and deterministic in-memory tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerError<E>
{
    /// The serial frame could not be interpreted and therefore was not executed.
    Parse(ParseError),

    /// An SD-dependent command was attempted while card detection was inactive.
    NoCard,

    /// The backing filesystem rejected an otherwise valid file operation.
    File(E),

    /// FPGA `DONE` did not follow the required low-then-high configuration handshake.
    ConfigurationFailed,

    /// A command selected a register outside the fixed Pascal register banks.
    InvalidRegister,

    /// A command supplied text where a number was required, or the reverse.
    InvalidParameter,

    /// Register arithmetic refused division by zero instead of producing an invalid value.
    DivisionByZero,
}
