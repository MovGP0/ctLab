//! Defines ADA command identifiers and mnemonic metadata used by parser dispatch.

#[allow(unused_imports)]
use super::*;

/// Identifies the command mnemonic selected by parser lookup before subchannel dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CmdWhich {
    /// `TRG` — Trigger: schedules the channels selected by the trigger masks for immediate sampling.
    Trg,

    /// `STR` — Status: returns the packed ADA runtime/error status without changing outputs.
    Str,

    /// `IDN` — Identification: returns firmware version and detected ADA board features.
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

    /// `DIR` — Direction: reads or writes an I2C-expander direction mask.
    Dir,

    /// `DSP` — Display: selects or labels the front-panel parameter view.
    Dsp,

    /// `ALL` — All: restores EEPROM-backed ADA defaults and reapplies outputs.
    All,

    /// `OPT` — Option: reads or writes indexed startup, reference, and detection options.
    Opt,

    /// `TRM` — Trigger Mask: selects measurement and I/O responses emitted by a trigger.
    Trm,

    /// `TRT` — Trigger Timer: configures the automatic-trigger interval in milliseconds.
    Trt,

    /// `TRL` — Trigger Level: selects falling- or rising-edge external triggering.
    Trl,

    /// `ICB` — I2C Byte: reads or writes one byte at the active I2C slave address.
    Icb,

    /// `ICW` — I2C Word: reads or writes one 16-bit word in native byte order.
    Icw,

    /// `ICS` — I2C Swapped Word: exchanges a 16-bit value with its bytes reversed.
    Ics,

    /// `ICT` — I2C Target: stores the 7-bit slave address used by subsequent transfers.
    Ict,

    /// `ICA` — I2C Addressed Transfer: executes the address-oriented I2C parser operation.
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

    /// Returns sub channel offset from the command table metadata used to distinguish requests from setters.
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

    /// Parses an ADA mnemonic without allocating; matching is ASCII case-insensitive and ignores surrounding whitespace.
    pub fn from_str(keyword: &str) -> CmdWhich {
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

    /// Returns requires parameter on set from the command table metadata used to distinguish requests from setters.
    pub fn requires_parameter_on_set(self) -> bool {
        !matches!(
            self,
            CmdWhich::Trg | CmdWhich::Str | CmdWhich::Idn | CmdWhich::Err
        )
    }
}
