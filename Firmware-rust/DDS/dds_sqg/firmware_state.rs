use super::*;

#[derive(Debug, Clone)]
pub(super) struct FirmwareState {
    // EEPROM-backed defaults / runtime parameters.
    pub(super) defaults: EepromDefaults,
    pub(super) slave_ch: u8,
    pub(super) current_ch: u8,
    pub(super) sub_ch: u8,
    pub(super) frequenz: i32, // 1/10 Hz, 10000 = 1000.0 Hz
    pub(super) terz_num: u8,
    pub(super) dac_level: f64,
    pub(super) offset_mv: i32,
    pub(super) wave: u8,
    pub(super) burst_mode: u8,
    pub(super) inc_rast: i32,
    pub(super) attn_switch_point: f64,
    pub(super) attn_fac: f64,
    pub(super) pwr_gain: f64,
    pub(super) level_scale_low: f64,
    pub(super) level_scale_hi: f64,

    // Control state.
    pub(super) dss_cmd: u16,
    pub(super) wave_cmd: u16,
    pub(super) dds_frequ: i32,
    pub(super) switch_state: u8,
    pub(super) level_range: bool,
    pub(super) incr_fine: bool,
    pub(super) incr_diff: i32,
    pub(super) burst_count: u8,
    pub(super) verbose: bool,
    pub(super) changed_flag: bool,
    pub(super) lcd_present: bool,
    pub(super) modify: Modify,
    pub(super) first_turn: bool,
    pub(super) status: StatusFlags,
    pub(super) err_count: i32,
    pub(super) err_flag: bool,

    // Parser scratch values.
    pub(super) param: f64,
    pub(super) param_int: i32,
    pub(super) param_byte: u8,
    pub(super) param_long: i32,
    pub(super) digits: usize,
    pub(super) nachkomma: usize,
    pub(super) param_str: String,
    pub(super) ser_inp_str: String,
}
impl Default for FirmwareState {
    fn default() -> Self {
        let defaults = EepromDefaults::default();
        Self {
            defaults: defaults.clone(),
            slave_ch: 0,
            current_ch: 255,
            sub_ch: 254,
            frequenz: defaults.init_frequenz,
            terz_num: 9,
            dac_level: defaults.init_level,
            offset_mv: 0,
            wave: defaults.init_wave,
            burst_mode: defaults.init_burst,
            inc_rast: defaults.init_inc_rast,
            attn_switch_point: 1001.0,
            attn_fac: 40.0,
            pwr_gain: 2.0,
            level_scale_low: 1.0,
            level_scale_hi: 1.0,
            dss_cmd: 0,
            wave_cmd: C_DDS_SQUARE_CMD,
            dds_frequ: 0,
            switch_state: 0,
            level_range: false,
            incr_fine: false,
            incr_diff: 0,
            burst_count: 1,
            verbose: false,
            changed_flag: true,
            lcd_present: false,
            modify: Modify::FreqSel,
            first_turn: true,
            status: StatusFlags::default(),
            err_count: 0,
            err_flag: false,
            param: 0.0,
            param_int: 0,
            param_byte: 0,
            param_long: 0,
            digits: 2,
            nachkomma: 1,
            param_str: String::new(),
            ser_inp_str: String::new(),
        }
    }
}
impl FirmwareState {
    // Restore the editable setpoints that the Pascal firmware copied from
    // EEPROM during reset and after EEPROM writes.
    pub(super) fn patch_copy_from_ee(&mut self) {
        self.wave = self.defaults.init_wave;
        self.frequenz = self.defaults.init_frequenz;
        self.dac_level = self.defaults.init_level;
        self.terz_num = self.defaults.init_terz_num;
        self.burst_mode = self.defaults.init_burst;
        self.inc_rast = self.defaults.init_inc_rast;
        self.attn_switch_point = 1001.0;
        self.attn_fac = self.defaults.init_attn_fac;
        self.pwr_gain = self.defaults.init_pwr_gain;
        self.level_scale_low = self.defaults.level_scale_low;
        self.level_scale_hi = self.defaults.level_scale_hi;
    }

    pub(super) fn ser_crlf<H: HardwareInterface>(&self, hw: &mut H) {
        hw.serout_byte(b'\r');
        hw.serout_byte(b'\n');
    }

