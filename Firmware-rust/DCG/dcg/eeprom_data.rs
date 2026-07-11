use super::*;

#[derive(Debug, Clone)]
pub struct EepromData {
    pub dac_u_offsets: [i16; 2],
    pub dac_u_scales: [Float; 2],
    pub dac_i_offsets: [i16; 4],
    pub dac_i_scales: [Float; 4],
    pub adc_u_offsets: [i16; 2],
    pub adc_u_scales: [Float; 2],
    pub adc_i_offsets: [i16; 4],
    pub adc_i_scales: [Float; 4],
    pub option_array: [Float; OPTION_ARRAY_LEN],
    pub ee_ser_baud_reg: u8,
    pub inc_rast_def: i16,
}
impl Default for EepromData {
    fn default() -> Self {
        Self {
            dac_u_offsets: [10, 10],
            dac_u_scales: [1.001, 1.0032],
            dac_i_offsets: [10, 10, 10, 10],
            dac_i_scales: [1.003, 1.003, 1.003, 1.003],
            adc_u_offsets: [-180, -180],
            adc_u_scales: [1.005, 1.005],
            adc_i_offsets: [-180, -180, -180, -180],
            adc_i_scales: [1.0, 1.0, 1.0, 1.0],
            option_array: [
                5.0, 0.02, 3.0, 3.0, 0.25, 2.5, 30.0, 470.0, 47.0, 4.7, 0.47, 0.002, 0.020, 0.200,
                2.000, 2.0, 6.0, 7.0, 12.1, 8.6, 8.9, 50.0, 0.0, 4.0, 6.0,
            ],
            ee_ser_baud_reg: 51,
            inc_rast_def: 4,
        }
    }
}
impl EepromData {
    pub fn init_volt(&self) -> Float {
        self.option_array[OPT_INIT_VOLT]
    }

    pub fn init_amp(&self) -> Float {
        self.option_array[OPT_INIT_AMP]
    }

    pub fn init_gain_pre(&self) -> Float {
        self.option_array[OPT_INIT_GAIN_PRE]
    }

    pub fn init_gain_out(&self) -> Float {
        self.option_array[OPT_INIT_GAIN_OUT]
    }

    pub fn init_gain_i(&self) -> Float {
        self.option_array[OPT_INIT_GAIN_I]
    }

    pub fn uref(&self) -> Float {
        self.option_array[OPT_UREF]
    }

    pub fn umax(&self) -> Float {
        self.option_array[OPT_UMAX]
    }

    pub fn rsense(&self, index: usize) -> Float {
        self.option_array[OPT_RSENSE_BASE + index.min(3)]
    }

    pub fn imax(&self, index: usize) -> Float {
        self.option_array[OPT_IMAX_BASE + index.min(3)]
    }

    pub fn adc_u_fac(&self, index: usize) -> Float {
        self.option_array[OPT_ADCUFAC_BASE + index.min(1)]
    }

    pub fn init_options(&self) -> u8 {
        self.option_array[OPT_INIT_OPTIONS] as u8
    }

    pub fn init_switch_u(&self) -> Float {
        self.option_array[OPT_INIT_SWITCH_U]
    }

    pub fn init_hyst_low(&self) -> Float {
        self.option_array[OPT_INIT_HYST_LOW]
    }

    pub fn init_hyst_high(&self) -> Float {
        self.option_array[OPT_INIT_HYST_HIGH]
    }

    pub fn init_fan_on_temp(&self) -> Float {
        self.option_array[OPT_INIT_FAN_ON_TEMP]
    }

    pub fn init_ripple_percent(&self) -> i32 {
        self.option_array[OPT_INIT_RIPPLE_PERCENT] as i32
    }

    pub fn init_ton_time(&self) -> u16 {
        self.option_array[OPT_INIT_TON_TIME] as u16
    }

    pub fn init_toff_time(&self) -> u16 {
        self.option_array[OPT_INIT_TOFF_TIME] as u16
    }
}
