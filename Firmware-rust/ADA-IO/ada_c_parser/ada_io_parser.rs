//! Defines ADA command parsing and dispatch for the instrument serial protocol.

#[allow(unused_imports)]
use super::*;

/// Owns ada io parser state while decoding one serial command at a time.
pub struct AdaIoParser {
    /// Owns the ctx object that supplies this type's hardware or parser state.
    pub ctx: ParseContext,
}

impl Default for AdaIoParser {
    /// Creates a parser with Pascal calibration defaults and no pending serial frame.
    fn default() -> Self {
        Self {
            ctx: ParseContext::default(),
        }
    }
}

impl AdaIoParser {
    /// Maps command to index onto the command enum so dispatch uses a bounded match instead of string comparisons.
    pub fn cmd_to_index(&mut self) -> CmdWhich {
        CmdWhich::from_str(&self.ctx.param_str)
    }

    /// Parses extract and updates only the state owned by that protocol phase.
    pub fn parse_extract(&mut self) -> bool {
        self.ctx.param_str.clear();

        // Skip leading spaces before classifying the next token.
        while matches!(self.peek_char(), Some(' ')) {
            self.ctx.ser_inp_ptr += 1;
        }

        let first = match self.peek_char() {
            Some(ch) => ch,
            None => return false,
        };

        let mut is_param = false;
        if matches!(first, '*' | '0'..='9') {
            // Digits and `*` start a parameter token rather than a mnemonic.
            is_param = true;
            while let Some(ch) = self.peek_char() {
                if ch <= '9' {
                    self.ctx.param_str.push(ch);
                    self.ctx.ser_inp_ptr += 1;
                } else {
                    // Stop at the first separator or non-numeric suffix.
                    break;
                }
            }
        } else {
            while let Some(ch) = self.peek_char() {
                if ch >= 'A' {
                    self.ctx.param_str.push(ch);
                    self.ctx.ser_inp_ptr += 1;
                } else {
                    // Command names end once the stream falls back to digits or punctuation.
                    break;
                }
            }
        }

        is_param
    }

