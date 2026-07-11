#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone)]
pub struct ParserState {
    pub sub_ch: u8,
    pub current_ch: u8,
    pub cmd_which: CmdWhich,
    pub param: f32,
    pub param_long_int: i32,
    pub param_str: String,
    pub ser_inp_str: String,
    pub ser_inp_ptr: usize,
    pub slave_ch: u8,
    pub range: u8,
    pub ad24temp: i32,
    pub lcd_integrate: u8,
    pub init_lcd_integrate: u8,
    pub inc_rast: i32,
    pub init_inc_rast: i32,
    pub errcount: i32,
    pub ee_unlocked: bool,
    pub verbose: bool,
    pub overload_flag: bool,
    pub check_limit_err: ParserError,
    pub trig_mask: u8,
    pub trig_timer_value: u16,
    pub trigger: bool,
    pub offset_array24: [i32; 16],
    pub offset_array10: [i32; 16],
    pub scale_array24: [f32; 16],
    pub scale_array10: [f32; 16],
}

impl Default for ParserState {
    fn default() -> Self {
        Self {
            sub_ch: 0,
            current_ch: 255,
            cmd_which: CmdWhich::Val,
            param: 0.0,
            param_long_int: 0,
            param_str: String::new(),
            ser_inp_str: String::new(),
            ser_inp_ptr: 0,
            slave_ch: 0,
            range: 0,
            ad24temp: 0,
            lcd_integrate: 1,
            init_lcd_integrate: 1,
            inc_rast: 4,
            init_inc_rast: 4,
            errcount: 0,
            ee_unlocked: false,
            verbose: false,
            overload_flag: false,
            check_limit_err: ParserError::NoErr,
            trig_mask: 0,
            trig_timer_value: 0,
            trigger: false,
            offset_array24: [0; 16],
            offset_array10: [0; 16],
            scale_array24: [1.0; 16],
            scale_array10: [1.0; 16],
        }
    }
}
