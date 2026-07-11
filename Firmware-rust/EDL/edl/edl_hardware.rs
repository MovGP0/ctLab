use super::*;

pub trait EdlHardware {
    fn read_voltage_adc16(&mut self, on_phase: bool) -> u16;
    fn read_current_adc16(&mut self, on_phase: bool) -> u16;
    fn read_voltage_adc10(&mut self) -> i16;
    fn read_current_adc10(&mut self) -> i16;
    fn set_shunt(&mut self, shunt_index: u8);
    fn set_output_enabled(&mut self, enabled: bool);
    fn set_dac_raw(&mut self, raw: u16);
    fn read_temp_c(&mut self) -> Option<Float>;
    fn lm75_write(&mut self, address: u8, register: u8, data: &[u8]);
    fn serial_write(&mut self, text: &str);
    fn lcd_write_line(&mut self, row: u8, text: &str);
    fn read_trigger_in(&mut self) -> bool;
}
