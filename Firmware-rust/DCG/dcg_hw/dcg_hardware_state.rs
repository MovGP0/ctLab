#[derive(Debug, Clone)]
pub struct DcgHardwareState {
    pub dac_temp: u16,
    pub adc_temp: u16,
    pub adc_raw_u: u16,
    pub adc_raw_i: u16,
    pub dac_raw_u_on: u16,
    pub dac_raw_u_off: u16,
    pub dac_raw_i: u16,
    pub pw_counter: u16,
    pub pw_on_time: u16,
    pub pw_off_time: u16,
    pub pw_on_off: bool,
    pub ui_toggle: bool,
    pub adc16_present: bool,
    pub dac16_present: bool,
}
impl Default for DcgHardwareState {
    fn default() -> Self {
        Self {
            dac_temp: 0,
            adc_temp: 0,
            adc_raw_u: 0,
            adc_raw_i: 0,
            dac_raw_u_on: 0,
            dac_raw_u_off: 0,
            dac_raw_i: 0,
            pw_counter: 0,
            pw_on_time: 0,
            pw_off_time: 0,
            pw_on_off: false,
            ui_toggle: false,
            adc16_present: false,
            dac16_present: false,
        }
    }
}
