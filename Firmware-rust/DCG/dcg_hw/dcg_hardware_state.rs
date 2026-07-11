//! Implements DCG bit-level DAC/ADC transactions and interrupt-time sequencing.

/// Low-level DCG shift-register and timer state shared by foreground setup and the periodic interrupt path.
#[derive(Debug, Clone)]
pub struct DcgHardwareState {
    /// Scratch word shifted most-significant bit first; destructive shifts leave the retained calibrated DAC words untouched.
    pub dac_temp: u16,

    /// Scratch register assembled one bit at a time during an LTC1864 transfer so the previous complete raw sample remains intact until framing finishes.
    pub adc_temp: u16,

    /// Latest unscaled voltage-converter code, retained for `RAW` diagnostics before the active range's offset and volts-per-count factor are applied.
    pub adc_raw_u: u16,

    /// Latest unscaled current-converter code, retained for `RAW` diagnostics before the active shunt's offset and amperes-per-count factor are applied.
    pub adc_raw_i: u16,

    /// Calibrated voltage DAC code used during the energized ripple phase.
    pub dac_raw_u_on: u16,

    /// Calibrated voltage DAC code used during the off ripple phase.
    pub dac_raw_u_off: u16,

    /// Calibrated current-limit DAC code for the active shunt range.
    pub dac_raw_i: u16,

    /// Remaining timer ticks in the current DCG ripple phase before the hardware interrupt swaps DAC words.
    pub pw_counter: u16,

    /// Caches `pw_on_time` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub pw_on_time: u16,

    /// Caches `pw_off_time` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub pw_off_time: u16,

    /// Caches `pw_on_off` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub pw_on_off: bool,

    /// Caches `ui_toggle` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub ui_toggle: bool,

    /// Selects LTC1864 measurements when the sixteen-bit converter option is installed; otherwise conversions use AVR ADC10 channels.
    ///
    /// Converter width changes both the hardware sequence and the counts used by
    /// later engineering-unit conversion, so the interrupt path retains it here.
    pub adc16_present: bool,

    /// Selects the LTC1655 16-bit output framing when the corresponding hardware option is installed.
    pub dac16_present: bool,
}
impl Default for DcgHardwareState {
    /// Creates a de-energized DCG hardware image with zero DAC words, cleared samples, and ripple timing inactive.
    fn default() -> Self {
        Self {
            dac_temp: 0,
            adc_temp: 0,
            adc_raw_u: 0,
            adc_raw_i: 0,
            dac_raw_u_on: 0,
            dac_raw_u_off: 0,
            dac_raw_i: 0,
            pw_counter: 0,
            pw_on_time: 0,
            pw_off_time: 0,
            pw_on_off: false,
            ui_toggle: false,
            adc16_present: false,
            dac16_present: false,
        }
    }
}
