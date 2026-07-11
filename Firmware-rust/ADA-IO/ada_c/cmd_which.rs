#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdWhich {
    Trg,
    Str,
    Idn,
    Erc,
    Val,
    Ofs,
    Scl,
    Raw,
    Pio,
    Dir,
    Dsp,
    All,
    Opt,
    Trm,
    Trt,
    Trl,
    Icb,
    Icw,
    Ics,
    Ict,
    Ica,
    Ref,
    Wen,
    Sbd,
    Nop,
    Err,
}

impl CmdWhich {
    pub fn from_keyword(keyword: &str) -> Self {
        match keyword.trim().to_ascii_uppercase().as_str() {
            "TRG" => Self::Trg,
            "STR" => Self::Str,
            "IDN" => Self::Idn,
            "ERC" => Self::Erc,
            "VAL" => Self::Val,
            "OFS" => Self::Ofs,
            "SCL" => Self::Scl,
            "RAW" => Self::Raw,
            "PIO" => Self::Pio,
            "DIR" => Self::Dir,
            "DSP" => Self::Dsp,
            "ALL" => Self::All,
            "OPT" => Self::Opt,
            "TRM" => Self::Trm,
            "TRT" => Self::Trt,
            "TRL" => Self::Trl,
            "ICB" => Self::Icb,
            "ICW" => Self::Icw,
            "ICS" => Self::Ics,
            "ICT" => Self::Ict,
            "ICA" => Self::Ica,
            "REF" => Self::Ref,
            "WEN" => Self::Wen,
            "SBD" => Self::Sbd,
            "NOP" => Self::Nop,
            _ => Self::Err,
        }
    }

    pub fn sub_channel_offset(self) -> Option<u8> {
        match self {
            Self::Trg => Some(249),
            Self::Str => Some(255),
            Self::Idn => Some(254),
            Self::Erc => Some(251),
            Self::Val => Some(0),
            Self::Ofs => Some(100),
            Self::Scl => Some(200),
            Self::Raw => Some(50),
            Self::Pio => Some(30),
            Self::Dir => Some(40),
            Self::Dsp => Some(80),
            Self::All => Some(95),
            Self::Opt => Some(150),
            Self::Trm => Some(240),
            Self::Trt => Some(247),
            Self::Trl => Some(248),
            Self::Icb => Some(230),
            Self::Icw => Some(231),
            Self::Ics => Some(232),
            Self::Ict => Some(233),
            Self::Ica => Some(239),
            Self::Ref => Some(246),
            Self::Wen => Some(250),
            Self::Sbd => Some(252),
            Self::Nop => Some(0),
            Self::Err => None,
        }
    }

    pub fn requires_parameter_on_set(self) -> bool {
        !matches!(
            self,
            Self::Trg | Self::Str | Self::Idn | Self::Erc | Self::Err
        )
    }
}
