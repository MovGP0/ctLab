use super::*;

pub trait DcgHardware {
    fn read_adc10(&mut self, channel_1_based: u8) -> i16;
    fn read_adc16_voltage(&mut self) -> u16;
    fn read_adc16_current(&mut self) -> u16;
    fn serial_read_timeout(&mut self, timeout_ms: u16) -> Option<char>;
    fn set_voltage_dac_raw(&mut self, raw: u16);
    fn set_current_dac_raw(&mut self, raw: u16);
    fn set_voltage_dac_off_raw(&mut self, raw: u16);
    fn delay_ms(&mut self, milliseconds: u16);
    fn set_current_range(&mut self, range: CurrentRange);
    fn set_voltage_range(&mut self, range: VoltageRange);
    fn set_input_relay_high(&mut self, high: bool);
    fn current_limit_sense(&mut self) -> bool;
    fn set_output_enabled(&mut self, enabled: bool);
    fn read_temp_c(&mut self) -> Option<Float>;
    fn serial_write(&mut self, text: &str);
    fn lcd_write_line(&mut self, row: u8, text: &str);
}
