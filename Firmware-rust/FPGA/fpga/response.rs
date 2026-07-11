#[derive(Debug, Clone, PartialEq)]
pub enum Response
{
    None,
    Number(f64),
    Integer(i64),
    Text(String),
}
