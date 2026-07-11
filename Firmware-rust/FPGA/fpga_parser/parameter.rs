#[derive(Debug, Clone, PartialEq)]
pub enum Parameter
{
    None,
    Number(f64),
    Text(String),
}
