//! Defines ADA command identifiers and mnemonic metadata used by parser dispatch.

#[allow(unused_imports)]
use super::*;

/// Identifies the command mnemonic selected by parser lookup before subchannel dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdWhich {
    /// `TRG` — Trigger: schedules the channels selected by the trigger masks for immediate sampling.
    Trg,

    /// `STR` — Status: returns the packed ADA runtime/error status without changing outputs.
    Str,

    /// `IDN` — Identification: returns firmware version and detected DA12/DA16/AD16/IO32/LCD features.
    Idn,

    /// `ERC` — Error Count: reads or clears accumulated command errors.
    Erc,

    /// `VAL` — Value: accesses calibrated ADC/DAC or I/O values selected by subchannel.
    Val,

    /// `OFS` — Offset: reads or writes converter-count calibration offsets.
    Ofs,

    /// `SCL` — Scale: reads or writes counts-to-engineering-unit calibration factors.
    Scl,

    /// `RAW` — Raw: returns internal AD10 or external AD16 counts without calibration.
    Raw,

    /// `PIO` — Port I/O: reads or writes one of eight logical digital ports.
    Pio,

    /// `DIR` — Direction: reads or writes the I2C-expander direction mask for a logical port.
    Dir,

    /// `DSP` — Display: selects or labels the front-panel parameter view.
    Dsp,

    /// `ALL` — All: restores EEPROM-backed ADA defaults and reapplies detected-board outputs.
    All,

    /// `OPT` — Option: reads or writes indexed startup, reference, and detection options.
    Opt,

    /// `TRM` — Trigger Mask: selects AD10, AD16, DAC, and port responses emitted by a trigger.
    Trm,

    /// `TRT` — Trigger Timer: configures the automatic-trigger interval in milliseconds.
    Trt,

    /// `TRL` — Trigger Level: selects falling- or rising-edge external triggering.
    Trl,

    /// `ICB` — I2C Byte: reads or writes one byte at the active I2C slave address.
    Icb,

    /// `ICW` — I2C Word: reads or writes one 16-bit word in native byte order.
    Icw,

    /// `ICS` — I2C Swapped Word: exchanges a 16-bit value with high and low bytes reversed.
    Ics,

    /// `ICT` — I2C Target: stores the 7-bit slave address used by subsequent I2C commands.
    Ict,

    /// `ICA` — I2C Addressed Transfer: combines the selected address with the parser's I2C payload operation.
    Ica,

    /// `REF` — Reference: selects external or AVR internal ADC reference voltage.
    Ref,

    /// `WEN` — Write Enable: arms protected EEPROM calibration and option updates.
    Wen,

    /// `SBD` — Serial Baud: reads or writes the AVR UART baud-register value.
    Sbd,

    /// `NOP` — No Operation: validates framing while deliberately leaving ADA state unchanged.
    Nop,

    /// Internal error sentinel used when no ADA mnemonic matches.
    Err,
}

impl CmdWhich {
    /// Returns the three-letter ADA wire mnemonic, or `None` for the internal error sentinel.
    pub const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Trg => Some("TRG"),
            Self::Str => Some("STR"),
            Self::Idn => Some("IDN"),
            Self::Erc => Some("ERC"),
            Self::Val => Some("VAL"),
            Self::Ofs => Some("OFS"),
            Self::Scl => Some("SCL"),
            Self::Raw => Some("RAW"),
            Self::Pio => Some("PIO"),
            Self::Dir => Some("DIR"),
            Self::Dsp => Some("DSP"),
            Self::All => Some("ALL"),
            Self::Opt => Some("OPT"),
            Self::Trm => Some("TRM"),
            Self::Trt => Some("TRT"),
            Self::Trl => Some("TRL"),
            Self::Icb => Some("ICB"),
            Self::Icw => Some("ICW"),
            Self::Ics => Some("ICS"),
            Self::Ict => Some("ICT"),
            Self::Ica => Some("ICA"),
            Self::Ref => Some("REF"),
            Self::Wen => Some("WEN"),
            Self::Sbd => Some("SBD"),
            Self::Nop => Some("NOP"),
            Self::Err => None,
        }
    }

    /// Parses an ADA mnemonic without allocating; matching is ASCII case-insensitive and ignores surrounding whitespace.
    pub fn from_mnemonic(keyword: &str) -> Self {
        let keyword = keyword.trim();
        if keyword.eq_ignore_ascii_case("TRG") {
            Self::Trg
        } else if keyword.eq_ignore_ascii_case("STR") {
            Self::Str
        } else if keyword.eq_ignore_ascii_case("IDN") {
            Self::Idn
        } else if keyword.eq_ignore_ascii_case("ERC") {
            Self::Erc
        } else if keyword.eq_ignore_ascii_case("VAL") {
            Self::Val
        } else if keyword.eq_ignore_ascii_case("OFS") {
            Self::Ofs
        } else if keyword.eq_ignore_ascii_case("SCL") {
            Self::Scl
        } else if keyword.eq_ignore_ascii_case("RAW") {
            Self::Raw
        } else if keyword.eq_ignore_ascii_case("PIO") {
            Self::Pio
        } else if keyword.eq_ignore_ascii_case("DIR") {
            Self::Dir
        } else if keyword.eq_ignore_ascii_case("DSP") {
            Self::Dsp
        } else if keyword.eq_ignore_ascii_case("ALL") {
            Self::All
        } else if keyword.eq_ignore_ascii_case("OPT") {
            Self::Opt
        } else if keyword.eq_ignore_ascii_case("TRM") {
            Self::Trm
        } else if keyword.eq_ignore_ascii_case("TRT") {
            Self::Trt
        } else if keyword.eq_ignore_ascii_case("TRL") {
            Self::Trl
        } else if keyword.eq_ignore_ascii_case("ICB") {
            Self::Icb
        } else if keyword.eq_ignore_ascii_case("ICW") {
            Self::Icw
        } else if keyword.eq_ignore_ascii_case("ICS") {
            Self::Ics
        } else if keyword.eq_ignore_ascii_case("ICT") {
            Self::Ict
        } else if keyword.eq_ignore_ascii_case("ICA") {
            Self::Ica
        } else if keyword.eq_ignore_ascii_case("REF") {
            Self::Ref
        } else if keyword.eq_ignore_ascii_case("WEN") {
            Self::Wen
        } else if keyword.eq_ignore_ascii_case("SBD") {
            Self::Sbd
        } else if keyword.eq_ignore_ascii_case("NOP") {
            Self::Nop
        } else {
            Self::Err
        }
    }

    /// Returns sub channel offset from the command table metadata used to distinguish requests from setters.
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

    /// Returns requires parameter on set from the command table metadata used to distinguish requests from setters.
    pub fn requires_parameter_on_set(self) -> bool {
        !matches!(
            self,
            Self::Trg | Self::Str | Self::Idn | Self::Erc | Self::Err
        )
    }
}
