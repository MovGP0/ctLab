#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdWhich {
    Err,
    Index(usize),
}
