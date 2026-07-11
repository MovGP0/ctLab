use super::*;

#[derive(Debug, Clone)]
pub(super) struct EepromDefaults {
    pub(super) ee_initialized: u16,
    pub(super) init_frequenz: i32,
    pub(super) init_level: f64,
    pub(super) init_burst: u8,
    pub(super) init_wave: u8,
    pub(super) init_pwr_gain: f64,
    pub(super) init_attn_fac: f64,
    pub(super) init_inc_rast: i32,
    pub(super) init_terz_num: u8,
    pub(super) level_scale_low: f64,
    pub(super) level_scale_hi: f64,
    pub(super) ee_ser_baud_reg: u8,
}
impl Default for EepromDefaults {
    fn default() -> Self {
        Self {
            ee_initialized: EEPROM_INITIALIZED,
            init_frequenz: 10_000,
            init_level: 5_000.0,
            init_burst: 0,
            // SQG powers up in square-wave mode unless EEPROM overrides it.
            init_wave: C_SQUW,
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
