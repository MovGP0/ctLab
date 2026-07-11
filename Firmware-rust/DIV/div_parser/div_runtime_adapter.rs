#[allow(unused_imports)]
use super::*;

pub struct DivRuntimeAdapter<'a, H: DivRuntimeHardware> {
    pub device: &'a mut DivDeviceState<H>,
    pub busy: bool,
    pub activity_timer_ticks: Option<u8>,
    pub activity_led_low_count: usize,
}

impl<'a, H: DivRuntimeHardware> DivRuntimeAdapter<'a, H> {
    pub fn new(device: &'a mut DivDeviceState<H>) -> Self {
        Self {
            device,
            busy: false,
            activity_timer_ticks: None,
            activity_led_low_count: 0,
        }
    }

    pub(super) fn write_status_prompt(&mut self, state: &mut ParserState, err: ParserError) {
        const ERR_LABELS: [&str; 8] = [
            "[OK]", "[SRQUSR]", "[BUSY]", "[OVRLD]", "[CMDERR]", "[PARERR]", "[LOCKED]", "[CHKSUM]",
        ];
        const FAULT_LABELS: [&str; 4] = ["[OVRNEG]", "[OVRPOS]", "[]", "[]"];

        let original_sub_ch = state.sub_ch;
        state.sub_ch = 255;
        self.write_ch_prefix(state);
        state.sub_ch = original_sub_ch;

        let fault_flags = self.fault_flags();
        let mut status = 0u8;
        if self.busy {
            status |= 0x80;
        }
        if err == ParserError::UserReq {
            status |= 0x40;
        }
        if fault_flags != 0 || state.overload_flag {
            status |= 0x20;
        }
        if state.ee_unlocked {
            status |= 0x10;
        }

        if err == ParserError::OvlErr {
            status |= fault_flags;
        } else {
            status |= err as u8;
            if err != ParserError::NoErr && err != ParserError::UserReq {
                state.errcount += 1;
            }
        }

        self.device.hw.serial_write(&status.to_string());
        if fault_flags != 0 {
            for (bit, label) in FAULT_LABELS.iter().enumerate() {
                if (fault_flags & (1 << bit)) != 0 {
                    self.device.hw.serial_write(" ");
                    self.device.hw.serial_write(label);
                }
            }
        } else {
            self.device.hw.serial_write(" ");
            self.device
                .hw
                .serial_write(ERR_LABELS[(err as usize).min(ERR_LABELS.len() - 1)]);
        }
        self.ser_crlf();
    }

    pub(super) fn write_formatted_param(&mut self, state: &ParserState, overload: bool) {
        self.write_ch_prefix(state);
        if overload && state.sub_ch < 20 {
            self.device.hw.serial_write("-9999 [OVERLD]");
            self.ser_crlf();
            return;
        }

        self.device
            .hw
            .serial_write(&format_serial_param(state.param));
        if state.sub_ch < 20 {
            if let Some(suffix) = range_exponent_suffix(self.device.range) {
                self.device.hw.serial_write(suffix);
            }
        }
        self.ser_crlf();
    }

    pub(super) fn fault_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.device.overload_negative {
            flags |= 0x01;
        }
        if self.device.overload_positive {
            flags |= 0x02;
        }
        flags
    }
}

