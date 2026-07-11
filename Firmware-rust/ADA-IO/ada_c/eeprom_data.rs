#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone)]
pub struct EepromData {
    pub offset_array: [i16; 28],
    pub scale_array: [Float; 30],
    pub dir_init_array: [u8; 8],
    pub trig_mask_array: [u8; 4],
    pub trig_level: u8,
    pub trig_timer_value: u16,
    pub init_integrate_ad16: bool,
    pub ext_ref: u8,
    pub inc_rast_def: i16,
    pub ee_ser_baud_reg: u8,
    pub param_text_array: [String; 38],
    pub ee_initialised: u16,
    pub port_init_array: [u8; 8],
}

impl Default for EepromData {
    fn default() -> Self {
        Self {
            offset_array: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -40, -40, -40, -40, -40, -40, -40, -40, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
            scale_array: [
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 100.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                1.0, 1.0, 0.0, 3185.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 200.0, 3200.0,
            ],
            dir_init_array: [0; 8],
            trig_mask_array: [0; 4],
            trig_level: 0,
            trig_timer_value: 0,
            init_integrate_ad16: false,
            ext_ref: 1,
            inc_rast_def: 4,
            ee_ser_baud_reg: 51,
            param_text_array: array::from_fn(|_| String::new()),
            ee_initialised: 0xAA55,
            port_init_array: [0; 8],
        }
    }
}
