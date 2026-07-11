use super::*;

#[derive(Debug, Clone)]
pub struct EepromData {
    pub adc_u_offsets: [i16; 2],
    pub adc_u_scales: [Float; 2],
    pub adc_i_offsets: [i16; 4],
    pub adc_i_scales: [Float; 4],
    pub dac_i_offsets: [i16; 4],
    pub dac_i_scales: [Float; 4],
    pub dac_r_scales: [Float; 4],
    pub option_array: [Float; 22],
    pub ee_ser_baud_reg: u8,
    pub inc_rast_def: i16,
    pub trig_mask: u8,
    pub ee_initialised: u16,
    pub first_run: bool,
}

impl Default for EepromData {
    fn default() -> Self {
        Self {
            adc_u_offsets: [-260, -260],
            adc_u_scales: [1.01, 1.01],
            adc_i_offsets: [-260, -260, -260, -260],
            adc_i_scales: [1.01, 1.01, 1.01, 1.01],
            dac_i_offsets: [0, 0, 0, 0],
            dac_i_scales: [1.0, 1.0, 1.0, 1.0],
            dac_r_scales: [1.0, 1.0, 1.0, 1.0],
            option_array: [
                0.0, 0.02, 2.5, 10.0, 10.0, 2.5, 50.0, 100.0, 10.0, 1.0, 0.1, 0.002, 0.020, 0.200,
                2.0, 25.0, 6.1, 4.0, 0.0, 10.0, 0.0, 50.0,
            ],
            ee_ser_baud_reg: 51,
            inc_rast_def: 4,
            trig_mask: 0,
            ee_initialised: 0xAA55,
            first_run: false,
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

    pub fn init_low_divider_u(&self) -> Float {
        self.option_array[OPT_INIT_LOW_DIVIDER_U]
    }

    pub fn init_hi_divider_u(&self) -> Float {
        self.option_array[OPT_INIT_HI_DIVIDER_U]
    }

    pub fn init_gain_i(&self) -> Float {
        self.option_array[OPT_INIT_GAIN_I]
    }

    pub fn uref(&self) -> Float {
        self.option_array[OPT_UREF]
    }

    pub fn pmax(&self) -> Float {
        self.option_array[OPT_PMAX]
    }

    pub fn rsense(&self, index: usize) -> Float {
        self.option_array[OPT_RSENSE_BASE + index]
    }

    pub fn imax(&self, index: usize) -> Float {
        self.option_array[OPT_IMAX_BASE + index]
    }

    pub fn voltage_limit_hi(&self) -> Float {
        self.option_array[OPT_UMAX_HI]
    }

    pub fn voltage_limit_lo(&self) -> Float {
        self.option_array[OPT_UMAX_LO]
    }

    pub fn init_options(&self) -> u8 {
        self.option_array[OPT_INIT_OPTIONS] as u8
    }

    pub fn init_i_percent(&self) -> i32 {
        self.option_array[OPT_INIT_IPERCENT] as i32
    }

    pub fn init_ton(&self) -> i32 {
        self.option_array[OPT_INIT_TON] as i32
    }

    pub fn init_toff(&self) -> i32 {
        self.option_array[OPT_INIT_TOFF] as i32
    }

    pub fn init_fan_on_temp(&self) -> Float {
        self.option_array[OPT_INIT_FAN_TEMP]
    }
}
