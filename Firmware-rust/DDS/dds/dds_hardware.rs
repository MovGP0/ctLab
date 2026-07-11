use super::*;

pub trait DdsHardware {
    fn send_dds_frequency_word(&mut self, word: u32);
    fn send_amplitude_word(&mut self, word: u16);
    fn set_waveform(&mut self, waveform: Waveform);
    fn set_input_range(&mut self, range: InputRange);
    fn send_aux_config(&mut self, value: u8);
    fn read_input_level(&mut self) -> Float;
    fn read_input_overload(&mut self) -> bool;
    fn serial_write(&mut self, text: &str);
    fn serial_read(&mut self) -> Option<char>;
    fn set_serial_baud_register(&mut self, register: u8, double_speed: bool);
    fn read_slave_channel(&mut self) -> u8;
    fn set_activity_led(&mut self, enabled: bool);
    fn delay_ms(&mut self, milliseconds: u16);
    fn lcd_setup(&mut self) -> bool;
    fn lcd_define_custom_char(&mut self, slot: u8, bitmap: [u8; 8]);
    fn lcd_write_line(&mut self, row: u8, text: &str);
}