    pub(super) fn write_ch_prefix<H: HardwareInterface>(&self, hw: &mut H) {
        let mut prefix = String::new();
        let _ = write!(&mut prefix, "#{}:{}=", self.slave_ch, self.sub_ch);
        hw.write_serial(&prefix);
    }

    pub(super) fn write_ser_inp<H: HardwareInterface>(&self, hw: &mut H) {
        hw.write_serial(&self.ser_inp_str);
        self.ser_crlf(hw);
    }

    // Error/status response. The original parser used sub-channel 255 for
    // these prompts and encoded status bits plus the error number together.
    pub(super) fn ser_prompt<H: HardwareInterface>(&mut self, hw: &mut H, err: ErrorCode, status: u8) {
        if self.verbose || err != ErrorCode::NoErr {
            self.sub_ch = ERR_SUB_CH;
            self.write_ch_prefix(hw);
            let code = err.code().saturating_add(status);
            let _ = write!(
                &mut self.param_str,
                "{} {}",
                code,
                ERR_STR_ARR[err.code() as usize]
            );
            hw.write_serial(&self.param_str);
            self.ser_crlf(hw);
            self.param_str.clear();
        }
        if err != ErrorCode::NoErr {
            self.err_count += 1;
            self.err_flag = true;
        }
    }

    pub(super) fn write_param_str_ser<H: HardwareInterface>(&self, hw: &mut H) {
        self.write_ch_prefix(hw);
        hw.write_serial(&self.param_str);
        self.ser_crlf(hw);
    }

    pub(super) fn param_to_str(&mut self) {
        self.param_str.clear();
        if self.param == 0.0 {
            self.param_str.push('0');
            return;
        }

        let rendered = format!("{:.*}", self.nachkomma, self.param);
        self.param_str
            .push_str(rendered.trim_end_matches('0').trim_end_matches('.'));
        if self.param_str.is_empty() {
            self.param_str.push('0');
        }
    }

    pub(super) fn param_to_pm_str(&mut self) {
        self.param_to_str();
        if !self.param_str.starts_with('-') {
            self.param_str.insert(0, '+');
        }
    }

    pub(super) fn param_long_to_str(&mut self) {
        self.param = self.param_long as f64 / 10.0;
        self.param_to_str();
    }

    pub(super) fn offset_to_param(&mut self) {
        self.param = self.offset_mv as f64 / 1000.0;
    }

    pub(super) fn write_param_ser<H: HardwareInterface>(&mut self, hw: &mut H) {
        self.param_to_str();
        self.write_param_str_ser(hw);
    }

    pub(super) fn write_param_byte_ser<H: HardwareInterface>(&mut self, hw: &mut H) {
        self.param_str.clear();
        let _ = write!(&mut self.param_str, "{}", self.param_byte);
        self.write_param_str_ser(hw);
    }

    // Clamp user-facing values to the same legal ranges as the Pascal code.
    // A true return means the input had to be corrected.
    pub(super) fn check_limits(&mut self) -> bool {
        let mut out_of_range = false;

        // The panel toggles between the 1 V and 5 V amplitude ranges rather
        // than allowing arbitrary DAC full-scale values here.
        self.dac_level = if self.dac_level > 1000.0 {
            5000.0
        } else {
            1000.0
        };

        if self.frequenz < 0 {
            self.frequenz = 0;
            out_of_range = true;
        }
        if self.frequenz > 100_000_001 {
            self.frequenz = 100_000_000;
            out_of_range = true;
        }
        if self.wave > C_SQUW {
            self.wave = C_SQUW;
            out_of_range = true;
        }
        if self.terz_num > 30 {
            self.terz_num = 30;
            out_of_range = true;
        }
        if self.burst_mode > 100 {
            self.burst_mode = 100;
            out_of_range = true;
        }

        out_of_range
    }

