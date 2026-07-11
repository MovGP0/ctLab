use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerError<E>
{
    Parse(ParseError),
    NoCard,
    File(E),
    ConfigurationFailed,
    InvalidRegister,
    InvalidParameter,
    DivisionByZero,
}
