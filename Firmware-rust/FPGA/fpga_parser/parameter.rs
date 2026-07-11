/// Parameter representation retained after serial syntax has been removed.
#[derive(Debug, Clone, PartialEq)]
pub enum Parameter
{
    /// Getter or command with no right-hand-side value.
    None,

    /// Decimal value used by calculator, option, and register commands.
    Number(f64),

    /// Quoted filename or other data that must not undergo numeric conversion.
    Text(String),
}
