//! Defines ACV command identifiers and mnemonic metadata used by parser dispatch.

#[allow(unused_imports)]
use super::*;

/// Identifies the command mnemonic selected by parser lookup before subchannel dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CmdWhich {
    /// `STR` — Status: returns the packed runtime/error status without changing ACV state.
    Str,

    /// `IDN` — Identification: returns the ACV firmware version string.
    Idn,

    /// `VAL` — Value: accesses level, gain, auxiliary-command, and S/PDIF values selected by subchannel.
    Val,

    /// `SMP` — Sample Rate: reads or changes the consumer/professional 48/96/192 kHz S/PDIF setup.
    Smp,

    /// `INL` — Input Level: reports the latest left or right ADC-board level register.
    Inl,

    /// `RNG` — Range: reads or changes the programmable input-gain relay selection.
    Rng,

    /// `DSP` — Display: selects the front-panel gain, rate, auxiliary, bar, or millivolt view.
    Dsp,

    /// `ALL` — All: restores EEPROM-backed ACV startup settings and reapplies hardware outputs.
    All,

    /// `SCL` — Scale: reads or writes the per-channel millivolt calibration multiplier.
    Scl,

    /// `WEN` — Write Enable: arms protected EEPROM parameter updates.
    Wen,

    /// `ERC` — Error Count: reads or clears accumulated command errors.
    Erc,

    /// `SBD` — Serial Baud: reads or writes the AVR UART baud-register value.
    Sbd,

    /// `NOP` — No Operation: validates framing while deliberately leaving ACV state unchanged.
    Nop,

    /// Internal error sentinel used when no ACV mnemonic matches.
    Err,
}

impl CmdWhich {
    /// Returns the three-letter ACV wire mnemonic, or `None` for the internal error sentinel.
    pub(super) const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Str => Some("STR"),
            Self::Idn => Some("IDN"),
            Self::Val => Some("VAL"),
            Self::Smp => Some("SMP"),
            Self::Inl => Some("INL"),
            Self::Rng => Some("RNG"),
            Self::Dsp => Some("DSP"),
            Self::All => Some("ALL"),
            Self::Scl => Some("SCL"),
            Self::Wen => Some("WEN"),
            Self::Erc => Some("ERC"),
            Self::Sbd => Some("SBD"),
            Self::Nop => Some("NOP"),
            Self::Err => None,
        }
    }

    /// Parses an ACV mnemonic without allocating; matching is ASCII case-insensitive and ignores surrounding whitespace.
    pub(super) fn from_mnemonic(keyword: &str) -> Self {
        let keyword = keyword.trim();
        if keyword.eq_ignore_ascii_case("STR") {
            Self::Str
        } else if keyword.eq_ignore_ascii_case("IDN") {
            Self::Idn
        } else if keyword.eq_ignore_ascii_case("VAL") {
            Self::Val
        } else if keyword.eq_ignore_ascii_case("SMP") {
            Self::Smp
        } else if keyword.eq_ignore_ascii_case("INL") {
            Self::Inl
        } else if keyword.eq_ignore_ascii_case("RNG") {
            Self::Rng
        } else if keyword.eq_ignore_ascii_case("DSP") {
            Self::Dsp
        } else if keyword.eq_ignore_ascii_case("ALL") {
            Self::All
        } else if keyword.eq_ignore_ascii_case("SCL") {
            Self::Scl
        } else if keyword.eq_ignore_ascii_case("WEN") {
            Self::Wen
        } else if keyword.eq_ignore_ascii_case("ERC") {
            Self::Erc
        } else if keyword.eq_ignore_ascii_case("SBD") {
            Self::Sbd
        } else if keyword.eq_ignore_ascii_case("NOP") {
            Self::Nop
        } else {
            Self::Err
        }
    }

    /// Returns the Pascal command-table subchannel offset used by named command frames.
    pub(super) const fn sub_channel_offset(self) -> Option<u8> {
        match self {
            Self::Str => Some(255),
            Self::Idn => Some(254),
            Self::Val => Some(0),
            Self::Smp => Some(8),
            Self::Inl => Some(10),
            Self::Rng => Some(19),
            Self::Dsp => Some(80),
            Self::All => Some(99),
            Self::Scl => Some(200),
            Self::Wen => Some(250),
            Self::Erc => Some(251),
            Self::Sbd => Some(252),
            Self::Nop => Some(0),
            Self::Err => None,
        }
    }
}
