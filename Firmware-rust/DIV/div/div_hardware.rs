#[allow(unused_imports)]
use super::*;

pub trait DivHardware {
    fn read_adc10(&mut self, channel_1_based: u8) -> i16;
    fn read_adc24(&mut self) -> i32;
    fn read_adc24_fast_integrated(&mut self) -> i32;
    fn read_adc24_slow_integrated(&mut self) -> i32;
    fn adc24_overload_negative(&self) -> bool;
    fn adc24_overload_positive(&self) -> bool;
    fn clear_adc10_ready(&mut self);
    fn adc10_ready(&mut self) -> bool;
    fn clear_adc24_ready(&mut self);
    fn adc24_ready(&mut self) -> bool;
    fn set_range_config(&mut self, config: RangeRelayConfig);
    fn set_trigger_edge(&mut self, positive_edge: bool);
    fn poll_serial_byte(&mut self) -> Option<u8>;
    fn serial_write(&mut self, text: &str);
    fn lcd_write_line(&mut self, row: u8, text: &str);
}
