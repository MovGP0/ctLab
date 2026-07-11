//! Defines DIV the div runtime adapter responsibilities separated from the original monolithic source.

#[allow(unused_imports)]
use super::*;

/// Bridges the standalone DIV parser hooks to live device state so parser tests and the runtime share calibration, trigger, and output behavior.
pub struct DivRuntimeAdapter<'a, H: DivRuntimeHardware> {
    /// Owns the device object that supplies this type's hardware or parser state.
    pub device: &'a mut DivDeviceState<H>,

    /// Mirrors whether the runtime currently rejects state-changing parser commands.
    pub busy: bool,

    /// Counts the 125-systick activity-LED hold time loaded after a valid command.
    pub activity_timer_ticks: Option<u8>,

    /// Counts active-low LED assertions so host tests can observe parser activity without AVR GPIO.
    pub activity_led_low_count: usize,
}

impl<'a, H: DivRuntimeHardware> DivRuntimeAdapter<'a, H> {
    /// Attaches parser hooks to the live device with busy clear and the activity LED timer stopped.
    pub fn new(device: &'a mut DivDeviceState<H>) -> Self {
        Self {
            device,
            busy: false,
            activity_timer_ticks: None,
            activity_led_low_count: 0,
        }
    }

    /// Writes status prompt to the serial, display, or peripheral destination selected by the implementation.
    pub(super) fn write_status_prompt(&mut self, state: &mut ParserState, err: ParserError) {
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
            if (fault_flags & DivFault::NegativeOverload.mask()) != 0 {
                self.device.hw.serial_write(" ");
                self.device
                    .hw
                    .serial_write(DivFault::NegativeOverload.as_str());
            }
            if (fault_flags & DivFault::PositiveOverload.mask()) != 0 {
                self.device.hw.serial_write(" ");
                self.device
                    .hw
                    .serial_write(DivFault::PositiveOverload.as_str());
            }
        } else {
            self.device.hw.serial_write(" ");
            self.device.hw.serial_write(err.as_str());
        }
        self.ser_crlf();
    }

    /// Writes formatted parameter to the serial, display, or peripheral destination selected by the implementation.
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

    /// Derives fault flags from the current flags for protocol and protection decisions.
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
    /// Reports whether busy without mutating device state.
    fn is_busy(&self) -> bool {
        self.busy
    }

    /// Returns the latest LTC2400 sample so callers use the intended display or trigger integration mode.
    fn get_ad24(&mut self, sub_ch: u8, state: &mut ParserState) {
        let raw = self.device.hw.read_adc24()
            + self.device.eeprom.ad24_offsets[self.device.range as usize];
        state.ad24temp = match sub_ch {
            1 => raw,
            2 => raw,
            _ => raw,
        };
    }

    /// Waits for ad24 so callers cannot consume a stale hardware result.
    fn wait_ad24(&mut self, _state: &mut ParserState) {
        self.device.wait_ad24();
    }

    /// Waits for ad10 so callers cannot consume a stale hardware result.
    fn wait_ad10(&mut self, _state: &mut ParserState) {
        self.device.wait_ad10();
    }

    /// Obtains ad10 from the owning state or hardware register for the caller that consumes it.
    fn get_ad10(&mut self, channel: u8, state: &mut ParserState) {
        let raw = i32::from(self.device.hw.read_adc10(channel))
            + i32::from(self.device.eeprom.ad10_offsets[self.device.range as usize]);
        state.param_long_int = raw;
    }

    /// Returns one raw conversion from one-based AVR ADC channel 1..8.
    fn get_adc(&mut self, channel: u8) -> i32 {
        i32::from(self.device.hw.read_adc10(channel))
    }

    /// Applies the active range offset and scale while producing parameter scale24.
    fn param_scale24(&mut self, state: &mut ParserState) {
        state.param = self.device.param_scale_24(state.ad24temp);
    }

    /// Applies the active range offset and scale while producing parameter scale10.
    fn param_scale10(&mut self, state: &mut ParserState) {
        let raw = state.param_long_int as i16;
        state.param = self.device.param_scale_10(raw);
    }

    /// Reports whether ac range without mutating device state.
    fn is_ac_range(&self, _state: &ParserState) -> bool {
        self.device.is_ac_range()
    }

    /// Returns range from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy.
    fn get_range(&self) -> u8 {
        self.device.range as u8
    }

    /// Returns offset24 from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy.
    fn get_offset24(&self, index: usize) -> i32 {
        self.device.eeprom.ad24_offsets[index]
    }

    /// Writes the signed LTC2400 count correction for the indexed range into EEPROM state.
    fn set_offset24(&mut self, index: usize, value: i32) {
        self.device.eeprom.ad24_offsets[index] = value;
    }

    /// Returns offset10 from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy.
    fn get_offset10(&self, index: usize) -> i32 {
        i32::from(self.device.eeprom.ad10_offsets[index])
    }

    /// Writes the signed ADC10 count correction for the indexed range into EEPROM state.
    fn set_offset10(&mut self, index: usize, value: i32) {
        self.device.eeprom.ad10_offsets[index] = value as i16;
    }

    /// Returns scale24 from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy.
    fn get_scale24(&self, index: usize) -> f32 {
        self.device.eeprom.ad24_scales[index]
    }

    /// Writes the LTC2400 calibration multiplier for the indexed range into EEPROM state.
    fn set_scale24(&mut self, index: usize, value: f32) {
        self.device.eeprom.ad24_scales[index] = value;
    }

    /// Returns scale10 from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy.
    fn get_scale10(&self, index: usize) -> f32 {
        self.device.eeprom.ad10_scales[index]
    }

    /// Writes the ADC10 calibration multiplier for the indexed range into EEPROM state.
    fn set_scale10(&mut self, index: usize, value: f32) {
        self.device.eeprom.ad10_scales[index] = value;
    }

    /// Returns trigger mask from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy.
    fn get_trigger_mask(&self) -> u8 {
        self.device.eeprom.trigger_mode
    }

    /// Persists `TRM` bits 0..2 for AD24, ADC10 RMS, and ADC10 peak trigger output.
    fn set_trigger_mask(&mut self, value: u8) {
        self.device.eeprom.trigger_mode = value;
    }

    /// Returns trigger timer value from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy.
    fn get_trigger_timer_value(&self) -> u16 {
        self.device.eeprom.trigger_timer_ms
    }

    /// Persists the `TRT` automatic-trigger interval in milliseconds; zero disables it.
    fn set_trigger_timer_value(&mut self, value: u16) {
        self.device.eeprom.trigger_timer_ms = value;
    }

    /// Latches trigger now for deferred processing outside the interrupt-sensitive edge handler.
    fn trigger_now(&mut self) {
        self.device.trigger_pending = true;
    }

    /// Validates limits before dependent hardware state is changed.
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

    /// Applies range as one coherent state and hardware transition.
    fn switch_range(&mut self, state: &mut ParserState) {
        let range = div_range_from_u8(state.range);
        self.device.switch_range(range);
        state.range = self.device.range as u8;
    }

    /// Emits show range using the exact channel and status framing expected by existing clients.
    fn show_range(&mut self, _state: &mut ParserState) {
        self.device.show_range();
    }

    /// Stores the parser's working frame, command, calibration, and response values.
    fn write_param_long_int_ser(&mut self, state: &ParserState) {
        self.write_ch_prefix(state);
        self.device
            .hw
            .serial_write(&state.param_long_int.to_string());
        self.ser_crlf();
    }

    /// Stores the parser's working frame, command, calibration, and response values.
    fn write_param_ser(&mut self, state: &ParserState, overload: bool) {
        self.write_formatted_param(state, overload);
    }

    /// Writes the addressed channel prefix before a payload so every response keeps the Pascal wire framing.
    fn write_ch_prefix(&mut self, state: &ParserState) {
        self.device
            .hw
            .serial_write(&format!("#{}:{}=", state.slave_ch, state.sub_ch));
    }

    /// Writes serial inp to the serial, display, or peripheral destination selected by the implementation.
    fn write_ser_inp(&mut self, input: &str) {
        self.device.hw.serial_write(input);
        self.ser_crlf();
    }

    /// Writes string to the serial, display, or peripheral destination selected by the implementation.
    fn write_str(&mut self, text: &str) {
        self.device.hw.serial_write(text);
    }

    /// Terminates the current serial response with CRLF because existing clients parse line-delimited frames.
    fn ser_crlf(&mut self) {
        self.device.ser_crlf();
    }

    /// Emits serprompt using the exact channel and status framing expected by existing clients.
    fn serprompt(&mut self, state: &mut ParserState, err: ParserError) {
        self.write_status_prompt(state, err);
    }

    /// Arms the activity-indicator countdown for the requested number of systicks.
    fn set_activity_timer(&mut self, ticks: u8) {
        self.activity_timer_ticks = Some(ticks);
    }

    /// Records one assertion of the active-low activity LED for the runtime hardware boundary.
    fn set_activity_led_low(&mut self) {
        self.activity_led_low_count += 1;
    }
}
