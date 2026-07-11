use super::*;

#[derive(Debug, Clone)]
pub struct CalibrationScale {
    pub options: u8,
    pub dac_lsb_u: [Float; 2],
    pub dac_lsb_i: [Float; 4],
    pub adc_lsb_u: [Float; 2],
    pub adc_lsb_i: [Float; 4],
    pub dac_max: u16,
    pub switchpoint: Float,
    pub relay_low: Float,
    pub relay_high: Float,
    pub dac16_present: bool,
    pub adc16_present: bool,
    pub dcp_present: bool,
}
impl Default for CalibrationScale {
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