    pub(super) fn parse_get_param<H: HardwareInterface>(&mut self, hw: &mut H) {
        self.digits = 2;
        self.nachkomma = 1;

        match self.sub_ch {
            0 => {
                self.param_long = self.frequenz;
                self.param_long_to_str();
                self.write_param_str_ser(hw);
            }
            1 => {
                self.param = self.dac_level;
                self.param_to_str();
                self.write_param_str_ser(hw);
            }
            4 => {
                self.param_byte = self.wave;
                self.write_param_byte_ser(hw);
            }
            5 => {
                self.param_byte = self.burst_mode;
                self.write_param_byte_ser(hw);
            }
            20 => {
                self.nachkomma = 4;
                self.offset_to_param();
                self.write_param_ser(hw);
            }
            80 => {
                self.param_byte = match self.modify {
                    Modify::WaveSel => 0,
                    Modify::FreqSel => 1,
                    Modify::AmplSel => 2,
                    Modify::BurstSel => 3,
                    Modify::DcSel => 4,
                };
                self.write_param_byte_ser(hw);
            }
            89 => {
                self.param_byte = self.inc_rast as u8;
                self.write_param_byte_ser(hw);
            }
            251 => {
                self.param = self.err_count as f64;
                self.write_param_ser(hw);
            }
            252 => {
                self.param_byte = self.defaults.ee_ser_baud_reg;
                self.write_param_byte_ser(hw);
            }
            253 => {
                hw.write_serial(&self.ser_inp_str);
                self.ser_crlf(hw);
            }
            254 => {
                self.write_ch_prefix(hw);
                hw.write_serial(VERS1_STR);
                self.ser_crlf(hw);
            }
            250 | 255 => {
                // Both status aliases end up in the same status prompt path.
                self.ser_prompt(hw, ErrorCode::NoErr, self.status.to_status_byte());
            }
            _ => self.ser_prompt(hw, ErrorCode::ParamErr, 0),
        }
    }

    pub(super) fn parse_set_param<H: HardwareInterface>(&mut self, hw: &mut H) {
        if self.status.busy {
            self.ser_prompt(hw, ErrorCode::BusyErr, 0);
            return;
        }

        self.changed_flag = true;

        match self.sub_ch {
            0 => self.frequenz = (self.param * 10.0) as i32,
            1 => self.dac_level = self.param,
            4 => self.wave = self.param_byte,
            5 => self.burst_mode = self.param_byte,
            80 => {
                // DSP selects which value the front-panel encoder edits.
                self.modify = match self.param_byte {
                    0 => Modify::WaveSel,
                    1 => Modify::FreqSel,
                    2 => Modify::AmplSel,
                    3 => Modify::BurstSel,
                    4 => Modify::DcSel,
                    _ => {
                        self.ser_prompt(hw, ErrorCode::ParamErr, 0);
                        return;
                    }
                };
            }
            89 => {
                if self.status.ee_unlocked {
                    self.inc_rast = self.param_int;
                    self.defaults.init_inc_rast = self.inc_rast;
                } else {
                    self.ser_prompt(hw, ErrorCode::LockedErr, 0);
                    return;
                }
            }
            251 => self.err_count = self.param_int,
            252 => {
                if self.status.ee_unlocked {
                    self.defaults.ee_ser_baud_reg = self.param_byte;
                } else {
                    self.ser_prompt(hw, ErrorCode::LockedErr, 0);
                    return;
                }
            }
            250 => {}
            _ => {
                self.ser_prompt(hw, ErrorCode::ParamErr, 0);
                return;
            }
        }

        // WEN acts as the write-enable latch for EEPROM-backed parameters.
        self.status.ee_unlocked = self.sub_ch == 250;

        if self.check_limits() {
            self.ser_prompt(hw, ErrorCode::ParamErr, self.status.to_status_byte());
        } else {
            self.ser_prompt(hw, ErrorCode::NoErr, self.status.to_status_byte());
        }
        self.set_level_dds(hw);
    }

    pub(super) fn cmd_to_index(cmd: &str) -> CmdWhich {
        for (text, _, which) in CMD_TABLE {
            if cmd.eq_ignore_ascii_case(text) {
                return which;
            }
        }
        CmdWhich::Err
    }

    // Extract either a command token or a numeric parameter token from the
    // serial line. This mirrors the split parser in the Pascal firmware.
    pub(super) fn parse_extract(&self, input: &str, start: usize) -> (String, usize, bool) {
        let bytes = input.as_bytes();
        let mut idx = start;
        while idx < bytes.len() && bytes[idx] == b' ' {
            idx += 1;
        }

        if idx >= bytes.len() {
            return (String::new(), idx, false);
        }

        let is_param = matches!(bytes[idx], b'*'..=b'9');
        let begin = idx;

        if is_param {
            while idx < bytes.len() {
                if matches!(bytes[idx], b'*'..=b'9') {
                    idx += 1;
                } else {
                    break;
                }
            }
        } else {
            while idx < bytes.len() {
                let ch = bytes[idx] as char;
                if ch.is_ascii_alphabetic() {
                    idx += 1;
                } else {
                    break;
                }
            }
        }

        (input[begin..idx].to_string(), idx, is_param)
    }

