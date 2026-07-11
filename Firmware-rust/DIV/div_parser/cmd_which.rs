//! Defines DIV command identifiers and mnemonic metadata used by parser dispatch.

#[allow(unused_imports)]
use super::*;

/// Identifies the three-letter c't-Lab command decoded before subchannel dispatch.
///
/// Each variant keeps the mnemonic used on the serial wire, while its
/// documentation gives the complete command name and DIV-specific behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CmdWhich {
    /// **STR — Status Request:** reports status and the current error condition.
    Str = 0,

    /// **IDN — Identification:** reports the DIV firmware identification string.
    Idn = 1,

    /// **TRG — Trigger:** immediately emits the channels enabled by the trigger mask.
    Trg = 2,

    /// **VAL — Value:** reads a scaled or raw ADC measurement selected by subchannel.
    Val = 3,

    /// **RNG — Range:** reads or selects one of the 16 voltage/current ranges.
    Rng = 4,

    /// **DSP — Display:** reads or configures display and encoder parameters.
    Dsp = 5,

    /// **OFS — Offset:** reads or writes a range-specific ADC calibration offset.
    Ofs = 6,

    /// **SCL — Scale:** reads or writes a range-specific ADC calibration factor.
    Scl = 7,

    /// **ALL — All Values:** requests all DIV input voltages; the single-input
    /// DIV maps this to its current 24-bit ADC measurement on response channel 0.
    All = 8,

    /// **TRM — Trigger Mask:** selects which measurement channels a trigger emits.
    Trm = 9,

    /// **TRT — Trigger Timer:** sets periodic triggering in milliseconds; zero disables it.
    Trt = 10,

    /// **TRL — Trigger Level:** selects a falling (`0`) or rising (`1`) edge on PB2/INT2.
    Trl = 11,

    /// **ERC — Error Count:** reads or resets the accumulated protocol-error counter.
    Erc = 12,

    /// **SBD — Serial Baud:** selects the UART UBRR value while double-speed mode is active.
    Sbd = 13,

    /// **WEN — Write Enable:** unlocks one subsequent EEPROM-affecting command.
    Wen = 14,

    /// **NOP — No Operation:** accepts the frame without changing instrument state.
    Nop = 15,

    /// **ERR — Error:** internal sentinel used when no supported mnemonic matched.
    Err = 16,
}

impl CmdWhich {
    /// Returns the three-letter mnemonic emitted and accepted by the c't-Lab protocol.
    ///
    /// Keeping this exhaustive match beside the enum prevents variant order from
    /// silently changing the command text. [`Self::Err`] returns `None` because
    /// it is a parser sentinel, not a command accepted on the wire.
    pub const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Str => Some("STR"),
            Self::Idn => Some("IDN"),
            Self::Trg => Some("TRG"),
            Self::Val => Some("VAL"),
            Self::Rng => Some("RNG"),
            Self::Dsp => Some("DSP"),
            Self::Ofs => Some("OFS"),
            Self::Scl => Some("SCL"),
            Self::All => Some("ALL"),
            Self::Trm => Some("TRM"),
            Self::Trt => Some("TRT"),
            Self::Trl => Some("TRL"),
            Self::Erc => Some("ERC"),
            Self::Sbd => Some("SBD"),
            Self::Wen => Some("WEN"),
            Self::Nop => Some("NOP"),
            Self::Err => None,
        }
    }

    /// Decodes a three-letter command without allocating or depending on enum order.
    ///
    /// Matching ASCII case-insensitively preserves the Pascal parser's terminal-
    /// friendly behavior. Unknown text becomes Self::Err so dispatch cannot
    /// accidentally execute a valid command.
    pub fn from_mnemonic(value: &str) -> Self {
        if value.eq_ignore_ascii_case("STR") {
            Self::Str
        } else if value.eq_ignore_ascii_case("IDN") {
            Self::Idn
        } else if value.eq_ignore_ascii_case("TRG") {
            Self::Trg
        } else if value.eq_ignore_ascii_case("VAL") {
            Self::Val
        } else if value.eq_ignore_ascii_case("RNG") {
            Self::Rng
        } else if value.eq_ignore_ascii_case("DSP") {
            Self::Dsp
        } else if value.eq_ignore_ascii_case("OFS") {
            Self::Ofs
        } else if value.eq_ignore_ascii_case("SCL") {
            Self::Scl
        } else if value.eq_ignore_ascii_case("ALL") {
            Self::All
        } else if value.eq_ignore_ascii_case("TRM") {
            Self::Trm
        } else if value.eq_ignore_ascii_case("TRT") {
            Self::Trt
        } else if value.eq_ignore_ascii_case("TRL") {
            Self::Trl
        } else if value.eq_ignore_ascii_case("ERC") {
            Self::Erc
        } else if value.eq_ignore_ascii_case("SBD") {
            Self::Sbd
        } else if value.eq_ignore_ascii_case("WEN") {
            Self::Wen
        } else if value.eq_ignore_ascii_case("NOP") {
            Self::Nop
        } else {
            Self::Err
        }
    }

    /// Returns the base numeric subchannel assigned to this mnemonic.
    ///
    /// The parser adds any explicit command argument to this value. Self::Err
    /// has no subchannel because it must be rejected before dispatch.
    pub const fn sub_channel(self) -> Option<u8> {
        match self {
            Self::Str => Some(255),
            Self::Idn => Some(254),
            Self::Trg => Some(249),
            Self::Val => Some(0),
            Self::Rng => Some(19),
            Self::Dsp => Some(80),
            Self::Ofs => Some(100),
            Self::Scl => Some(200),
            Self::All => Some(99),
            Self::Trm => Some(240),
            Self::Trt => Some(247),
            Self::Trl => Some(248),
            Self::Erc => Some(251),
            Self::Sbd => Some(252),
            Self::Wen => Some(250),
            Self::Nop => Some(0),
            Self::Err => None,
        }
    }
}
