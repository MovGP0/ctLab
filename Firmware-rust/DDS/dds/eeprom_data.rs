//! Defines the persisted calibration image and startup settings retained across reset.

use super::*;

/// EEPROM image and calibrated defaults. Keeping the persisted layout together prevents runtime setpoints from being mistaken for calibration data.
#[derive(Debug, Clone)]
pub struct EepromData {
    /// Stores the EEPROM format marker; startup loads factory defaults when this byte does not match the expected signature.
    pub ee_initialized: u16,

    /// Persisted startup DDS frequency in tenths of a hertz, copied to the live setpoint before the first tuning-word write.
    pub init_frequency_tenths_hz: i32,

    /// Persisted startup logic-output amplitude in millivolts.
    pub init_logic_level_mv: Float,

    /// Persisted startup analog output RMS level in millivolts.
    pub init_level_mv: Float,

    /// Persisted startup logarithmic output level used to initialize DBU display/set behavior.
    pub init_db: Float,

    /// Persisted startup waveform code decoded before the first AD9833 control write.
    pub init_wave: u8,

    /// Persisted startup burst interval used to initialize continuous or gated waveform operation.
    pub init_burst: u8,

    /// Persisted startup DC offset in volts, converted to millivolts for the offset DAC path.
    pub init_offset_v: Float,

    /// Persisted output-stage gain used by RMS, peak, dB, and amplitude-DAC conversion.
    pub init_pwr_gain: Float,

    /// Persisted attenuation correction applied when the low-level relay range is active.
    pub init_attn_fac: Float,

    /// Persisted number of quadrature edges per logical panel encoder detent.
    pub init_inc_rast: i32,

    /// Persisted index into the preferred one-third-octave frequency table.
    pub init_terz_num: u8,

    /// Persisted conversion from requested DDS level to amplitude-DAC code in the attenuated low-level range.
    pub level_scale_low: Float,

    /// Persisted conversion from requested DDS level to amplitude-DAC code in the high-level range.
    pub level_scale_high: Float,

    /// Persisted RMS input calibration factor for each of the four DDS input-gain ranges, indexed by `InputRange`.
    pub adc_scales: [Float; 4],

    /// Stores the AVR UART divisor selected by the protected `SBD` command for reuse after reset.
    pub ee_ser_baud_reg: u8,
}
impl Default for EepromData {
    /// Builds the module's factory EEPROM calibration and startup image used when persisted data is absent.
    #[rustfmt::skip]
    fn default() -> Self {
        Self {
            ee_initialized: EEPROM_INITIALIZED,
            init_frequency_tenths_hz: 10_000,
            init_logic_level_mv: 5_000.0,
            init_level_mv: 774.6,
            init_db: 0.0,
            init_wave: 1,
            init_burst: 0,
            init_offset_v: 0.0,
            init_pwr_gain: 2.0,
            init_attn_fac: 40.0,
            init_inc_rast: 4,
            init_terz_num: 17,
            level_scale_low: 1.0,
            level_scale_high: 1.0,
            adc_scales: [
                1.0,
                1.0,
                1.0,
                1.0,
            ],
            ee_ser_baud_reg: 51,
        }
    }
}
