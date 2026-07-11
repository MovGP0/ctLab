use super::*;

#[derive(Debug, Clone)]
pub struct EepromData {
    pub ee_initialized: u16,
    pub init_frequency_tenths_hz: i32,
    pub init_logic_level_mv: Float,
    pub init_level_mv: Float,
    pub init_db: Float,
    pub init_wave: u8,
    pub init_burst: u8,
    pub init_offset_v: Float,
    pub init_pwr_gain: Float,
    pub init_attn_fac: Float,
    pub init_inc_rast: i32,
    pub init_terz_num: u8,
    pub level_scale_low: Float,
    pub level_scale_high: Float,
    pub adc_scales: [Float; 4],
    pub ee_ser_baud_reg: u8,
}
impl Default for EepromData {
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
            adc_scales: [1.0, 1.0, 1.0, 1.0],
            ee_ser_baud_reg: 51,
        }
    }
}