impl<H: DivRuntimeHardware> DivParserHooks for DivRuntimeAdapter<'_, H> {
    fn is_busy(&self) -> bool {
        self.busy
    }

    fn get_ad24(&mut self, sub_ch: u8, state: &mut ParserState) {
        let raw = self.device.hw.read_adc24()
            + self.device.eeprom.ad24_offsets[self.device.range as usize];
        state.ad24temp = match sub_ch {
            1 => raw,
            2 => raw,
            _ => raw,
        };
    }

    fn wait_ad24(&mut self, _state: &mut ParserState) {
        self.device.wait_ad24();
    }

    fn wait_ad10(&mut self, _state: &mut ParserState) {
        self.device.wait_ad10();
    }

    fn get_ad10(&mut self, channel: u8, state: &mut ParserState) {
        let raw = i32::from(self.device.hw.read_adc10(channel))
            + i32::from(self.device.eeprom.ad10_offsets[self.device.range as usize]);
        state.param_long_int = raw;
    }

    fn get_adc(&mut self, channel: u8) -> i32 {
        i32::from(self.device.hw.read_adc10(channel))
    }

    fn param_scale24(&mut self, state: &mut ParserState) {
        state.param = self.device.param_scale_24(state.ad24temp);
    }

    fn param_scale10(&mut self, state: &mut ParserState) {
        let raw = state.param_long_int as i16;
        state.param = self.device.param_scale_10(raw);
    }

    fn is_ac_range(&self, _state: &ParserState) -> bool {
        self.device.is_ac_range()
    }

    fn get_range(&self) -> u8 {
        self.device.range as u8
    }

    fn get_offset24(&self, index: usize) -> i32 {
        self.device.eeprom.ad24_offsets[index]
    }

    fn set_offset24(&mut self, index: usize, value: i32) {
        self.device.eeprom.ad24_offsets[index] = value;
    }

    fn get_offset10(&self, index: usize) -> i32 {
        i32::from(self.device.eeprom.ad10_offsets[index])
    }

    fn set_offset10(&mut self, index: usize, value: i32) {
        self.device.eeprom.ad10_offsets[index] = value as i16;
    }

    fn get_scale24(&self, index: usize) -> f32 {
        self.device.eeprom.ad24_scales[index]
    }

    fn set_scale24(&mut self, index: usize, value: f32) {
        self.device.eeprom.ad24_scales[index] = value;
    }

    fn get_scale10(&self, index: usize) -> f32 {
        self.device.eeprom.ad10_scales[index]
    }

    fn set_scale10(&mut self, index: usize, value: f32) {
        self.device.eeprom.ad10_scales[index] = value;
    }

    fn get_trigger_mask(&self) -> u8 {
        self.device.eeprom.trigger_mode
    }

    fn set_trigger_mask(&mut self, value: u8) {
        self.device.eeprom.trigger_mode = value;
    }

    fn get_trigger_timer_value(&self) -> u16 {
        self.device.eeprom.trigger_timer_ms
    }

    fn set_trigger_timer_value(&mut self, value: u16) {
        self.device.eeprom.trigger_timer_ms = value;
    }

    fn trigger_now(&mut self) {
        self.device.trigger_pending = true;
    }

    fn check_limits(&mut self, state: &mut ParserState) {
        state.check_limit_err = ParserError::NoErr;
        if state.range > 127 {
            state.range = 0;
            state.check_limit_err = ParserError::ParamErr;
        }
        if state.range > 15 {
            state.range = 15;
            state.check_limit_err = ParserError::ParamErr;
        }
    }

    fn switch_range(&mut self, state: &mut ParserState) {
        let range = div_range_from_u8(state.range);
        self.device.switch_range(range);
        state.range = self.device.range as u8;
    }

    fn show_range(&mut self, _state: &mut ParserState) {
        self.device.show_range();
    }

    fn write_param_long_int_ser(&mut self, state: &ParserState) {
        self.write_ch_prefix(state);
        self.device
            .hw
            .serial_write(&state.param_long_int.to_string());
        self.ser_crlf();
    }

    fn write_param_ser(&mut self, state: &ParserState, overload: bool) {
        self.write_formatted_param(state, overload);
    }

    fn write_ch_prefix(&mut self, state: &ParserState) {
        self.device
            .hw
            .serial_write(&format!("#{}:{}=", state.slave_ch, state.sub_ch));
    }

    fn write_ser_inp(&mut self, input: &str) {
        self.device.hw.serial_write(input);
        self.ser_crlf();
    }

    fn write_str(&mut self, text: &str) {
        self.device.hw.serial_write(text);
    }

    fn ser_crlf(&mut self) {
        self.device.ser_crlf();
    }

    fn serprompt(&mut self, state: &mut ParserState, err: ParserError) {
        self.write_status_prompt(state, err);
    }

    fn set_activity_timer(&mut self, ticks: u8) {
        self.activity_timer_ticks = Some(ticks);
    }

    fn set_activity_led_low(&mut self) {
        self.activity_led_low_count += 1;
    }
}
