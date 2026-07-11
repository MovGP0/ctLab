#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Parser command identity kept index-aligned with the Pascal mnemonic table.
pub enum CmdWhich {
    /// `STR` — Status Register: reads packed runtime and protection flags on subchannel 255.
    Str,

    /// `IDN` — Identification: returns the EDL firmware/version string on subchannel 254.
    Idn,

    /// `CHN` — Channel: reads or changes the module's main bus address.
    Chn,

    /// `VAL` — Value: addresses the same live-value subchannels through a generic mnemonic.
    Val,

    /// `ENA` — Enable: opens or closes the electronic-load output path.
    Ena,

    /// `DCA` — DC Amperes: reads or sets constant current in amperes or milliamperes.
    Dca,

    /// `DCP` — DC Power: reads or sets the constant-power target in watts.
    Dcp,

    /// `DCV` — DC Voltage: reads or sets the low-input shutdown threshold and clears its latch.
    Dcv,

    /// `DCR` — DC Resistance: reads or sets the simulated load resistance in ohms.
    Dcr,

    /// `MAH` — Measured Ampere-Hours: reports or resets integrated charge.
    Mah,

    /// `MWH` — Measured Watt-Hours: reports or resets integrated energy.
    Mwh,

    /// `MSV` — Measured Sense Voltage: reports on/off-phase load voltage.
    Msv,

    /// `MSA` — Measured Sense Amperes: reports on/off-phase load current.
    Msa,

    /// `RNG` — Range: selects output mode/voltage range or the current shunt range.
    Rng,

    /// `MSW` — Measured Sense Watts: reports instantaneous and averaged dissipated power.
    Msw,

    /// `PCA` — Percent Current Amplitude: sets active/off-phase current percentage.
    Pca,

    /// `RON` — Ripple On Time: sets the active load-pulse duration.
    Ron,

    /// `ROF` — Ripple Off Time: sets the inactive load-pulse duration.
    Rof,

    /// `RIP` — Ripple: reads or changes ripple-current modulation behavior.
    Rip,

    /// `RAW` — Raw Data: exposes ADC16, ADC10, and DAC codes for calibration diagnostics.
    Raw,

    /// `DSP` — Display: reads or selects the front-panel menu/edit target.
    Dsp,

    /// `ALL` — All Measurements: emits the complete live measurement set in one request.
    All,

    /// `OFS` — Offset: reads or writes unlocked per-range ADC/DAC zero corrections.
    Ofs,

    /// `SCL` — Scale: reads or writes unlocked per-range gain calibration factors.
    Scl,

    /// `OPT` — Option: accesses indexed EEPROM hardware and limit settings.
    Opt,

    /// `TMP` — Temperature: reports internal/external LM75 readings.
    Tmp,

    /// `TRM` — Trigger Mask: configures external trigger and LM75 enable bits.
    Trm,

    /// `WEN` — Write Enable: unlocks protected calibration and EEPROM setters.
    Wen,

    /// `ERC` — Error Count: reads or clears accumulated command errors.
    Erc,

    /// `SBD` — Serial Baud Divisor: stores the AVR UBRR value used with double-speed UART mode.
    Sbd,

    /// `NOP` — No Operation: validates framing and optional prompt behavior without changing state.
    Nop,

    /// Sentinel returned when no mnemonic matches.
    Err,
}

