use super::*;

#[derive(Debug, Clone, Copy)]
/// Calibration values derived from EEPROM for the active mode and installed DAC.
pub struct ScaleState {
    /// Packed hardware option byte selecting DAC and temperature features.
    pub options: u8,

    /// Wire protocol required by the installed DAC.
    pub dac_kind: DacKind,

    /// Full-scale DAC code, either 12-bit or 16-bit.
    pub dac_max: u16,

    /// Active voltage divider ratio selected from low/high mode.
    pub divider_u: Float,

    /// Active low/high voltage ADC zero correction.
    pub adc_u_offset: i16,

    /// Volts represented by one 16-bit ADC code.
    pub adc16_lsb_u: Float,

    /// Amperes represented by one DAC code for each shunt.
    pub dac_lsb_i: [Float; 4],

    /// Resistance-mode numerator represented by one DAC code for each shunt.
    pub dac_lsb_r: [Float; 4],

    /// Amperes represented by one ADC code for each shunt.
    pub adc16_lsb_i: [Float; 4],

    /// Lowest resistance safe for the calibrated current path.
    pub dc_ohm_min: Float,

    /// Highest resistance retaining useful DAC resolution.
    pub dc_ohm_max: Float,
}

impl Default for ScaleState {
    /// Starts conversion factors inert until [`DeviceState::init_scales`] supplies EEPROM values.
    fn default() -> Self {
        Self {
            options: 0,
            dac_kind: DacKind::Ltc8043,
            dac_max: 4095,
            divider_u: 1.0,
            adc_u_offset: 0,
            adc16_lsb_u: 0.0,
            dac_lsb_i: [0.0; 4],
            dac_lsb_r: [0.0; 4],
            adc16_lsb_i: [0.0; 4],
            dc_ohm_min: 0.0,
            dc_ohm_max: 0.0,
        }
    }
}
