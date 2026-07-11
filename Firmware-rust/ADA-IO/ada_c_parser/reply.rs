#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    Float { sub_ch: u8, value: f32 },
    Int { sub_ch: u8, value: i32 },
    Text(String),
    Echo(String),
    Status { error: ParseError, status: u8 },
}
