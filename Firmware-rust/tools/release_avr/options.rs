use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Options
{
    pub(super) mcu: String,
    pub(super) elf: PathBuf,
    pub(super) hex: Option<PathBuf>,
    pub(super) manifest: PathBuf,
    pub(super) budget: Option<u64>,
    pub(super) baseline: Option<u64>,
    pub(super) allowed_regression: u64,
    pub(super) cargo_args: Vec<String>,
}
