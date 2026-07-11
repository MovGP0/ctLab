//! Defines the decoded command identities whose discriminants match the legacy mnemonic tables.

/// Decoded command identity kept separate from the wire mnemonic so dispatch remains exhaustive when protocol aliases share a subchannel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CmdWhich {
    /// STR — Status Request: returns the legacy status text and error details; set syntax only affects prompt/echo handling.
    Str,

    /// IDN — Identification: returns the firmware identification banner and does not mutate instrument state.
    Idn,

    /// TRG — Trigger: executes the configured DDS trigger action after parsing, rather than storing a new continuous setpoint.
    Trg,

    /// VAL — Value: bypasses mnemonic lookup and addresses the following numeric subchannel directly for generic read/write access.
    Val,

    /// FRQ — Frequency: reads or sets DDS output frequency in the protocol's tenths-of-hertz unit before tuning-word generation.
    Frq,

    /// LVL — RMS Level: reads or sets the DDS output amplitude as RMS millivolts and converts it through the active attenuation calibration.
    Lvl,

    /// LVP — Level Peak Voltage: reads or sets peak millivolts, converting to the shared amplitude-DAC setpoint.
    Lvp,

    /// DBU — Decibels Unloaded: reads or sets RMS level relative to 0.774597 volts.
    Dbu,

    /// WAV — Waveform: reads or selects the AD9833 mode and matching analog, square, logic, or external relay route.
    Wav,

    /// BST — Burst: reads or sets the burst interval that periodically gates the configured DDS waveform off and on.
    Bst,

    /// INL — Input Level: returns calibrated RMS input amplitude, including overload reporting for a clipped measurement path.
    Inl,

    /// RNG — Range: reads or selects the calibrated analog input gain range instead of relying on automatic range switching.
    Rng,

    /// OFS — Offset: reads or writes the indexed EEPROM converter zero correction after `WEN` authorization.
    Ofs,

    /// DSP — Display: reads or changes the front-panel page/edit selection used by local interaction.
    Dsp,

    /// ALL — All Measurements: returns the module's aggregate set of live measurements in one request and has no set operation.
    All,

    /// SCL — Scale: reads or writes the indexed EEPROM gain correction and rebuilds the affected engineering-unit conversion.
    Scl,

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
    /// Returns the exact mnemonic transmitted by the legacy SQG protocol.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Str => "STR",
            Self::Idn => "IDN",
            Self::Trg => "TRG",
            Self::Val => "VAL",
            Self::Frq => "FRQ",
            Self::Lvl => "LVL",
            Self::Lvp => "LVP",
            Self::Dbu => "DBU",
            Self::Wav => "WAV",
            Self::Bst => "BST",
            Self::Inl => "INL",
            Self::Rng => "RNG",
            Self::Ofs => "DCO",
            Self::Dsp => "DSP",
            Self::All => "ALL",
            Self::Scl => "SCL",
            Self::Wen => "WEN",
            Self::Erc => "ERC",
            Self::Sbd => "SBD",
            Self::Nop => "NOP",
            Self::Err => "ERR",
        }
    }

    /// Decodes an SQG mnemonic without allocating or changing case-insensitive command handling.
    pub(super) fn from_mnemonic(value: &str) -> Self {
        Self::ALL
            .iter()
            .copied()
            .find(|command| command.as_str().eq_ignore_ascii_case(value))
            .unwrap_or(Self::Err)
    }

    /// Returns the first numeric subchannel in the command's protocol block.
    pub(super) const fn default_subchannel(self) -> u8 {
        match self {
            Self::Str => 255,
            Self::Idn => 254,
            Self::Trg => 249,
            Self::Val | Self::Frq | Self::Nop | Self::Err => 0,
            Self::Lvl => 1,
            Self::Lvp => 2,
            Self::Dbu => 3,
            Self::Wav => 4,
            Self::Bst => 5,
            Self::Inl => 10,
            Self::Rng => 19,
            Self::Ofs => 20,
            Self::Dsp => 80,
            Self::All => 99,
            Self::Scl => 200,
            Self::Wen => 250,
            Self::Erc => 251,
            Self::Sbd => 252,
        }
    }

    #[rustfmt::skip]
    const ALL: [Self; 20] = [
        Self::Str,
        Self::Idn,
        Self::Trg,
        Self::Val,
        Self::Frq,
        Self::Lvl,
        Self::Lvp,
        Self::Dbu,
        Self::Wav,
        Self::Bst,
        Self::Inl,
        Self::Rng,
        Self::Ofs,
        Self::Dsp,
        Self::All,
        Self::Scl,
        Self::Wen,
        Self::Erc,
        Self::Sbd,
        Self::Nop,
    ];
}
