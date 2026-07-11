//! Defines the decoded command identities whose discriminants match the legacy mnemonic tables.

/// Decoded command identity kept separate from the wire mnemonic so dispatch remains exhaustive when protocol aliases share a subchannel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdWhich {
    /// STR — Status Request: returns the legacy status text and error details; set syntax only affects prompt/echo handling.
    Str,

    /// IDN — Identification: returns the firmware identification banner and does not mutate instrument state.
    Idn,

    /// CHN — Channel: reads or changes the multidrop channel used to accept addressed DCG frames and prefix replies.
    Chn,

    /// VAL — Value: bypasses mnemonic lookup and addresses the following numeric subchannel directly for generic read/write access.
    Val,

    /// DCV — Direct Voltage: reads or sets the DCG voltage setpoint, with range selection and limit correction applied before output programming.
    Dcv,

    /// DCA — Direct Current: reads or sets the DCG current-limit setpoint in amperes for the active shunt range.
    Dca,

    /// MAH — Milliampere Hours: returns the DCG's integrated charge accumulator and does not accept a set value.
    Mah,

    /// MWH — Milliwatt Hours: returns the DCG's integrated energy accumulator and does not accept a set value.
    Mwh,

    /// MSV — Measured Voltage: returns the calibrated live DCG output voltage rather than the requested setpoint.
    Msv,

    /// MSA — Measured Amperage: returns the calibrated live DCG output current rather than the current-limit setpoint.
    Msa,

    /// MSW — Measured Watts: returns live output power derived from the paired voltage and current samples.
    Msw,

    /// PCV — Percent Control Voltage: reads or sets the percentage modifier applied within the forced DCG voltage range without relay auto-ranging.
    Pcv,

    /// PCA — Percent Control Amperage: reads or sets the percentage modifier applied within the forced DCG current range without shunt auto-ranging.
    Pca,

    /// PWON — Pulse-Width On Time: reads or sets the milliseconds spent at the energized DCG ripple voltage.
    Pwon,

    /// PWOFF — Pulse-Width Off Time: reads or sets the milliseconds spent at the reduced DCG ripple voltage.
    Pwoff,

    /// RIP — Ripple: reads or sets the percentage voltage reduction used to derive the DCG off-phase DAC word.
    Rip,

    /// RAW — Raw Converter Value: returns unscaled ADC or DAC counts for the indexed calibration subchannel.
    Raw,

    /// DSP — Display: reads or changes the front-panel page/edit selection used by local interaction.
    Dsp,

    /// OFS — Offset: reads or writes the indexed EEPROM converter zero correction after `WEN` authorization.
    Ofs,

    /// SCL — Scale: reads or writes the indexed EEPROM gain correction and rebuilds the affected engineering-unit conversion.
    Scl,

    /// OPT — Option: reads or writes an indexed EEPROM hardware option controlling installed converter and board paths.
    Opt,

    /// ALL — All Measurements: returns the module's aggregate set of live measurements in one request and has no set operation.
    All,

    /// TMP — Temperature: returns the latest LM75 reading used by thermal protection and fan control.
    Tmp,

    /// WEN — Write Enable: arms the protected EEPROM path so one subsequent calibration, option, or baud write is accepted.
    Wen,

    /// ERC — Error Count: returns the number of parser errors accumulated since the previous counter reset.
    Erc,

    /// SBD — Set Baud: writes the protected EEPROM UART divisor and applies the corresponding serial rate.
    Sbd,

    /// NOP — No Operation: validates and acknowledges the frame without changing setpoints, EEPROM, or hardware.
    Nop,

    /// ERR — Invalid Command: parser sentinel used when mnemonic lookup fails, forcing a syntax-error reply instead of dispatching a subchannel.
    Err,
}
impl CmdWhich {
    /// Returns the exact mnemonic transmitted by the legacy DCG protocol.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Str => "STR",
            Self::Idn => "IDN",
            Self::Chn => "CHN",
            Self::Val => "VAL",
            Self::Dcv => "DCV",
            Self::Dca => "DCA",
            Self::Mah => "MAH",
            Self::Mwh => "MWH",
            Self::Msv => "MSV",
            Self::Msa => "MSA",
            Self::Msw => "MSW",
            Self::Pcv => "PCV",
            Self::Pca => "PCA",
            Self::Pwon => "RON",
            Self::Pwoff => "ROF",
            Self::Rip => "RIP",
            Self::Raw => "RAW",
            Self::Dsp => "DSP",
            Self::Ofs => "OFS",
            Self::Scl => "SCL",
            Self::Opt => "OPT",
            Self::All => "ALL",
            Self::Tmp => "TMP",
            Self::Wen => "WEN",
            Self::Erc => "ERC",
            Self::Sbd => "SBD",
            Self::Nop => "NOP",
            Self::Err => "ERR",
        }
    }

    /// Decodes a DCG mnemonic without allocating or changing the protocol's case-insensitive behavior.
    pub fn from_str(value: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|command| command.as_str().eq_ignore_ascii_case(value))
            .unwrap_or(Self::Err)
    }

    /// Returns the first numeric subchannel in the command's protocol block.
    pub const fn default_subchannel(self) -> u8 {
        match self {
            Self::Str => 255,
            Self::Idn => 254,
            Self::Chn => 253,
            Self::Val | Self::Dcv | Self::Nop | Self::Err => 0,
            Self::Dca => 1,
            Self::Mah => 7,
            Self::Mwh => 8,
            Self::Msv => 10,
            Self::Msa => 11,
            Self::Msw => 18,
            Self::Pcv => 20,
            Self::Pca => 21,
            Self::Pwon => 27,
            Self::Pwoff => 28,
            Self::Rip => 29,
            Self::Raw => 50,
            Self::Dsp => 80,
            Self::Ofs => 100,
            Self::Scl => 200,
            Self::Opt => 150,
            Self::All => 99,
            Self::Tmp => 233,
            Self::Wen => 250,
            Self::Erc => 251,
            Self::Sbd => 252,
        }
    }

    #[rustfmt::skip]
    const ALL: [Self; 27] = [
        Self::Str,
        Self::Idn,
        Self::Chn,
        Self::Val,
        Self::Dcv,
        Self::Dca,
        Self::Mah,
        Self::Mwh,
        Self::Msv,
        Self::Msa,
        Self::Msw,
        Self::Pcv,
        Self::Pca,
        Self::Pwon,
        Self::Pwoff,
        Self::Rip,
        Self::Raw,
        Self::Dsp,
        Self::Ofs,
        Self::Scl,
        Self::Opt,
        Self::All,
        Self::Tmp,
        Self::Wen,
        Self::Erc,
        Self::Sbd,
        Self::Nop,
    ];
}