    pub(super) fn parse_sub_ch<H: HardwareInterface>(&mut self, hw: &mut H) {
        if self.ser_inp_str.is_empty() {
            self.ser_prompt(hw, ErrorCode::NoErr, 0);
            return;
        }

        // Requests have no '='. '#'-prefixed lines are already results and are
        // forwarded unchanged so chained devices preserve upstream replies.
        let has_main_ch = self.ser_inp_str.contains(':');
        let is_request = !self.ser_inp_str.contains('=');
        let first_char = self.ser_inp_str.chars().next().unwrap_or_default();
        let is_omni = first_char == '*';
        let is_result = first_char == '#';

        if is_result {
            self.write_ser_inp(hw);
            return;
        }

        let mut next_idx = 0;
        if has_main_ch {
            let (main_ch_str, main_ch_end, _) = self.parse_extract(&self.ser_inp_str, 0);
            next_idx = main_ch_end.saturating_add(1);
            if is_omni {
                self.write_ser_inp(hw);
            } else {
                self.current_ch = main_ch_str.parse::<u8>().unwrap_or(self.current_ch);
            }

            if !is_omni && self.current_ch != self.slave_ch {
                self.write_ser_inp(hw);
                return;
            }
        } else if !is_omni && self.current_ch != self.slave_ch {
            self.write_ser_inp(hw);
            return;
        }

        let (token, token_end, token_is_param) = self.parse_extract(&self.ser_inp_str, next_idx);
        if token_is_param {
            self.sub_ch = token.parse::<u8>().unwrap_or(self.sub_ch);
        } else {
            let which = Self::cmd_to_index(&token);
            if which == CmdWhich::Err {
                self.ser_prompt(hw, ErrorCode::SyntaxErr, 0);
                return;
            }
            let offset = CMD_TABLE
                .iter()
                .find(|(_, _, candidate)| *candidate == which)
                .map(|(_, sub_ch, _)| *sub_ch)
                .unwrap_or(0);
            let (sub_param, _, _) = self.parse_extract(&self.ser_inp_str, token_end);
            let direct = sub_param.parse::<u8>().unwrap_or(0);
            self.sub_ch = direct.saturating_add(offset);
        }

        // '!' or '?' request the verbose response form.
        self.verbose = self.ser_inp_str.contains('?') || self.ser_inp_str.contains('!');

        if let Some(check_pos) = self.ser_inp_str.find('$') {
            // Optional XOR checksum over the command, excluding the '$' prefix
            // and the checksum bytes themselves.
            let checksum_in = u8::from_str_radix(
                self.ser_inp_str
                    .get(check_pos + 1..check_pos + 3)
                    .unwrap_or("00"),
                16,
            )
            .unwrap_or(0);
            let checksum = self.ser_inp_str[..check_pos]
                .bytes()
                .fold(0u8, |acc, byte| acc ^ byte);
            if checksum != checksum_in {
                self.ser_prompt(hw, ErrorCode::ChecksumErr, 0);
                return;
            }
        }

        hw.set_activity_led(true);

        if is_request {
            self.parse_get_param(hw);
            return;
        }

        if let Some(eq_pos) = self.ser_inp_str.find('=') {
            let (param_str, _, is_param) = self.parse_extract(&self.ser_inp_str, eq_pos + 1);
            if !is_param {
                self.ser_prompt(hw, ErrorCode::ParamErr, 0);
                return;
            }

            self.param = param_str.parse::<f64>().unwrap_or(0.0);
            self.param_int = self.param as i32;
            self.param_byte = self.param_int as u8;
            self.parse_set_param(hw);
        } else {
            self.ser_prompt(hw, ErrorCode::ParamErr, 0);
        }
    }

