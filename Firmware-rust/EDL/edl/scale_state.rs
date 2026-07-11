use super::*;

#[derive(Debug, Clone, Copy)]
pub struct ScaleState {
    pub options: u8,
    pub dac_kind: DacKind,
    pub dac_max: u16,
    pub divider_u: Float,
    pub adc_u_offset: i16,
    pub adc16_lsb_u: Float,
    pub dac_lsb_i: [Float; 4],
    pub dac_lsb_r: [Float; 4],
    pub adc16_lsb_i: [Float; 4],
    pub dc_ohm_min: Float,
    pub dc_ohm_max: Float,
}

impl Default for ScaleState {
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
