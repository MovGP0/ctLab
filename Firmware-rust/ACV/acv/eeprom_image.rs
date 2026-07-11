#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone)]
pub(super) struct EepromImage {
    pub(super) ee_initialized: u16,
    pub(super) init_inc_rast: i32,
    pub(super) init_gain: u8,
    pub(super) init_rate: Spdif,
    pub(super) init_aux_cmd: u8,
    pub(super) ee_ser_baud_reg: u8,
    pub(super) adc_scales_l: [u16; 8],
    pub(super) adc_scales_r: [u16; 8],
    pub(super) init_lr_swap: bool,
}

impl Default for EepromImage {
    fn default() -> Self {
        Self {
            ee_initialized: EE_INITIALIZED_MAGIC,
            init_inc_rast: 4,
            init_gain: 2,
            init_rate: Spdif::C48Khz,
            init_aux_cmd: 7,
            ee_ser_baud_reg: 51,
            adc_scales_l: [2100, 664, 2100, 664, 2100, 664, 2100, 664],
            adc_scales_r: [2100, 664, 2100, 664, 2100, 664, 2100, 664],
            init_lr_swap: false,
        }
    }
}