    /// Parses get parameter and updates only the state owned by that protocol phase.
    pub fn parse_get_param(&mut self) -> Result<Vec<Reply>, ParseError> {
        let mut replies = Vec::new();
        let mut is_integer = false;

        match self.ctx.sub_ch {
            0..=47 => {
                // Direct request for live AD10/AD16 input values.
                is_integer = self.get_new_value(self.ctx.sub_ch);
            }
            50..=67 => {
                // RAW aliases expose the same channels as integer ADC readings.
                self.get_new_value(self.ctx.sub_ch - 50);
                is_integer = true;
            }
            70..=77 => {
                self.ctx.param_int =
                    i32::from(self.ctx.dac_raw_array[(self.ctx.sub_ch - 70) as usize]);
                is_integer = true;
            }
            80 => {
                self.ctx.param_int = i32::from(self.ctx.modify);
                is_integer = true;
            }
            85 => {
                replies.push(self.write_ch_prefix_text(&self.ctx.egg_str));
                return Ok(replies);
            }
            89 | 159 => {
                self.ctx.param_int = self.ctx.inc_rast;
                is_integer = true;
            }
            95 => {
                // Dump all AD10 input channels.
                for sub_ch in 0..=7 {
                    self.ctx.sub_ch = sub_ch;
                    self.get_new_value(sub_ch);
                    replies.push(self.write_param());
                }
                return Ok(replies);
            }
            96 => {
                // Dump all AD16 input channels.
                for sub_ch in 10..=17 {
                    self.ctx.sub_ch = sub_ch;
                    self.get_new_value(sub_ch);
                    replies.push(self.write_param());
                }
                return Ok(replies);
            }
            98 => {
                // Dump all digital input levels.
                for sub_ch in 30..=37 {
                    self.ctx.sub_ch = sub_ch;
                    self.get_new_value(sub_ch);
                    replies.push(self.write_param_int());
                }
                return Ok(replies);
            }
            99 => {
                // Combined dump of AD10, AD16, and port levels in Pascal order.
                for sub_ch in 0..=7 {
                    self.ctx.sub_ch = sub_ch;
                    self.get_new_value(sub_ch);
                    replies.push(self.write_param());
                }
                for sub_ch in 10..=17 {
                    self.ctx.sub_ch = sub_ch;
                    self.get_new_value(sub_ch);
                    replies.push(self.write_param());
                }
                for sub_ch in 30..=37 {
                    self.ctx.sub_ch = sub_ch;
                    self.get_new_value(sub_ch);
                    replies.push(self.write_param_int());
                }
                return Ok(replies);
            }
            100..=127 => {
                self.ctx.param_int = self.ctx.offset_array[(self.ctx.sub_ch - 100) as usize];
                is_integer = true;
            }
            156 | 246 => {
                // REF exposes whether the external reference is selected.
                self.ctx.param_int = i32::from(self.ctx.ext_ref);
                is_integer = true;
            }
            157 => {
                // Optional AD16 integration mode is reported as 0/1.
                self.ctx.param_int = i32::from(self.ctx.integrate_ad16);
                is_integer = true;
            }
            180..=187 => {
                self.ctx.param_int =
                    i32::from(self.ctx.port_init_array[(self.ctx.sub_ch - 180) as usize]);
                is_integer = true;
            }
            190..=197 => {
                self.ctx.param_int =
                    i32::from(self.ctx.dir_init_array[(self.ctx.sub_ch - 190) as usize]);
                is_integer = true;
            }
            200..=229 => {
                self.ctx.param = self.ctx.scale_array[(self.ctx.sub_ch - 200) as usize];
            }
            230 => {
                // Generic I2C byte read.
                let byte = self.twi_inp_byte(self.ctx.i2c_slave_adr);
                self.ctx.param_int = i32::from(byte);
                is_integer = true;
            }
            231 => {
                // Generic I2C word read.
                self.ctx.param_int = i32::from(self.twi_inp_word(self.ctx.i2c_slave_adr));
                is_integer = true;
            }
            232 => {
                // Generic I2C word read with byte order swapped afterward.
                let value = self.twi_inp_word(self.ctx.i2c_slave_adr);
                self.ctx.param_int = i32::from(value.swap_bytes());
                is_integer = true;
            }
            233 => {
                // LM75-compatible scaling: swap, keep 9 bits, then divide by 2.
                let value = self.twi_inp_word(self.ctx.i2c_slave_adr).swap_bytes() >> 7;
                self.ctx.param_int = i32::from(value);
                self.ctx.param = value as f32 / 2.0;
            }
            234 => {
                // DS1631-compatible scaling: swapped 16-bit fixed-point / 256.
                let value = self.twi_inp_word(self.ctx.i2c_slave_adr).swap_bytes();
                self.ctx.param_int = i32::from(value);
                self.ctx.param = value as f32 / 256.0;
            }
            239 => {
                self.ctx.param_int = i32::from(self.ctx.i2c_slave_adr);
                is_integer = true;
            }
            240..=243 => {
                self.ctx.param_int =
                    i32::from(self.ctx.trig_mask_array[(self.ctx.sub_ch - 240) as usize]);
                is_integer = true;
            }
            247 => {
                self.ctx.param_int = i32::from(self.ctx.trig_timer_value);
                is_integer = true;
            }
            248 => {
                self.ctx.param_int = i32::from(self.ctx.trig_level);
                is_integer = true;
            }
            249 => {
                // Manual trigger executes immediately and replies with status.
                self.ctx.trigger = true;
                replies.push(self.status_reply(ParseError::NoErr, self.ctx.status));
                return Ok(replies);
            }
            250 | 255 => {
                replies.push(self.status_reply(ParseError::NoErr, self.ctx.status));
                return Ok(replies);
            }
            251 => {
                self.ctx.param_int = self.ctx.err_count;
                is_integer = true;
            }
            252 => {
                self.ctx.param_int = i32::from(self.ctx.ee_ser_baud_reg);
                is_integer = true;
            }
            253 => {
                replies.push(Reply::Text(self.ctx.ser_inp_str.clone()));
                return Ok(replies);
            }
            254 => {
                let mut text = self.write_ch_prefix();
                text.push_str(&self.ctx.vers1_str);
                text.push_str(&self.write_features());
                replies.push(Reply::Text(text));
                return Ok(replies);
            }
            _ => return Err(ParseError::ParamErr),
        }

        replies.push(if is_integer {
            self.write_param_int()
        } else {
            self.write_param()
        });
        Ok(replies)
    }