    // Burst generation runs from the 10 ms system tick. Count 1 starts the
    // programmed waveform, count 0 forces DDS reset, then the period reloads.
    pub(super) fn on_systick<H: HardwareInterface>(&mut self, hw: &mut H) {
        if self.burst_mode == 0 {
            return;
        }

        if self.burst_count == 1 {
            self.dss_cmd = self.wave_cmd;
            hw.send_dds_word(self.dss_cmd);
        }
        if self.burst_count == 0 {
            self.dss_cmd = C_DDS_RESET_CMD;
            hw.send_dds_word(self.dss_cmd);
            self.burst_count = self.burst_mode.saturating_add(1);
        }
        self.burst_count = self.burst_count.saturating_sub(1);
    }

    // Apply the relay state, then emit the AD9833 frequency words followed by
    // the waveform command. SQG kept the original float-based digit-summing
    // path instead of replacing it with new integer math.
    pub(super) fn set_level_dds<H: HardwareInterface>(&mut self, hw: &mut H) {
        self.switch_state = 0;
        let mut offset_mv = self.offset_mv;

        // Zero offset disconnects the DC offset path; non-zero values keep the
        // DAC path enabled and are shifted after relay selection below.
        self.set_switch_bit(OFFS_SW_BIT, offset_mv == 0);

        let level = if self.dac_level < self.attn_switch_point {
            let scaled = (self.dac_level * self.attn_fac * self.level_scale_low).round() as i32;
            self.set_switch_bit(ATTN_SW_BIT, true);
            if self.level_range {
                self.dss_cmd = C_DDS_RESET_CMD;
                hw.send_dds_word(self.dss_cmd);
                hw.shift_out_level_sr(0, self.switch_state);
                hw.delay_ms(5);
                self.level_range = false;
            }
            scaled
        } else {
            self.set_switch_bit(ATTN_SW_BIT, false);
            self.level_range = true;
            (self.dac_level * self.level_scale_hi).round() as i32
        };

        // Logic mode reuses the DDS square-wave output stage.
        self.wave_cmd = match self.wave {
            C_SINW => C_DDS_SINE_CMD,
            C_TRIW => C_DDS_TRIANGLE_CMD,
            C_SQUW => {
                self.set_switch_bit(SQUARE_SW_BIT, true);
                C_DDS_SQUARE_CMD
            }
            C_LOGIC => {
                self.set_switch_bit(SQUARE_SW_BIT, true);
                offset_mv = (self.dac_level * self.pwr_gain * 1.41421).round() as i32;
                self.set_switch_bit(OFFS_SW_BIT, false);
                C_DDS_SQUARE_CMD
            }
            C_EXT => {
                self.set_switch_bit(EXT_ON_BIT, true);
                C_DDS_RESET_CMD
            }
            _ => C_DDS_RESET_CMD,
        };

        hw.shift_out_offset_dac((offset_mv / 5) as i16);
        hw.shift_out_level_sr(level, self.switch_state);

        // Frequency is stored in 0.1 Hz and split into the same nine decimal
        // decades as Pascal without allocating a temporary string.
        let normalized_frequency = self.frequenz.max(0);
        let mut divisor = 100_000_000;
        let mut add_f = 0.0f64;
        for factor in FHZ {
            let digit = (normalized_frequency / divisor) % 10;
            add_f += factor * f64::from(digit);
            divisor /= 10;
        }
        self.dds_frequ = add_f as i32;

        // AD9833 frequency programming is split into two 14-bit register words.
        self.dss_cmd = ((self.dds_frequ as u16) & 0x3fff) | DDS_FREQ_REG_CMD;
        hw.send_dds_word(self.dss_cmd);

        let shifted = (self.dds_frequ as u32) << 2;
        self.dss_cmd = (((shifted >> 16) as u16) & 0x3fff) | DDS_FREQ_REG_CMD;
        hw.send_dds_word(self.dss_cmd);

        self.dss_cmd = self.wave_cmd;
        hw.send_dds_word(self.dss_cmd);
    }

    // Regelmaessig ausserhalb des Interrupts aus CheckSer heraus aufgerufen.
    pub(super) fn chores(&mut self) {}

