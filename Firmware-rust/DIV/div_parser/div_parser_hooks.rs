#[allow(unused_imports)]
use super::*;

pub trait DivParserHooks {
    fn is_busy(&self) -> bool;
    fn get_ad24(&mut self, sub_ch: u8, state: &mut ParserState);
    fn wait_ad24(&mut self, state: &mut ParserState);
    fn wait_ad10(&mut self, state: &mut ParserState);
    fn get_ad10(&mut self, channel: u8, state: &mut ParserState);
    fn get_adc(&mut self, channel: u8) -> i32;
    fn param_scale24(&mut self, state: &mut ParserState);
    fn param_scale10(&mut self, state: &mut ParserState);
    fn is_ac_range(&self, state: &ParserState) -> bool;
    fn get_range(&self) -> u8;
    fn get_offset24(&self, index: usize) -> i32;
    fn set_offset24(&mut self, index: usize, value: i32);
    fn get_offset10(&self, index: usize) -> i32;
    fn set_offset10(&mut self, index: usize, value: i32);
    fn get_scale24(&self, index: usize) -> f32;
    fn set_scale24(&mut self, index: usize, value: f32);
    fn get_scale10(&self, index: usize) -> f32;
    fn set_scale10(&mut self, index: usize, value: f32);
    fn get_trigger_mask(&self) -> u8;
    fn set_trigger_mask(&mut self, value: u8);
    fn get_trigger_timer_value(&self) -> u16;
    fn set_trigger_timer_value(&mut self, value: u16);
    fn trigger_now(&mut self);
    fn check_limits(&mut self, state: &mut ParserState);
    fn switch_range(&mut self, state: &mut ParserState);
    fn show_range(&mut self, state: &mut ParserState);

    fn write_param_long_int_ser(&mut self, state: &ParserState);
    fn write_param_ser(&mut self, state: &ParserState, overload: bool);
    fn write_ch_prefix(&mut self, state: &ParserState);
    fn write_ser_inp(&mut self, input: &str);
    fn write_str(&mut self, text: &str);
    fn ser_crlf(&mut self);
    fn serprompt(&mut self, state: &mut ParserState, err: ParserError);

    fn set_activity_timer(&mut self, ticks: u8);
    fn set_activity_led_low(&mut self);
}
