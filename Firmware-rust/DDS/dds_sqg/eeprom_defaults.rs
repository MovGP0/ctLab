//! Defines reset values copied from SQG EEPROM into live state.

use super::*;

/// SQG EEPROM defaults mirrored separately from mutable runtime values so reset and write-enable behavior match Pascal.
#[derive(Debug, Clone)]
pub(super) struct EepromDefaults {
    /// Stores the EEPROM format marker; startup loads factory defaults when this byte does not match the expected signature.
    pub(super) ee_initialized: u16,

    /// Persisted SQG startup frequency in tenths of a hertz, copied before AD9833 tuning-word generation.
    pub(super) init_frequenz: i32,

    /// Persisted SQG startup output level used to initialize amplitude-DAC conversion.
    pub(super) init_level: f64,

    /// Persisted startup burst interval used to initialize continuous or gated waveform operation.
    pub(super) init_burst: u8,

    /// Persisted startup waveform code decoded before the first AD9833 control write.
    pub(super) init_wave: u8,

    /// Persisted output-stage gain used by RMS, peak, dB, and amplitude-DAC conversion.
    pub(super) init_pwr_gain: f64,

    /// Persisted attenuation correction applied when the low-level relay range is active.
    pub(super) init_attn_fac: f64,

    /// Persisted number of quadrature edges per logical panel encoder detent.
    pub(super) init_inc_rast: i32,

    /// Persisted index into the preferred one-third-octave frequency table.
    pub(super) init_terz_num: u8,

    /// Persisted conversion from requested DDS level to amplitude-DAC code in the attenuated low-level range.
    pub(super) level_scale_low: f64,

    /// Persisted conversion from requested SQG level to amplitude-DAC code in the high-level range.
    pub(super) level_scale_hi: f64,

    /// Stores the AVR UART divisor selected by the protected `SBD` command for reuse after reset.
    pub(super) ee_ser_baud_reg: u8,
}
impl Default for EepromDefaults {
    /// Builds the SQG factory EEPROM image, including frequency, level, waveform, calibration, and UART defaults.
    fn default() -> Self {
        Self {
            ee_initialized: EEPROM_INITIALIZED,
            init_frequenz: 10_000,
            init_level: 5_000.0,
            init_burst: 0,
            // SQG powers up in square-wave mode unless EEPROM overrides it.
            init_wave: Waveform::Square.as_byte(),
            init_pwr_gain: 2.0,
            init_attn_fac: 40.0,
            init_inc_rast: 4,
            init_terz_num: 9,
            level_scale_low: 1.0,
            level_scale_hi: 1.0,
            ee_ser_baud_reg: 51,
        }
    }
}