    /// Parses set parameter and updates only the state owned by that protocol phase.
    pub fn parse_set_param(&mut self) -> Result<Vec<Reply>, ParseError> {
        self.ctx.changed_flag = true;

        match self.ctx.sub_ch {
            20..=27 => {
                // Set DAC engineering value and refresh the corresponding output.
                self.ctx.dac_value_array[(self.ctx.sub_ch - 20) as usize] = self.ctx.param;
                self.set_dac(self.ctx.sub_ch);
            }
            30..=37 => {
                // Drive the selected PIO output pin now.
                self.set_port(self.ctx.sub_ch - 30, self.ctx.param_byte);
            }
            40..=47 => {
                // Update direction only; this path deliberately does not touch
                // the EEPROM-backed init array.
                self.set_dir(self.ctx.sub_ch - 40, self.ctx.param_byte);
            }
            80 => {
                if self.ctx.param_byte > 37 {
                    return Err(ParseError::ParamErr);
                }
                self.ctx.modify = self.ctx.param_byte;
            }
            81 => {
                if self.ctx.param_byte > 37 {
                    return Err(ParseError::ParamErr);
                }
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }

                // LCD text is carried inside the first bracketed segment.
                let text = self.extract_bracket_text();
                self.ctx.param_text_array[self.ctx.param_byte as usize] = text;
            }
            89 | 159 => {
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                self.ctx.inc_rast = self.ctx.param_int;
                self.ctx.inc_rast_def = self.ctx.inc_rast;
            }
            100..=127 => {
                let index = (self.ctx.sub_ch - 100) as usize;
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                self.ctx.offset_array[index] = self.ctx.param_int;
                self.set_dac(index as u8);
            }
            156 | 246 => {
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                // `0` selects the external ADC reference; any non-zero value
                // switches to the internal reference path.
                self.ctx.ext_ref = self.ctx.param_byte;
                self.set_reference_mode(self.ctx.param_byte != 0);
            }
            157 => {
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                self.ctx.integrate_ad16 = self.ctx.param_byte > 0;
                self.ctx.init_integrate_ad16 = self.ctx.integrate_ad16;
            }
            180..=187 => {
                let index = (self.ctx.sub_ch - 180) as usize;
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                self.ctx.port_init_array[index] = self.ctx.param_byte;
                self.set_port(index as u8, self.ctx.param_byte);
            }
            190..=197 => {
                let index = (self.ctx.sub_ch - 190) as usize;
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                self.ctx.dir_init_array[index] = self.ctx.param_byte;
                self.set_dir(index as u8, self.ctx.param_byte);
            }
            200..=229 => {
                let index = (self.ctx.sub_ch - 200) as usize;
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                self.ctx.scale_array[index] = self.ctx.param;
                self.set_dac(index as u8);
                self.set_base_scales();
            }
            230 => self.twi_out_byte(self.ctx.i2c_slave_adr, self.ctx.param_byte),
            231 => self.twi_out_word(self.ctx.i2c_slave_adr, self.ctx.param_int as u16),
            232 => {
                let swapped = (self.ctx.param_int as u16).swap_bytes();
                self.twi_out_word(self.ctx.i2c_slave_adr, swapped);
            }
            239 => {
                // Store the generic I2C target address for later I/O commands.
                self.ctx.i2c_slave_adr = self.ctx.param_byte;
            }
            240..=243 => {
                let index = (self.ctx.sub_ch - 240) as usize;
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                self.ctx.trig_mask_array[index] = self.ctx.param_byte;
            }
            247 => {
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                // The original firmware rejected auto-trigger values from 1 to 9 ms.
                if (1..=9).contains(&self.ctx.param_int) {
                    return Err(ParseError::ParamErr);
                }
                self.ctx.trig_timer_value = self.ctx.param_int as u16;
            }
            248 => {
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                // `0` means negative edge, non-zero means positive edge.
                self.ctx.trig_level = self.ctx.param_byte;
                self.set_trigger_edge(self.ctx.param_byte != 0);
            }
            249 => {
                // Immediate manual trigger path.
                self.ctx.trigger = true;
                return Ok(vec![self.status_reply(ParseError::NoErr, self.ctx.status)]);
            }
            250 => {}
            251 => {
                self.ctx.err_count = self.ctx.param_int;
            }
            252 => {
                if !self.ctx.ee_unlocked {
                    return Err(ParseError::LockedErr);
                }
                self.ctx.ee_ser_baud_reg = self.ctx.param_byte;
            }
            _ => return Err(ParseError::ParamErr),
        }

