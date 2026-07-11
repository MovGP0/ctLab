//! Defines the persisted calibration image and startup settings retained across reset.

use super::*;

/// EEPROM image and calibrated defaults. Keeping the persisted layout together prevents runtime setpoints from being mistaken for calibration data.
#[derive(Debug, Clone)]
pub struct EepromData {
    /// Per-voltage-range zero codes loaded from EEPROM before converting requested volts to raw DAC words.
    pub dac_u_offsets: [i16; 2],

    /// Per-voltage-range gain corrections applied when converting requested volts to DAC counts.
    pub dac_u_scales: [Float; 2],

    /// Per-shunt zero codes loaded from EEPROM before converting requested amperes to raw DAC words.
    pub dac_i_offsets: [i16; 4],

    /// Per-shunt gain corrections applied when converting requested amperes to DAC counts.
    pub dac_i_scales: [Float; 4],

    /// Persisted raw zero-code correction for each of the two voltage ranges, subtracted before voltage scaling.
    pub adc_u_offsets: [i16; 2],

    /// Persisted gain correction for each voltage range, used to derive calibrated volts per ADC count.
    pub adc_u_scales: [Float; 2],

    /// Persisted raw zero-code correction for each of the four current shunts, subtracted before current scaling.
    pub adc_i_offsets: [i16; 4],

    /// Persisted gain correction for each current shunt, used to derive calibrated amperes per ADC count.
    pub adc_i_scales: [Float; 4],

    /// Runtime copy of the 25-slot DCG EEPROM option image used by calibration accessors and protected serial writes.
    pub option_array: [Float; OPTION_ARRAY_LEN],

    /// Stores the AVR UART divisor selected by the protected `SBD` command for reuse after reset.
    pub ee_ser_baud_reg: u8,

    /// Persists the number of quadrature edges that constitute one panel encoder detent.
    pub inc_rast_def: i16,
}
impl Default for EepromData {
    /// Builds the module's factory EEPROM calibration and startup image used when persisted data is absent.
    #[rustfmt::skip]
    fn default() -> Self {
        Self {
            dac_u_offsets: [
                10,
                10,
            ],
            dac_u_scales: [
                1.001,
                1.0032,
            ],
            dac_i_offsets: [
                10,
                10,
                10,
                10,
            ],
            dac_i_scales: [
                1.003,
                1.003,
                1.003,
                1.003,
            ],
            adc_u_offsets: [
                -180,
                -180,
            ],
            adc_u_scales: [
                1.005,
                1.005,
            ],
            adc_i_offsets: [
                -180,
                -180,
                -180,
                -180,
            ],
            adc_i_scales: [
                1.0,
                1.0,
                1.0,
                1.0,
            ],
            option_array: [
                5.0,
                0.02,
                3.0,
                3.0,
                0.25,
                2.5,
                30.0,
                470.0,
                47.0,
                4.7,
                0.47,
                0.002,
                0.020,
                0.200,
                2.000,
                2.0,
                6.0,
                7.0,
                12.1,
                8.6,
                8.9,
                50.0,
                0.0,
                4.0,
                6.0,
            ],
            ee_ser_baud_reg: 51,
            inc_rast_def: 4,
        }
    }
}
impl EepromData {
    /// Initializes volt before dependent calculations or outputs are enabled.
    pub fn init_volt(&self) -> Float {
        self.option_array[OPT_INIT_VOLT]
    }

    /// Initializes amp before dependent calculations or outputs are enabled.
    pub fn init_amp(&self) -> Float {
        self.option_array[OPT_INIT_AMP]
    }

    /// Initializes gain pre before dependent calculations or outputs are enabled.
    pub fn init_gain_pre(&self) -> Float {
        self.option_array[OPT_INIT_GAIN_PRE]
    }

    /// Initializes gain out before dependent calculations or outputs are enabled.
    pub fn init_gain_out(&self) -> Float {
        self.option_array[OPT_INIT_GAIN_OUT]
    }

    /// Initializes gain i before dependent calculations or outputs are enabled.
    pub fn init_gain_i(&self) -> Float {
        self.option_array[OPT_INIT_GAIN_I]
    }

    /// Returns the persisted ADC reference voltage option used to derive volts per converter count.
    pub fn uref(&self) -> Float {
        self.option_array[OPT_UREF]
    }

    /// Returns the persisted full-scale DCG voltage used to derive range limits and voltage DAC scaling.
    pub fn umax(&self) -> Float {
        self.option_array[OPT_UMAX]
    }

    /// Returns the persisted sense resistance for the bounded current-shunt index.
    pub fn rsense(&self, index: usize) -> Float {
        self.option_array[OPT_RSENSE_BASE + index.min(3)]
    }

    /// Returns the persisted full-scale current for the bounded shunt index.
    pub fn imax(&self, index: usize) -> Float {
        self.option_array[OPT_IMAX_BASE + index.min(3)]
    }

    /// Returns the persisted voltage ADC divider factor for the bounded voltage-range index.
    pub fn adc_u_fac(&self, index: usize) -> Float {
        self.option_array[OPT_ADCUFAC_BASE + index.min(1)]
    }

    /// Initializes options before dependent calculations or outputs are enabled.
    pub fn init_options(&self) -> u8 {
        self.option_array[OPT_INIT_OPTIONS] as u8
    }

    /// Initializes switch u before dependent calculations or outputs are enabled.
    pub fn init_switch_u(&self) -> Float {
        self.option_array[OPT_INIT_SWITCH_U]
    }

    /// Initializes hyst low before dependent calculations or outputs are enabled.
    pub fn init_hyst_low(&self) -> Float {
        self.option_array[OPT_INIT_HYST_LOW]
    }

    /// Initializes hyst high before dependent calculations or outputs are enabled.
    pub fn init_hyst_high(&self) -> Float {
        self.option_array[OPT_INIT_HYST_HIGH]
    }

    /// Initializes fan on temp before dependent calculations or outputs are enabled.
    pub fn init_fan_on_temp(&self) -> Float {
        self.option_array[OPT_INIT_FAN_ON_TEMP]
    }

    /// Initializes ripple percent before dependent calculations or outputs are enabled.
    pub fn init_ripple_percent(&self) -> i32 {
        self.option_array[OPT_INIT_RIPPLE_PERCENT] as i32
    }

    /// Initializes ton time before dependent calculations or outputs are enabled.
    pub fn init_ton_time(&self) -> u16 {
        self.option_array[OPT_INIT_TON_TIME] as u16
    }

    /// Initializes toff time before dependent calculations or outputs are enabled.
    pub fn init_toff_time(&self) -> u16 {
        self.option_array[OPT_INIT_TOFF_TIME] as u16
    }
}
