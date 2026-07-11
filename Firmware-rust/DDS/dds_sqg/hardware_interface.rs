use super::*;

pub(super) trait HardwareInterface {
    fn serout_byte(&mut self, byte: u8);
    fn write_serial(&mut self, text: &str);
    fn send_dds_word(&mut self, word: u16);
    fn shift_out_level_sr(&mut self, level: i32, switch_state: u8);
    fn shift_out_offset_dac(&mut self, dac_counts: i16);
    fn serial_timeout_char(&mut self, timeout_ticks: u8) -> Option<char>;
    fn serial_pending(&self) -> bool;
    fn take_systick(&mut self) -> bool;
    fn next_panel_event(&mut self) -> PanelEvent;
    fn set_serial_baud_register(&mut self, register: u8, double_speed: bool);
    fn read_slave_channel(&mut self) -> u8;
    fn serial_read_immediate(&mut self) -> Option<char>;
    fn lcd_setup(&mut self) -> bool;
    fn lcd_define_custom_char(&mut self, slot: u8, bitmap: [u8; 8]);
    fn lcd_write_line(&mut self, row: u8, text: &str);
    fn set_activity_led(&mut self, active_low: bool);
    fn delay_ms(&mut self, ms: u16);
}
