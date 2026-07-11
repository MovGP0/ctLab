//! Best-effort Rust port of `ADA-C-parser.pas`.
//!
//! This keeps the original parser structure intact:
//! - command lookup via allocation-free enum conversions
//! - `parse_get_param` and `parse_set_param` large dispatches
//! - `parse_extract`, `cmd_to_index`, and `parse_sub_ch` flow
//!
//! Hardware-facing helpers are intentionally lightweight stubs so the parser
//! logic remains readable and can be integrated with a real backend later.

use std::{collections::VecDeque, mem};

#[path = "ada_c_parser/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;
#[path = "ada_c_parser/parse_error.rs"]
mod parse_error;
pub use parse_error::ParseError;
#[path = "ada_c_parser/reply.rs"]
mod reply;
pub use reply::Reply;
#[path = "ada_c_parser/parse_context.rs"]
mod parse_context;
pub use parse_context::ParseContext;
#[path = "ada_c_parser/ada_io_parser.rs"]
mod ada_io_parser;
pub use ada_io_parser::AdaIoParser;

#[cfg(test)]
#[path = "ADA-C-parser_tests.rs"]
mod tests;
