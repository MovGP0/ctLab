#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Modify {
    WaveSel,
    FreqSel,
    AmplSel,
    BurstSel,
    DcSel,
}
