#[allow(unused_imports)]
use super::*;

pub trait AdaHardware {
    fn get_adc(&mut self, channel_1_based: u8) -> i16;
    fn twi_out(&mut self, slave_addr: u8, command: u16) -> bool;
    fn shift_out_sr(&mut self, port_array: &[u8; 8]);
    fn read_io_pin(&mut self, port: u8) -> u8;
    fn write_io_dir(&mut self, port: u8, value: u8);
    fn detect_i2c_expander(&mut self) -> bool;
    fn detect_sense(&mut self) -> bool;
    fn read_slave_channel(&mut self) -> u8;
    fn set_external_trigger_edge(&mut self, positive: bool);
    fn enable_interrupts(&mut self);
    fn twi_inp_byte(&mut self, slave_addr: u8) -> u8;
    fn twi_inp_word(&mut self, slave_addr: u8) -> u16;
    fn serial_read_byte_timeout(&mut self, timeout_ms: u16) -> Option<u8>;
    fn serial_write(&mut self, text: &str);
    fn set_serial_baud(&mut self, ubrr: u8, double_speed: bool);
    fn set_internal_reference(&mut self, internal: bool);
    fn set_sdataout(&mut self, high: bool);
    fn set_str_dac(&mut self, high: bool);
    fn set_str_ad16(&mut self, high: bool);
    fn set_trigger_led(&mut self, active: bool);
}
