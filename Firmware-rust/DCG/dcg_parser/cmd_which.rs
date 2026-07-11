#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum CmdWhich {
    Str = 0,
    Idn,
    Chn,
    Val,
    Dcv,
    Dca,
    Mah,
    Mwh,
    Msv,
    Msa,
    Msw,
    Pcv,
    Pca,
    Pwon,
    Pwoff,
    Rip,
    Raw,
    Dsp,
    Ofs,
    Scl,
    Opt,
    All,
    Tmp,
    Wen,
    Erc,
    Sbd,
    Nop,
    Err,
}
impl CmdWhich {
    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Str,
            1 => Self::Idn,
            2 => Self::Chn,
            3 => Self::Val,
            4 => Self::Dcv,
            5 => Self::Dca,
            6 => Self::Mah,
            7 => Self::Mwh,
            8 => Self::Msv,
            9 => Self::Msa,
            10 => Self::Msw,
            11 => Self::Pcv,
            12 => Self::Pca,
            13 => Self::Pwon,
            14 => Self::Pwoff,
            15 => Self::Rip,
            16 => Self::Raw,
            17 => Self::Dsp,
            18 => Self::Ofs,
            19 => Self::Scl,
            20 => Self::Opt,
            21 => Self::All,
            22 => Self::Tmp,
            23 => Self::Wen,
            24 => Self::Erc,
            25 => Self::Sbd,
            26 => Self::Nop,
            _ => Self::Err,
        }
    }

    pub const fn as_index(self) -> usize {
        self as usize
    }
}
