//! Derives the DCG converter and relay calibration used by every engineering-unit conversion.

use super::*;

/// Derived engineering-units-per-count factors. They are recomputed from EEPROM whenever options or ranges change.
#[derive(Debug, Clone)]
pub struct CalibrationScale {
    /// Copies the EEPROM hardware-option bytes used to choose installed ADC, DAC, and daughterboard paths.
    pub options: u8,

    /// Voltage represented by one DAC count after the active converter width and voltage range are applied.
    pub dac_lsb_u: [Float; 2],

    /// Current represented by one DAC count after the active converter width and shunt are applied.
    pub dac_lsb_i: [Float; 4],

    /// Volts represented by one ADC count for each of the two voltage ranges, indexed by `VoltageRange` after offset subtraction.
    pub adc_lsb_u: [Float; 2],

    /// Amperes represented by one ADC count for each of the four shunt ranges, indexed by `CurrentRange` after offset subtraction.
    pub adc_lsb_i: [Float; 4],

    /// Maximum raw code of the selected 12- or 16-bit DAC, used to clamp every calibrated output.
    pub dac_max: u16,

    /// Calibrated voltage threshold around which relay hysteresis changes the DCG voltage range.
    pub switchpoint: Float,

    /// Lower voltage threshold used with `relay_high` to provide hysteresis around a DCG relay transition.
    pub relay_low: Float,

    /// Upper voltage threshold used with `relay_low` to provide hysteresis around a DCG relay transition.
    pub relay_high: Float,

    /// Selects the LTC1655 16-bit output framing when the corresponding hardware option is installed.
    pub dac16_present: bool,

    /// Records that the external 16-bit ADC is installed and its scale factors are active.
    ///
    /// Measurement code uses this flag to choose the external converter instead
    /// of applying 16-bit calibration to the ATmega's 10-bit ADC readings.
    pub adc16_present: bool,

    /// Records installation of the DC power daughterboard so its relay and sensing paths are only used when available.
    pub dcp_present: bool,
}
impl Default for CalibrationScale {
    /// Starts with neutral conversion factors and absent optional hardware until EEPROM-derived calibration is initialized.
    fn default() -> Self {
        Self {
            options: 0,
            dac_lsb_u: [0.0; 2],
            dac_lsb_i: [0.0; 4],
            adc_lsb_u: [0.0; 2],
            adc_lsb_i: [0.0; 4],
            dac_max: 65_535,
            switchpoint: 0.0,
            relay_low: 0.0,
            relay_high: 0.0,
            dac16_present: false,
            adc16_present: false,
            dcp_present: false,
        }
    }
}