impl CmdWhich {
    /// Returns the exact three-letter EDL wire mnemonic, or `None` for the internal error sentinel.
    pub const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Str => Some("STR"),
            Self::Idn => Some("IDN"),
            Self::Chn => Some("CHN"),
            Self::Val => Some("VAL"),
            Self::Ena => Some("ENA"),
            Self::Dca => Some("DCA"),
            Self::Dcp => Some("DCP"),
            Self::Dcv => Some("DCV"),
            Self::Dcr => Some("DCR"),
            Self::Mah => Some("MAH"),
            Self::Mwh => Some("MWH"),
            Self::Msv => Some("MSV"),
            Self::Msa => Some("MSA"),
            Self::Rng => Some("RNG"),
            Self::Msw => Some("MSW"),
            Self::Pca => Some("PCA"),
            Self::Ron => Some("RON"),
            Self::Rof => Some("ROF"),
            Self::Rip => Some("RIP"),
            Self::Raw => Some("RAW"),
            Self::Dsp => Some("DSP"),
            Self::All => Some("ALL"),
            Self::Ofs => Some("OFS"),
            Self::Scl => Some("SCL"),
            Self::Opt => Some("OPT"),
            Self::Tmp => Some("TMP"),
            Self::Trm => Some("TRM"),
            Self::Wen => Some("WEN"),
            Self::Erc => Some("ERC"),
            Self::Sbd => Some("SBD"),
            Self::Nop => Some("NOP"),
            Self::Err => None,
        }
    }

    /// Parses an EDL mnemonic without allocation, accepting ASCII case differences and surrounding whitespace.
    pub fn from_mnemonic(keyword: &str) -> Self {
        let keyword = keyword.trim();
        if keyword.eq_ignore_ascii_case("STR") { Self::Str }
        else if keyword.eq_ignore_ascii_case("IDN") { Self::Idn }
        else if keyword.eq_ignore_ascii_case("CHN") { Self::Chn }
        else if keyword.eq_ignore_ascii_case("VAL") { Self::Val }
        else if keyword.eq_ignore_ascii_case("ENA") { Self::Ena }
        else if keyword.eq_ignore_ascii_case("DCA") { Self::Dca }
        else if keyword.eq_ignore_ascii_case("DCP") { Self::Dcp }
        else if keyword.eq_ignore_ascii_case("DCV") { Self::Dcv }
        else if keyword.eq_ignore_ascii_case("DCR") { Self::Dcr }
        else if keyword.eq_ignore_ascii_case("MAH") { Self::Mah }
        else if keyword.eq_ignore_ascii_case("MWH") { Self::Mwh }
        else if keyword.eq_ignore_ascii_case("MSV") { Self::Msv }
        else if keyword.eq_ignore_ascii_case("MSA") { Self::Msa }
        else if keyword.eq_ignore_ascii_case("RNG") { Self::Rng }
        else if keyword.eq_ignore_ascii_case("MSW") { Self::Msw }
        else if keyword.eq_ignore_ascii_case("PCA") { Self::Pca }
        else if keyword.eq_ignore_ascii_case("RON") { Self::Ron }
        else if keyword.eq_ignore_ascii_case("ROF") { Self::Rof }
        else if keyword.eq_ignore_ascii_case("RIP") { Self::Rip }
        else if keyword.eq_ignore_ascii_case("RAW") { Self::Raw }
        else if keyword.eq_ignore_ascii_case("DSP") { Self::Dsp }
        else if keyword.eq_ignore_ascii_case("ALL") { Self::All }
        else if keyword.eq_ignore_ascii_case("OFS") { Self::Ofs }
        else if keyword.eq_ignore_ascii_case("SCL") { Self::Scl }
        else if keyword.eq_ignore_ascii_case("OPT") { Self::Opt }
        else if keyword.eq_ignore_ascii_case("TMP") { Self::Tmp }
        else if keyword.eq_ignore_ascii_case("TRM") { Self::Trm }
        else if keyword.eq_ignore_ascii_case("WEN") { Self::Wen }
        else if keyword.eq_ignore_ascii_case("ERC") { Self::Erc }
        else if keyword.eq_ignore_ascii_case("SBD") { Self::Sbd }
        else if keyword.eq_ignore_ascii_case("NOP") { Self::Nop }
        else { Self::Err }
    }

    /// Returns the named command's Pascal base subchannel before a numeric argument is added.
    pub const fn sub_channel_offset(self) -> Option<u8> {
        match self {
            Self::Str => Some(255),
            Self::Idn => Some(254),
            Self::Chn => Some(253),
            Self::Val | Self::Ena | Self::Nop => Some(0),
            Self::Dca => Some(1),
            Self::Dcp => Some(3),
            Self::Dcv => Some(4),
            Self::Dcr => Some(5),
            Self::Mah => Some(7),
            Self::Mwh => Some(8),
            Self::Msv => Some(10),
            Self::Msa => Some(11),
            Self::Rng => Some(19),
            Self::Msw => Some(18),
            Self::Pca => Some(21),
            Self::Ron => Some(27),
            Self::Rof => Some(28),
            Self::Rip => Some(29),
            Self::Raw => Some(50),
            Self::Dsp => Some(80),
            Self::All => Some(99),
            Self::Ofs => Some(100),
            Self::Scl => Some(200),
            Self::Opt => Some(150),
            Self::Tmp => Some(233),
            Self::Trm => Some(240),
            Self::Wen => Some(250),
            Self::Erc => Some(251),
            Self::Sbd => Some(252),
            Self::Err => None,
        }
    }
}
