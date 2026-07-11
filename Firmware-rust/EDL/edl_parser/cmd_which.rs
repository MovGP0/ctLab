#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Result of mnemonic lookup without duplicating the 31-entry protocol enum.
pub enum CmdWhich {
    /// No command matched the input token.
    Err,

    /// Index into the paired mnemonic and base-subchannel tables.
    Index(usize),
}
