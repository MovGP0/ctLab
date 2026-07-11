#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    pub const LOOKUP: [CmdWhich; 25] = [
        CmdWhich::Trg,
        CmdWhich::Str,
        CmdWhich::Idn,
        CmdWhich::Val,
        CmdWhich::Ofs,
        CmdWhich::Scl,
        CmdWhich::Raw,
        CmdWhich::Pio,
        CmdWhich::Dir,
        CmdWhich::Dsp,
        CmdWhich::All,
        CmdWhich::Opt,
        CmdWhich::Trm,
        CmdWhich::Trt,
        CmdWhich::Trl,
        CmdWhich::Icb,
        CmdWhich::Icw,
        CmdWhich::Ics,
        CmdWhich::Ict,
        CmdWhich::Ica,
        CmdWhich::Ref,
        CmdWhich::Wen,
        CmdWhich::Erc,
        CmdWhich::Sbd,
        CmdWhich::Nop,
    ];

    pub fn keyword(self) -> Option<&'static str> {
        match self {
            CmdWhich::Trg => Some("TRG"),
            CmdWhich::Str => Some("STR"),
            CmdWhich::Idn => Some("IDN"),
            CmdWhich::Erc => Some("ERC"),
            CmdWhich::Val => Some("VAL"),
            CmdWhich::Ofs => Some("OFS"),
            CmdWhich::Scl => Some("SCL"),
            CmdWhich::Raw => Some("RAW"),
            CmdWhich::Pio => Some("PIO"),
            CmdWhich::Dir => Some("DIR"),
            CmdWhich::Dsp => Some("DSP"),
            CmdWhich::All => Some("ALL"),
            CmdWhich::Opt => Some("OPT"),
            CmdWhich::Trm => Some("TRM"),
            CmdWhich::Trt => Some("TRT"),
            CmdWhich::Trl => Some("TRL"),
            CmdWhich::Icb => Some("ICB"),
            CmdWhich::Icw => Some("ICW"),
            CmdWhich::Ics => Some("ICS"),
            CmdWhich::Ict => Some("ICT"),
            CmdWhich::Ica => Some("ICA"),
            CmdWhich::Ref => Some("REF"),
            CmdWhich::Wen => Some("WEN"),
            CmdWhich::Sbd => Some("SBD"),
            CmdWhich::Nop => Some("NOP"),
            CmdWhich::Err => None,
        }
    }

    pub fn sub_channel_offset(self) -> Option<u8> {
        match self {
            CmdWhich::Trg => Some(249),
            CmdWhich::Str => Some(255),
            CmdWhich::Idn => Some(254),
            CmdWhich::Val => Some(0),
            CmdWhich::Ofs => Some(100),
            CmdWhich::Scl => Some(200),
            CmdWhich::Raw => Some(50),
            CmdWhich::Pio => Some(30),
            CmdWhich::Dir => Some(40),
            CmdWhich::Dsp => Some(80),
            CmdWhich::All => Some(95),
            CmdWhich::Opt => Some(150),
            CmdWhich::Trm => Some(240),
            CmdWhich::Trt => Some(247),
            CmdWhich::Trl => Some(248),
            CmdWhich::Icb => Some(230),
            CmdWhich::Icw => Some(231),
            CmdWhich::Ics => Some(232),
            CmdWhich::Ict => Some(233),
            CmdWhich::Ica => Some(239),
            CmdWhich::Ref => Some(246),
            CmdWhich::Wen => Some(250),
            CmdWhich::Erc => Some(251),
            CmdWhich::Sbd => Some(252),
            CmdWhich::Nop => Some(0),
            CmdWhich::Err => None,
        }
    }

    pub fn from_keyword(keyword: &str) -> CmdWhich {
        let upper = keyword.trim().to_ascii_uppercase();
        Self::LOOKUP
            .iter()
            .copied()
            .find(|cmd| cmd.keyword() == Some(upper.as_str()))
            .unwrap_or(CmdWhich::Err)
    }

    pub fn requires_parameter_on_set(self) -> bool {
        !matches!(
            self,
            CmdWhich::Trg | CmdWhich::Str | CmdWhich::Idn | CmdWhich::Err
        )
    }
}