    pub(super) fn check_ser<H: HardwareInterface>(&mut self, hw: &mut H) {
        while let Some(ch) = hw.serial_timeout_char(2) {
            // The original loop accepted printable 7-bit ASCII only, handled
            // backspace locally, and parsed on carriage return.
            if (' '..='~').contains(&ch) {
                self.ser_inp_str.push(ch);
            }
            if ch == '\u{0008}' {
                self.ser_inp_str.pop();
            }
            if ch == '\r' {
                self.parse_sub_ch(hw);
                self.ser_inp_str.clear();
            }
        }
    }

    pub(super) fn check_delay<H: HardwareInterface>(&mut self, hw: &mut H, delay_ticks: u8) {
        for _ in 0..delay_ticks {
            self.check_ser(hw);
        }
    }

    pub(super) fn set_switch_bit(&mut self, bit: u8, high: bool) {
        if high {
            self.switch_state |= 1 << bit;
        } else {
            self.switch_state &= !(1 << bit);
        }
    }

    pub(super) fn modify_to_sub_ch(&self) -> u8 {
        match self.modify {
            Modify::FreqSel => 0,
            Modify::AmplSel => 1,
            Modify::WaveSel => 4,
            Modify::BurstSel => 5,
            Modify::DcSel => 20,
        }
    }

    pub(super) fn cycle_modify(&mut self, forward: bool) {
        self.modify = match (self.modify, forward) {
            (Modify::WaveSel, true) => Modify::FreqSel,
            (Modify::FreqSel, true) => Modify::AmplSel,
            (Modify::AmplSel, true) => Modify::BurstSel,
            (Modify::BurstSel, true) => Modify::DcSel,
            (Modify::DcSel, true) => Modify::WaveSel,
            (Modify::WaveSel, false) => Modify::DcSel,
            (Modify::FreqSel, false) => Modify::WaveSel,
            (Modify::AmplSel, false) => Modify::FreqSel,
            (Modify::BurstSel, false) => Modify::AmplSel,
            (Modify::DcSel, false) => Modify::BurstSel,
        };
    }

    pub(super) fn report_panel_activity<H: HardwareInterface>(&mut self, hw: &mut H, status: u8) {
        self.ser_prompt(
            hw,
            ErrorCode::NoErr,
            self.status.to_status_byte().saturating_add(status),
        );
    }

    pub(super) fn service_panel_event<H: HardwareInterface>(&mut self, hw: &mut H, event: PanelEvent) {
        match event {
            PanelEvent::None => {}
            PanelEvent::Encoder(delta) => {
                if delta == 0 {
                    return;
                }

                self.changed_flag = true;
                self.status.busy = true;
                self.incr_diff += delta;

                if self.incr_diff.abs() < self.inc_rast {
                    return;
                }

                let mut incr_diff = self.incr_diff / self.inc_rast;
                let incr_diff_byte = incr_diff as u8;
                if incr_diff.abs() > 1 {
                    incr_diff *= 2;
                }
                if incr_diff.abs() > 2 {
                    incr_diff *= 2;
                }
                let incr_acc_int10 = incr_diff * 10;

                if self.first_turn {
                    self.report_panel_activity(hw, USER_SRQ_PANEL_ACTIVE);
                }

                match self.modify {
                    Modify::FreqSel => {
                        if self.incr_fine {
                            if self.first_turn {
                                self.frequenz = (self.frequenz / 10) * 10;
                            }
                            self.frequenz += incr_acc_int10;
                        } else {
                            self.terz_num = self.terz_num.wrapping_add(incr_diff_byte);
                            self.check_limits();
                            self.frequenz = TERZ_ARRAY[self.terz_num as usize];
                        }
                    }
                    Modify::AmplSel => {
                        self.level_range = !self.level_range;
                        self.dac_level = if self.level_range { 5000.0 } else { 1000.0 };
                    }
                    Modify::WaveSel => {
                        self.wave = self.wave.wrapping_add(incr_diff_byte);
                        self.check_limits();
                    }
                    Modify::BurstSel => {
                        self.burst_mode = self.burst_mode.wrapping_add(incr_diff as u8);
                        self.check_limits();
                    }
                    Modify::DcSel => {
                        self.offset_mv += incr_acc_int10;
                    }
                }

                self.incr_diff = 0;
                self.check_limits();
                self.sub_ch = self.modify_to_sub_ch();
                self.parse_get_param(hw);
                self.set_level_dds(hw);
                self.first_turn = false;
            }
            PanelEvent::Button(button) => {
                self.changed_flag = true;
                self.status.busy = true;
                match button {
                    PanelButton::Enter => {
                        self.report_panel_activity(hw, USER_SRQ_PANEL_ACTIVE);
                        self.incr_fine = !self.incr_fine;
                    }
                    PanelButton::Left => {
                        self.report_panel_activity(hw, USER_SRQ_LEFT);
                        self.cycle_modify(true);
                    }
                    PanelButton::Right => {
                        self.report_panel_activity(hw, USER_SRQ_RIGHT);
                        self.cycle_modify(false);
                    }
                }
                self.sub_ch = self.modify_to_sub_ch();
                self.parse_get_param(hw);
                self.set_level_dds(hw);
                self.first_turn = false;
            }
            PanelEvent::IncrTimerExpired => {
                if !self.first_turn {
                    self.report_panel_activity(hw, USER_SRQ_RELEASED);
                }
                self.first_turn = true;
            }
            PanelEvent::DisplayTimerExpired => {
                self.incr_fine = false;
                self.status.busy = false;
                self.changed_flag = false;
            }
        }
    }