        // WEN is a one-shot EEPROM write enable; all other commands clear it again.
        self.ctx.ee_unlocked = self.ctx.sub_ch == 250 && self.ctx.param_byte == 1;
        Ok(vec![self.status_reply(ParseError::NoErr, self.ctx.status)])
    }

    /// Parses sub channel and updates only the state owned by that protocol phase.
    pub fn parse_sub_ch(&mut self) -> Result<Vec<Reply>, ParseError> {
        if self.ctx.ser_inp_str.is_empty() {
            return Ok(vec![self.status_reply(ParseError::NoErr, 0)]);
        }

        let has_main_ch = self.ctx.ser_inp_str.contains(':');
        let is_request = !self.ctx.ser_inp_str.contains('=');
        let first_char = self.ctx.ser_inp_str.chars().next().unwrap_or_default();
        let is_omni = first_char == '*';
        let is_result = first_char == '#';

        if is_result {
            // Incoming result frames are relayed instead of parsed locally.
            return Ok(vec![Reply::Echo(self.ctx.ser_inp_str.clone())]);
        }

        self.ctx.ser_inp_ptr = 0;
        let mut replies = Vec::new();

        if has_main_ch {
            let is_param = self.parse_extract();
            if !is_param {
                return Err(ParseError::SyntaxErr);
            }
            self.skip_char(':');
            if is_omni {
                // Omni commands are forwarded so every slave can act on them.
                replies.push(Reply::Echo(self.ctx.ser_inp_str.clone()));
            } else {
                self.ctx.current_ch = self.parse_u8(&self.ctx.param_str)?;
            }
        }

        if !is_omni && has_main_ch && self.ctx.current_ch != self.ctx.slave_ch {
            // A command for another slave is only passed through.
            return Ok(vec![Reply::Echo(self.ctx.ser_inp_str.clone())]);
        }

        // `!` or `?` request the verbose reply form from the original parser.
        self.ctx.verbose = self.ctx.ser_inp_str.contains('!') || self.ctx.ser_inp_str.contains('?');

        if let Some(check_pos) = self.ctx.ser_inp_str.find('$') {
            // Optional XOR checksum covers the line up to, but not including, `$xx`.
            let supplied = self
                .ctx
                .ser_inp_str
                .get(check_pos + 1..check_pos + 3)
                .ok_or(ParseError::ChecksumErr)?;
            let check_sum_in =
                u8::from_str_radix(supplied, 16).map_err(|_| ParseError::ChecksumErr)?;
            let mut check_sum = 0u8;
            for ch in self.ctx.ser_inp_str[..check_pos].bytes() {
                check_sum ^= ch;
            }
            if check_sum != check_sum_in {
                return Err(ParseError::ChecksumErr);
            }
        }

        self.set_sys_timer_activity();
        self.ctx.led_activity_low = true;

        let sub_ch_offset = if self.parse_extract() {
            // Short numeric form omits `VAL` and reuses the current channel.
            self.ctx.cmd_which = CmdWhich::Val;
            0
        } else {
            // Text commands are translated to their base sub-channel ranges.
            self.ctx.cmd_which = self.cmd_to_index();
            if self.ctx.cmd_which == CmdWhich::Err {
                return Err(ParseError::SyntaxErr);
            }
            let offset = self
                .ctx
                .cmd_which
                .sub_channel_offset()
                .ok_or(ParseError::SyntaxErr)?;
            // Command form expects a following sub-channel token.
            self.parse_extract();
            offset
        };

        // The parser stores sub-channels as absolute offsets into the dispatch tables.
        let sub_ch_value = if self.ctx.param_str.trim().is_empty() {
            0
        } else {
            self.parse_u8_or_wildcard(&self.ctx.param_str)?
        };
        self.ctx.sub_ch = sub_ch_value.saturating_add(sub_ch_offset);

        if is_request {
            replies.extend(self.parse_get_param()?);
            Ok(replies)
        } else {
            if let Some(equal_pos) = self.ctx.ser_inp_str.find('=') {
                self.ctx.ser_inp_ptr = equal_pos + 1;
            }

            if self.parse_extract() {
                // Cache the same payload as float, integer, and byte, matching
                // the Pascal parser's shared parameter variables.
                self.ctx.param = self.parse_f32(&self.ctx.param_str)?;
                self.ctx.param_int = self.ctx.param as i32;
                self.ctx.param_byte = self.ctx.param_int as u8;
            } else if self.ctx.cmd_which.requires_parameter_on_set() {
                return Err(ParseError::ParamErr);
            }

            replies.extend(self.parse_set_param()?);
            Ok(replies)
        }
    }

    /// Advances the parser with peek char while keeping the byte cursor within the received frame.
    pub(super) fn peek_char(&self) -> Option<char> {
        self.ctx.ser_inp_str[self.ctx.ser_inp_ptr..].chars().next()
    }

    /// Advances the parser with skip char while keeping the byte cursor within the received frame.
    pub(super) fn skip_char(&mut self, expected: char) {
        if self.peek_char() == Some(expected) {
            self.ctx.ser_inp_ptr += expected.len_utf8();
        }
    }

    /// Parses u8 and updates only the state owned by that protocol phase.
    pub(super) fn parse_u8(&self, value: &str) -> Result<u8, ParseError> {
        value.trim().parse::<u8>().map_err(|_| ParseError::ParamErr)
    }

    /// Parses u8 or wildcard and updates only the state owned by that protocol phase.
    pub(super) fn parse_u8_or_wildcard(&self, value: &str) -> Result<u8, ParseError> {
        if value.trim() == "*" {
            Ok(self.ctx.current_ch)
        } else {
            self.parse_u8(value)
        }
    }

    /// Parses f32 and updates only the state owned by that protocol phase.
    pub(super) fn parse_f32(&self, value: &str) -> Result<f32, ParseError> {
        value
            .trim()
            .parse::<f32>()
            .map_err(|_| ParseError::ParamErr)
    }

    /// Consumes bracket text once so it is not emitted or processed twice.
    pub(super) fn extract_bracket_text(&self) -> String {
        let start = self.ctx.ser_inp_str.find('[').map(|idx| idx + 1);
        let end = self.ctx.ser_inp_str[self.ctx.ser_inp_ptr..]
            .find(']')
            .map(|idx| idx + self.ctx.ser_inp_ptr);

        match (start, end) {
            (Some(start), Some(end)) if end >= start => {
                self.ctx.ser_inp_str[start..end].to_string()
            }
            _ => String::new(),
        }
    }

    /// Packages the active subchannel and calibrated floating value as a transport-neutral parser reply.
    pub(super) fn write_param(&self) -> Reply {
        Reply::Float {
            sub_ch: self.ctx.sub_ch,
            value: self.ctx.param,
        }
    }

    /// Packages the active subchannel and raw integer value as a transport-neutral parser reply.
    pub(super) fn write_param_int(&self) -> Reply {
        Reply::Int {
            sub_ch: self.ctx.sub_ch,
            value: self.ctx.param_int,
        }
    }

    /// Writes the addressed channel prefix before a payload so every response keeps the Pascal wire framing.
    pub(super) fn write_ch_prefix(&self) -> String {
        format!("{}:", self.ctx.slave_ch)
    }

    /// Writes channel prefix text to the serial, display, or peripheral destination selected by the implementation.
    pub(super) fn write_ch_prefix_text(&self, text: &str) -> Reply {
        let mut out = self.write_ch_prefix();
        out.push_str(text);
        Reply::Text(out)
    }

    /// Writes features to the serial, display, or peripheral destination selected by the implementation.
    pub(super) fn write_features(&self) -> String {
        let mut features = String::new();

        if self.ctx.dac12_present {
            features.push_str("DA12 ");
        }
        if self.ctx.dac714_present || self.ctx.dac16_present {
            features.push_str("DA16 ");
        }
        if self.ctx.adc16_present {
            features.push_str("AD16 ");
        }
        if self.ctx.io_present {
            features.push_str("IO32 ");
        }
        if self.ctx.lcd_present {
            features.push_str("LCD ");
        }

        features.push(']');
        features
    }

    /// Derives status reply from the current flags for protocol and protection decisions.
    pub(super) fn status_reply(&self, error: ParseError, status: u8) -> Reply {
        Reply::Status { error, status }
    }

    /// Extracts the setter value following '=' and converts it according to the active command's parameter type.
    pub(super) fn get_new_value(&mut self, sub_ch: u8) -> bool {
        self.ctx.param_int = 0;
        self.ctx.param = 0.0;

        match sub_ch {
            0..=7 => {
                let index = sub_ch as usize;
                self.ctx.param_int = self.ctx.adc10_raw_array[index];
                self.ctx.param = ((self.ctx.param_int + self.ctx.offset_array[index]) as f32
                    * self.ctx.scale_array[index])
                    / self.ctx.base_scale_ad10;
                false
            }
            10..=17 => {
                let index = (sub_ch - 10) as usize;
                let scale_index = sub_ch as usize;
                self.ctx.param_int = self.ctx.adc_raw_array[index];
                self.ctx.param = ((self.ctx.param_int + self.ctx.offset_array[scale_index]) as f32
                    * self.ctx.scale_array[scale_index])
                    / self.ctx.base_scale_ad16;
                false
            }
            20..=27 => {
                self.ctx.param = self.ctx.dac_value_array[(sub_ch - 20) as usize];
                false
            }
            30..=37 => {
                self.ctx.param_int = i32::from(self.get_port(sub_ch - 30));
                true
            }
            40..=47 => {
                self.ctx.param_int = i32::from(self.ctx.dir_init_array[(sub_ch - 40) as usize]);
                true
            }
            _ => false,
        }
    }

    /// Calibrates DAC subchannel 20..27 and encodes it for the detected DAC714, LTC1655, or LTC1257.
    pub(super) fn set_dac(&mut self, sub_ch: u8) {
        if !(20..=27).contains(&sub_ch) {
            return;
        }

        let index = (sub_ch - 20) as usize;
        let offset = self.ctx.offset_array[sub_ch as usize];
        let scale = self.ctx.scale_array[sub_ch as usize];
        let value = self.ctx.dac_value_array[index];

        if self.ctx.dac714_present {
            let raw = (self.ctx.base_scale_da16 * (value * scale)) as i32 + offset;
            self.ctx.dac_raw_array[index] = raw.clamp(-32767, 32767) as i16 as u16;
        }

        if self.ctx.dac16_present {
            let raw = (self.ctx.base_scale_da16 * (value * scale)) as i32 + offset;
            self.ctx.dac_raw_array[index] = (0x8000_i32 - raw.clamp(-32767, 32767)) as u16;
        }

        if self.ctx.dac12_present {
            let raw = (self.ctx.base_scale_da12 * (value * scale)) as i32 + offset;
            self.ctx.dac_raw_array[index] = (0x0800_i32 - raw.clamp(-2047, 2047)) as u16;
        }
    }

    /// Returns port from the selected local port or I2C expander cache.
    pub(super) fn get_port(&mut self, index: u8) -> u8 {
        if self.ctx.io_present {
            self.ctx.io_pin_array[index as usize]
        } else {
            self.ctx.port_array[index as usize]
        }
    }

    /// Updates one live output byte and records the corresponding I2C-expander or complete 4094-chain write.
    pub(super) fn set_port(&mut self, index: u8, value: u8) {
        self.ctx.port_array[index as usize] = value;

        if self.ctx.io_present {
            self.ctx.i2c_slave_adr = index + 0x38;
            self.ctx.param_int = 0x0100 + i32::from(value);
            self.twi_out_word(self.ctx.i2c_slave_adr, self.ctx.param_int as u16);
        } else {
            self.ctx.shift_register_writes.push(self.ctx.port_array);
        }
    }

    /// Records one direction byte for detected I2C-expander hardware; local 4094 outputs have no direction register.
    pub(super) fn set_dir(&mut self, index: u8, value: u8) {
        if self.ctx.io_present {
            self.ctx.dir_output_array[index as usize] = value;
        }
    }

    /// Loads converter full-scale divisors from calibration slots 9, 19, 28, and 29.
    pub(super) fn set_base_scales(&mut self) {
        self.ctx.base_scale_ad10 = self.ctx.scale_array[9];
        self.ctx.base_scale_ad16 = self.ctx.scale_array[19];
        self.ctx.base_scale_da12 = self.ctx.scale_array[28];
        self.ctx.base_scale_da16 = self.ctx.scale_array[29];
    }

    /// Records whether ADC10 commands select the AVR internal 2.56 V reference.
    pub(super) fn set_reference_mode(&mut self, internal_reference: bool) {
        self.ctx.internal_reference = internal_reference;
    }

    /// Records the `TRL` external-trigger polarity: false for falling, true for rising.
    pub(super) fn set_trigger_edge(&mut self, positive_edge: bool) {
        self.ctx.trigger_positive_edge = positive_edge;
    }

    /// Reloads the command-activity indicator to the Pascal hold time of 125 systicks.
    pub(super) fn set_sys_timer_activity(&mut self) {
        self.ctx.activity_timer_ticks = 125;
    }

    /// Encodes TWI inp byte in the compact representation consumed by registers or the serial protocol.
    pub(super) fn twi_inp_byte(&mut self, _slave: u8) -> u8 {
        self.ctx.i2c_byte_reads.pop_front().unwrap_or_default()
    }

    /// Transfers TWI inp word using the byte order expected by the attached peripheral.
    pub(super) fn twi_inp_word(&mut self, _slave: u8) -> u16 {
        self.ctx.i2c_word_reads.pop_front().unwrap_or_default()
    }

    /// Encodes TWI out byte in the compact representation consumed by registers or the serial protocol.
    pub(super) fn twi_out_byte(&mut self, slave: u8, value: u8) {
        self.ctx.i2c_writes.push((slave, u16::from(value)));
    }

    /// Transfers TWI out word using the byte order expected by the attached peripheral.
    pub(super) fn twi_out_word(&mut self, slave: u8, value: u16) {
        self.ctx.i2c_writes.push((slave, value));
    }

    /// Consumes context once so it is not emitted or processed twice.
    pub fn take_context(&mut self) -> ParseContext {
        mem::take(&mut self.ctx)
    }
}