    // Startup sequence after reset, before the main serial/panel loop begins.
    pub(super) fn init_all<H: HardwareInterface>(&mut self, hw: &mut H) {
        if !(9..=239).contains(&self.defaults.ee_ser_baud_reg) {
            self.defaults.ee_ser_baud_reg = 51;
        }
        hw.set_serial_baud_register(self.defaults.ee_ser_baud_reg, true);

        self.patch_copy_from_ee();
        self.slave_ch = hw.read_slave_channel();
        hw.set_activity_led(true);

        self.lcd_present = hw.lcd_setup();
        if self.lcd_present {
            hw.lcd_define_custom_char(0, [0x01, 0x03, 0x07, 0x0F, 0x07, 0x03, 0x01, 0x00]);
            hw.lcd_define_custom_char(1, [0x01, 0x03, 0x05, 0x09, 0x05, 0x03, 0x01, 0x00]);
            hw.lcd_define_custom_char(2, [0x01, 0x02, 0x05, 0x0A, 0x05, 0x02, 0x01, 0x00]);
            hw.lcd_write_line(0, VERS3_STR);
            if self.defaults.ee_initialized != EEPROM_INITIALIZED {
                hw.lcd_write_line(1, EEPROM_EMPTY_STR);
            } else {
                hw.lcd_write_line(1, &format!("{ADR_STR}{}", self.slave_ch));
            }
        }

        hw.delay_ms(1000);
        if self.slave_ch > 0 {
            for _ in 0..self.slave_ch {
                hw.set_activity_led(false);
                hw.delay_ms(150);
                hw.set_activity_led(true);
                hw.delay_ms(150);
            }
        }
        hw.set_activity_led(false);

        // This matches the Pascal power-up state before the first user action.
        self.status = StatusFlags::default();
        self.burst_count = 1;
        self.modify = Modify::FreqSel;
        self.incr_fine = false;
        self.incr_diff = 0;
        self.first_turn = true;
        self.current_ch = 255;
        self.err_count = 0;
        self.err_flag = false;
        self.changed_flag = true;
        self.ser_inp_str.clear();
        while hw.serial_read_immediate().is_some() {}
        self.level_range = self.dac_level > 1000.0;

        hw.delay_ms(500);

        self.sub_ch = 254;
        self.write_ch_prefix(hw);
        hw.write_serial(VERS1_STR);
        if self.defaults.ee_initialized != EEPROM_INITIALIZED {
            hw.write_serial(EEPROM_EMPTY_STR);
        }
        self.ser_crlf(hw);

        self.set_level_dds(hw);
    }

    // One best-effort outer loop step from the original `loop ... endloop`.
    // The Pascal loop serviced serial traffic first, then let the optional
    // LCD/encoder panel own the device while the UART was idle.
    pub(super) fn run_main_loop_iteration<H: HardwareInterface>(&mut self, hw: &mut H) {
        while hw.take_systick() {
            self.on_systick(hw);
        }

        self.check_ser(hw);

        if !hw.serial_pending() && self.lcd_present {
            let event = hw.next_panel_event();
            self.service_panel_event(hw, event);
        }
    }
}
