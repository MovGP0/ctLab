use super::*;

#[derive(Debug, Clone)]
pub struct DeviceState<H> {
    pub hw: H,
    pub eeprom: EepromData,
    pub status: RuntimeStatus,
    pub slave_channel: u8,
    pub current_channel: u8,
    pub frequency_tenths_hz: i32,
    pub terz_num: u8,
    pub offset_mv: i32,
    pub dac_level: Float,
    pub dac_level_max: Float,
    pub db: Float,
    pub db_max: Float,
    pub pwr_gain: Float,
    pub attn_fac: Float,
    pub attn_switch_point: Float,
    pub waveform: Waveform,
    pub burst_mode: u8,
    pub burst_count: u8,
    pub burst_gate_open: bool,
    pub input_level_mv: Float,
    pub range: InputRange,
    pub old_range: InputRange,
    pub range_str: &'static str,
    pub input_gain_fac: Float,
    pub panel_modify: Modify,
    pub inc_rast: i32,
    pub incr_fine: bool,
    pub encoder_delta_accum: i32,
    pub lcd_present: bool,
    pub serial_baud_reg: u8,
    pub burst_timer_ticks: u8,
    pub activity_timer_ticks: u8,
    pub display_timer_ticks: u16,
    pub incr_timer_ticks: u16,
    pub level_range_high: bool,
    pub changed_flag: bool,
    pub first_turn: bool,
    pub err_count: i32,
    pub err_flag: bool,
    pub ser_input: String,
    pub param_str: String,
}
impl<H: DdsHardware> DeviceState<H> {
    pub fn new(hw: H) -> Self {
        let mut state = Self {
            hw,
            eeprom: EepromData::default(),
            status: RuntimeStatus::default(),
            slave_channel: 0,
            current_channel: 255,
            frequency_tenths_hz: 10_000,
            terz_num: 17,
            offset_mv: 0,
            dac_level: 1.0,
            dac_level_max: DAC_LEVEL_MAX,
            db: 0.0,
            db_max: 0.0,
            pwr_gain: 2.0,
            attn_fac: 40.0,
            attn_switch_point: DAC_LEVEL_MAX / 40.0,
            waveform: Waveform::Sine,
            burst_mode: 0,
            burst_count: 1,
            burst_gate_open: true,
            input_level_mv: 0.0,
            range: InputRange::Ac1V,
            old_range: InputRange::NoRange,
            range_str: "In    1V",
            input_gain_fac: 1.0,
            panel_modify: Modify::FreqSel,
            inc_rast: 4,
            incr_fine: false,
            encoder_delta_accum: 0,
            lcd_present: false,
            serial_baud_reg: 51,
            burst_timer_ticks: 0,
            activity_timer_ticks: 0,
            display_timer_ticks: 0,
            incr_timer_ticks: 0,
            level_range_high: false,
            changed_flag: false,
            first_turn: true,
            err_count: 0,
            err_flag: false,
            ser_input: String::new(),
            param_str: String::new(),
        };
        state.patch_copy_from_ee();
        state.switch_range();
        state.db = state.dac_level_to_db(state.dac_level);
        state.level_range_high = state.dac_level > 1_000.0;
        state
    }

    pub(super) fn pascal_byte(value: Float) -> u8 {
        (value as i32) as u8
    }

    pub(super) fn format_param(value: Float, decimals: usize) -> String {
        if value.abs() < 0.000_05 {
            "0".to_string()
        } else {
            format!("{value:.decimals$}")
        }
    }

    pub(super) fn format_param_pm(value: Float, decimals: usize) -> String {
        let mut text = Self::format_param(value, decimals);
        if !text.starts_with('-') {
            text.insert(0, '+');
        }
        text
    }

    pub(super) fn format_tenths_hz(value: i32) -> String {
        Self::format_param(value as Float / 10.0, 1)
    }

    pub(super) fn pascal_add_byte(base: u8, delta: i32) -> u8 {
        base.wrapping_add(delta as u8)
    }

    pub(super) fn emit_user_srq(&mut self, status_offset: u8) {
        let masked_status = self.status.as_byte() & 0x2f;
        self.ser_prompt(
            ErrorCode::NoErr,
            masked_status.wrapping_add(status_offset),
            true,
        );
    }

    pub(super) fn parse_get_param_for_panel(&mut self, sub_ch: u8) {
        let _ = self.parse_get_param(sub_ch, true, "");
    }

    pub(super) fn write_ch_prefix(&mut self, sub_ch: u8) {
        self.hw
            .serial_write(&format!("#{}:{}=", self.slave_channel, sub_ch));
    }

    pub fn ser_crlf(&mut self) {
        self.hw.serial_write("\r\n");
    }

    pub fn write_ser_inp(&mut self, text: &str) {
        self.hw.serial_write(text);
        self.ser_crlf();
    }

    pub(super) fn write_param_str_ser(&mut self, sub_ch: u8, value: &str) {
        self.write_ch_prefix(sub_ch);
        self.hw.serial_write(value);
        self.ser_crlf();
    }

    pub(super) fn write_param_byte_ser(&mut self, sub_ch: u8, value: u8) {
        self.write_param_str_ser(sub_ch, &value.to_string());
    }

    pub(super) fn ser_prompt(&mut self, err: ErrorCode, status: u8, verbose: bool) {
        if verbose || err != ErrorCode::NoErr {
            self.write_ch_prefix(ERR_SUB_CH);
            self.hw
                .serial_write(&(status.wrapping_add(err as u8)).to_string());
            self.hw.serial_write(" ");
            self.hw.serial_write(ERR_LABELS[err as usize]);
            self.ser_crlf();
        }

        if err != ErrorCode::NoErr {
            self.err_count += 1;
            self.err_flag = true;
        }
    }

    pub fn patch_copy_from_ee(&mut self) {
        self.waveform = Waveform::from_byte(self.eeprom.init_wave);
        self.pwr_gain = self.eeprom.init_pwr_gain;
        self.frequency_tenths_hz = self.eeprom.init_frequency_tenths_hz;
        self.dac_level = self.rms_to_dac_level(self.eeprom.init_level_mv);
        self.db = self.eeprom.init_db;
        self.terz_num = self.eeprom.init_terz_num;
        self.offset_mv = (self.eeprom.init_offset_v * 1000.0) as i32;
        self.burst_mode = self.eeprom.init_burst;
        self.inc_rast = self.eeprom.init_inc_rast;
        self.dac_level_max = DAC_LEVEL_MAX;
        self.attn_fac = self.eeprom.init_attn_fac;
        self.attn_switch_point = self.dac_level_max / self.attn_fac.max(0.001);
        self.burst_gate_open = true;
        self.level_range_high = self.dac_level > 1_000.0;
        self.set_limits();
    }

    pub fn on_sys_tick(&mut self) {
        self.input_level_mv = self.hw.read_input_level();
        self.status.overload_flag = self.hw.read_input_overload();

        if self.burst_mode != 0 {
            if self.burst_count == 1 {
                self.burst_gate_open = true;
                self.hw.set_waveform(self.effective_waveform());
            }

            if self.burst_count == 0 {
                self.burst_gate_open = false;
                self.hw.set_waveform(Waveform::Off);
                self.burst_count = self.burst_mode.saturating_add(1);
            }

            self.burst_count = self.burst_count.wrapping_sub(1);
        }
    }

    pub fn dac_level_to_rms(&self, mut level: Float) -> Float {
        level *= self.pwr_gain;
        match self.waveform {
            Waveform::Triangle => level * TRIANGLE_RMS_FACTOR,
            Waveform::Square | Waveform::Logic => level * SQUARE_RMS_FACTOR,
            _ => level,
        }
    }

    pub fn rms_to_dac_level(&self, mut level: Float) -> Float {
        level /= self.pwr_gain.max(0.001);
        match self.waveform {
            Waveform::Triangle => level * TRIANGLE_DAC_FACTOR,
            Waveform::Square | Waveform::Logic => level * SQUARE_DAC_FACTOR,
            _ => level,
        }
    }

    pub fn level_to_db(&self, level: Float) -> Float {
        20.0 * (level / DB_REFERENCE_MV).log10()
    }

    pub fn db_to_level(&self, db: Float) -> Float {
        DB_REFERENCE_MV * 10.0_f32.powf(db / 20.0)
    }

    pub fn db_to_dac_level(&self, db: Float) -> Float {
        self.rms_to_dac_level(self.db_to_level(db))
    }

    pub fn dac_level_to_db(&self, level: Float) -> Float {
        self.level_to_db(self.dac_level_to_rms(level))
    }

    pub fn dac_level_to_peak_mv(&self) -> Float {
        self.dac_level * self.pwr_gain * PEAK_FACTOR
    }

    pub fn set_limits(&mut self) {
        self.db_max = self.dac_level_to_db(self.dac_level_max);
    }

    pub fn switch_range(&mut self) {
        let adc_scale = self
            .eeprom
            .adc_scales
            .get((self.range.as_byte()).min(3) as usize)
            .copied()
            .unwrap_or(1.0);
        let inp_gain = INP_GAINS
            .get((self.range.as_byte()).min(3) as usize)
            .copied()
            .unwrap_or(1.0);
        self.input_gain_fac = inp_gain * adc_scale;

        if self.range == self.old_range {
            return;
        }

        self.old_range = self.range;
        self.range_str = match self.range {
            InputRange::Ac100mV => "In 100mV",
            InputRange::Ac1V => "In    1V",
            InputRange::Ac10V => "In   10V",
            InputRange::Ac100V => "In  100V",
            InputRange::NoRange => "In    1V",
        };
        self.hw.set_input_range(self.range);
    }

    pub(super) fn effective_waveform(&self) -> Waveform {
        if self.burst_mode != 0 && !self.burst_gate_open {
            Waveform::Off
        } else {
            self.waveform
        }
    }

    pub(super) fn dds_tuning_word(&self) -> u32 {
        let normalized = self.frequency_tenths_hz.max(0);
        let mut divisor = 10_000_000;
        DDS_FACTORS.into_iter().fold(0_u32, |acc, factor| {
            let digit = (normalized / divisor) % 10;
            divisor /= 10;
            acc.saturating_add(digit as u32 * factor)
        })
    }

    pub(super) fn effective_input_level_mv(&self) -> Float {
        if self.status.overload_flag {
            -9_999.0
        } else {
            self.input_level_mv
        }
    }

    pub fn apply_output_state(&mut self) {
        self.switch_range();
        self.level_range_high = self.dac_level > 1_000.0;
        self.hw.send_dds_frequency_word(self.dds_tuning_word());
        self.hw
            .send_amplitude_word(self.dac_level.clamp(0.0, u16::MAX as Float) as u16);
        if let Waveform::External(index) = self.waveform {
            self.hw.send_aux_config(index);
        }
        self.hw.set_waveform(self.effective_waveform());
    }

    pub fn param_str_on_lcd(&mut self) {
        if !self.lcd_present {
            return;
        }
        self.hw.lcd_write_line(1, &self.param_str);
    }

    pub fn soll_werte_on_lcd(&mut self) {
        if !self.lcd_present {
            return;
        }
        self.hw
            .lcd_write_line(0, &Self::format_tenths_hz(self.frequency_tenths_hz));
        self.hw.lcd_write_line(
            1,
            &Self::format_param(self.dac_level_to_rms(self.dac_level), 1),
        );
    }

    pub fn check_limits(&mut self) -> bool {
        let mut out_of_range = false;

        if self.frequency_tenths_hz < 0 {
            self.frequency_tenths_hz = 0;
            out_of_range = true;
        }
        if self.frequency_tenths_hz > MAX_FREQUENCY_TENTHS_HZ {
            self.frequency_tenths_hz = MAX_FREQUENCY_TENTHS_HZ;
            out_of_range = true;
        }
        if self.dac_level <= 0.0 {
            self.dac_level = 1.0;
            out_of_range = true;
        }
        if self.dac_level > self.dac_level_max {
            self.dac_level = self.dac_level_max;
            out_of_range = true;
        }
        if self.offset_mv < -MAX_OFFSET_MV {
            self.offset_mv = -MAX_OFFSET_MV;
            out_of_range = true;
        }
        if self.offset_mv > MAX_OFFSET_MV {
            self.offset_mv = MAX_OFFSET_MV;
            out_of_range = true;
        }
        if self.db > self.db_max {
            self.db = self.db_max;
            self.dac_level = self.db_to_dac_level(self.db);
            out_of_range = true;
        }
        if self.db < MIN_DB {
            self.db = MIN_DB;
            self.dac_level = self.db_to_dac_level(self.db);
            out_of_range = true;
        }

        let wave_byte = self.waveform.as_byte();
        if wave_byte > 249 {
            self.waveform = Waveform::Off;
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
        if self.range.as_byte() > InputRange::Ac100V.as_byte() {
            self.range = InputRange::Ac100mV;
            out_of_range = true;
        }

        out_of_range
    }

    pub(super) fn parse_get_param(
        &mut self,
        sub_ch: u8,
        verbose: bool,
        raw_line: &str,
    ) -> Result<(), ErrorCode> {
        match sub_ch {
            0 => {
                self.write_param_str_ser(sub_ch, &Self::format_tenths_hz(self.frequency_tenths_hz))
            }
            1 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.dac_level_to_rms(self.dac_level), 1),
            ),
            2 => self
                .write_param_str_ser(sub_ch, &Self::format_param(self.dac_level_to_peak_mv(), 1)),
            3 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.dac_level_to_db(self.dac_level), 2),
            ),
            4 => self.write_param_byte_ser(sub_ch, self.waveform.as_byte()),
            5 => self.write_param_byte_ser(sub_ch, self.burst_mode),
            10 | 99 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.effective_input_level_mv(), 1),
            ),
            11 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.effective_input_level_mv() * PEAK_FACTOR, 1),
            ),
            12 => {
                let input_level = self.effective_input_level_mv();
                let value = if input_level < 0.0 {
                    input_level
                } else {
                    self.level_to_db(input_level)
                };
                self.write_param_str_ser(sub_ch, &Self::format_param(value, 2));
            }
            19 => self.write_param_byte_ser(sub_ch, self.range.as_byte()),
            20 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.offset_mv as Float / 1000.0, 4),
            ),
            80 => self.write_param_byte_ser(sub_ch, self.panel_modify as u8),
            89 => self.write_param_byte_ser(sub_ch, self.inc_rast as u8),
            150 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.eeprom.init_frequency_tenths_hz as Float / 10.0, 1),
            ),
            151 => {
                self.write_param_str_ser(sub_ch, &Self::format_param(self.eeprom.init_level_mv, 1))
            }
            152 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.eeprom.init_logic_level_mv, 1),
            ),
            153 => self.write_param_str_ser(sub_ch, &Self::format_param(self.eeprom.init_db, 2)),
            154 => self.write_param_byte_ser(sub_ch, self.eeprom.init_wave),
            155 => self.write_param_byte_ser(sub_ch, self.eeprom.init_burst),
            170 => {
                self.write_param_str_ser(sub_ch, &Self::format_param(self.eeprom.init_offset_v, 4))
            }
            200 => self
                .write_param_str_ser(sub_ch, &Self::format_param(self.eeprom.level_scale_low, 4)),
            201 => self
                .write_param_str_ser(sub_ch, &Self::format_param(self.eeprom.level_scale_high, 4)),
            202 => self.write_param_str_ser(sub_ch, &Self::format_param(self.pwr_gain, 4)),
            203 => self.write_param_str_ser(sub_ch, &Self::format_param(self.attn_fac, 4)),
            204 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.eeprom.init_logic_level_mv, 4),
            ),
            210..=213 => self.write_param_str_ser(
                sub_ch,
                &Self::format_param(self.eeprom.adc_scales[(sub_ch - 210) as usize], 4),
            ),
            251 => self.write_param_str_ser(sub_ch, &self.err_count.to_string()),
            252 => self.write_param_byte_ser(sub_ch, self.eeprom.ee_ser_baud_reg),
            253 => self.write_ser_inp(raw_line),
            254 => self.write_param_str_ser(sub_ch, VERS1_STR),
            250 | 255 => self.ser_prompt(ErrorCode::NoErr, self.status.as_byte(), verbose),
            _ => return Err(ErrorCode::ParamErr),
        }
        Ok(())
    }

    pub(super) fn parse_set_param(
        &mut self,
        sub_ch: u8,
        param: Float,
        verbose: bool,
    ) -> Result<(), ErrorCode> {
        if self.status.busy_flag {
            return Err(ErrorCode::BusyErr);
        }

        self.changed_flag = true;
        let param_int = param as i32;
        let param_byte = Self::pascal_byte(param);

        match sub_ch {
            0 => self.frequency_tenths_hz = (param * 10.0) as i32,
            1 => {
                self.dac_level = self.rms_to_dac_level(param);
                self.db = self.level_to_db(param);
            }
            2 => {
                self.dac_level = param / self.pwr_gain.max(0.001) / PEAK_FACTOR;
                self.db = self.dac_level_to_db(self.dac_level);
            }
            3 => {
                self.db = param;
                self.dac_level = self.db_to_dac_level(self.db);
            }
            4 => {
                self.waveform = Waveform::from_byte(param_byte);
                self.set_limits();
                self.db = self.dac_level_to_db(self.dac_level);
                if self.waveform == Waveform::Logic {
                    self.dac_level =
                        self.eeprom.init_logic_level_mv / self.pwr_gain.max(0.001) / PEAK_FACTOR;
                    self.db = self.dac_level_to_db(self.dac_level);
                }
            }
            5 => self.burst_mode = param_byte,
            9 => self.hw.send_aux_config(param_byte),
            19 => self.range = InputRange::from_byte(param_byte),
            20 => self.offset_mv = (param * 1000.0) as i32,
            80 => {
                self.panel_modify = Modify::from_byte(param_byte).ok_or(ErrorCode::ParamErr)?;
            }
            89 => {
                if !self.status.ee_unlocked {
                    return Err(ErrorCode::LockedErr);
                }
                self.inc_rast = param_int;
                self.eeprom.init_inc_rast = param_int;
            }
            150..=170 => {
                if !self.status.ee_unlocked {
                    return Err(ErrorCode::LockedErr);
                }
                match sub_ch {
                    150 => self.eeprom.init_frequency_tenths_hz = (param * 10.0) as i32,
                    151 => self.eeprom.init_level_mv = param,
                    152 => self.eeprom.init_logic_level_mv = param,
                    154 => self.eeprom.init_wave = param_byte,
                    155 => self.eeprom.init_burst = param_byte,
                    170 => self.eeprom.init_offset_v = param,
                    _ => return Err(ErrorCode::ParamErr),
                }
            }
            200..=213 => {
                if !self.status.ee_unlocked {
                    return Err(ErrorCode::LockedErr);
                }
                match sub_ch {
                    200 => self.eeprom.level_scale_low = param,
                    201 => self.eeprom.level_scale_high = param,
                    202 => {
                        self.eeprom.init_pwr_gain = param;
                        self.patch_copy_from_ee();
                    }
                    203 => {
                        self.eeprom.init_attn_fac = param;
                        self.patch_copy_from_ee();
                    }
                    204 => {
                        self.eeprom.init_logic_level_mv = param;
                        self.patch_copy_from_ee();
                    }
                    210..=213 => self.eeprom.adc_scales[(sub_ch - 210) as usize] = param,
                    _ => return Err(ErrorCode::ParamErr),
                }
            }
            251 => self.err_count = param_int,
            252 => {
                if !self.status.ee_unlocked {
                    return Err(ErrorCode::LockedErr);
                }
                self.eeprom.ee_ser_baud_reg = param_byte;
            }
            250 => {}
            _ => return Err(ErrorCode::ParamErr),
        }

        self.status.ee_unlocked = sub_ch == 250;
        let out_of_range = self.check_limits();
        self.switch_range();
        self.apply_output_state();
        self.ser_prompt(
            if out_of_range {
                ErrorCode::ParamErr
            } else {
                ErrorCode::NoErr
            },
            self.status.as_byte(),
            verbose,
        );
        Ok(())
    }

    pub fn cmd2_index(&self, text: &str) -> CmdWhich {
        CMD_STR_ARR
            .iter()
            .position(|candidate| *candidate == text)
            .and_then(|index| {
                use CmdWhich::*;
                Some(match index {
                    0 => Str,
                    1 => Idn,
                    2 => Trg,
                    3 => Val,
                    4 => Frq,
                    5 => Lvl,
                    6 => Lvp,
                    7 => Dbv,
                    8 => Wav,
                    9 => Bst,
                    10 => Aux,
                    11 => Inl,
                    12 => Rng,
                    13 => Ofs,
                    14 => Dsp,
                    15 => All,
                    16 => Opt,
                    17 => Scl,
                    18 => Wen,
                    19 => Erc,
                    20 => Sbd,
                    21 => Nop,
                    _ => return None,
                })
            })
            .unwrap_or(CmdWhich::Err)
    }

    pub(super) fn parse_alpha_prefix<'a>(&self, text: &'a str) -> (&'a str, &'a str) {
        let split = text
            .char_indices()
            .find_map(|(index, ch)| (!ch.is_ascii_alphabetic()).then_some(index))
            .unwrap_or(text.len());
        (&text[..split], &text[split..])
    }

    pub(super) fn parse_numeric_prefix<'a>(&self, text: &'a str) -> (&'a str, &'a str) {
        let split = text
            .char_indices()
            .find_map(|(index, ch)| (!(('*'..='9').contains(&ch))).then_some(index))
            .unwrap_or(text.len());
        (&text[..split], &text[split..])
    }

    pub(super) fn parse_subchannel_token(&self, token: &str) -> Result<u8, ErrorCode> {
        let token = token.trim();
        if token.is_empty() {
            return Err(ErrorCode::SyntaxErr);
        }

        let first = token.chars().next().ok_or(ErrorCode::SyntaxErr)?;
        if ('*'..='9').contains(&first) {
            return token.parse::<u8>().map_err(|_| ErrorCode::ParamErr);
        }

        let upper = token.to_ascii_uppercase();
        let (cmd_text, suffix_text) = self.parse_alpha_prefix(&upper);
        let which = self.cmd2_index(cmd_text);
        if which == CmdWhich::Err {
            return Err(ErrorCode::SyntaxErr);
        }

        let cmd_index = CMD_STR_ARR
            .iter()
            .position(|candidate| *candidate == cmd_text)
            .ok_or(ErrorCode::SyntaxErr)?;
        let base = CMD2_SUB_CH_ARR[cmd_index];
        let suffix = if suffix_text.is_empty() {
            0
        } else {
            suffix_text.parse::<u8>().map_err(|_| ErrorCode::ParamErr)?
        };
        Ok(base.wrapping_add(suffix))
    }

    pub(super) fn verify_checksum<'a>(&self, line: &'a str) -> Result<&'a str, ErrorCode> {
        if let Some(pos) = line.find('$') {
            let body = &line[..pos];
            let checksum_text = line.get(pos + 1..pos + 3).ok_or(ErrorCode::ChecksumErr)?;
            let expected =
                u8::from_str_radix(checksum_text, 16).map_err(|_| ErrorCode::ChecksumErr)?;
            let actual = body.bytes().fold(0_u8, |acc, byte| acc ^ byte);
            if actual != expected {
                return Err(ErrorCode::ChecksumErr);
            }
            Ok(body)
        } else {
            Ok(line)
        }
    }

    pub fn process_serial_command(&mut self, line: &str) {
        if line.is_empty() {
            self.ser_prompt(ErrorCode::NoErr, 0, false);
            return;
        }

        if line.starts_with('#') {
            self.write_ser_inp(line);
            return;
        }

        let verbose = line.contains('!') || line.contains('?');
        let checksum_free = match self.verify_checksum(line) {
            Ok(body) => body,
            Err(err) => {
                self.ser_prompt(err, 0, verbose);
                return;
            }
        };

        let parser_input = checksum_free.replace(['!', '?'], "");
        let mut working = parser_input.as_str();

        if let Some((head, rest)) = working.split_once(':') {
            if head == "*" {
                self.write_ser_inp(line);
                working = rest;
            } else {
                let current_channel = match head.parse::<u8>() {
                    Ok(channel) => channel,
                    Err(_) => {
                        self.ser_prompt(ErrorCode::SyntaxErr, 0, verbose);
                        return;
                    }
                };
                self.current_channel = current_channel;
                if current_channel != self.slave_channel {
                    self.write_ser_inp(line);
                    return;
                }
                working = rest;
            }
        }

        let is_request = !working.contains('=');
        let (token, value) = if let Some((lhs, rhs)) = working.split_once('=') {
            (lhs.trim(), Some(rhs.trim()))
        } else {
            (working.trim(), None)
        };

        let sub_ch = match self.parse_subchannel_token(token) {
            Ok(sub_ch) => sub_ch,
            Err(err) => {
                self.ser_prompt(err, 0, verbose);
                return;
            }
        };

        if is_request {
            if let Err(err) = self.parse_get_param(sub_ch, verbose, line) {
                self.ser_prompt(err, 0, verbose);
            }
            return;
        }

        let Some(value_text) = value else {
            self.ser_prompt(ErrorCode::ParamErr, 0, verbose);
            return;
        };

        let param = match value_text.parse::<Float>() {
            Ok(param) => param,
            Err(_) => {
                self.ser_prompt(ErrorCode::ParamErr, 0, verbose);
                return;
            }
        };

        if let Err(err) = self.parse_set_param(sub_ch, param, verbose) {
            self.ser_prompt(err, 0, verbose);
        }
    }

    pub fn handle_panel_event(&mut self, event: PanelEvent) {
        match event {
            PanelEvent::EncoderDelta(delta) => {
                self.activity_timer_ticks = 25;
                self.hw.set_activity_led(true);
                self.encoder_delta_accum =
                    self.encoder_delta_accum.saturating_add(i32::from(delta));
                self.incr_timer_ticks = 20;

                let inc_rast = self.inc_rast.max(1);
                if self.encoder_delta_accum.abs() < inc_rast {
                    return;
                }

                self.status.busy_flag = true;
                self.changed_flag = true;

                let scaled_delta = self.encoder_delta_accum / inc_rast;
                self.encoder_delta_accum = 0;

                let sign = scaled_delta.signum();
                let accel_index =
                    (scaled_delta.unsigned_abs() as usize).min(INCR_ACC_ARRAY.len() - 1);
                let accelerated_delta = sign * INCR_ACC_ARRAY[accel_index];
                let acc_int10 = accelerated_delta * 10;
                let acc_float = accelerated_delta as Float;
                self.display_timer_ticks = 150;

                if self.first_turn {
                    self.emit_user_srq(67);
                }

                match self.panel_modify {
                    Modify::FreqSel => {
                        if self.incr_fine {
                            if self.first_turn {
                                self.frequency_tenths_hz = (self.frequency_tenths_hz / 10) * 10;
                            }
                            self.frequency_tenths_hz =
                                self.frequency_tenths_hz.saturating_add(acc_int10);
                        } else {
                            self.terz_num = Self::pascal_add_byte(self.terz_num, scaled_delta);
                            self.check_limits();
                            self.frequency_tenths_hz = TERZ_ARRAY[self.terz_num as usize];
                        }
                        self.parse_get_param_for_panel(0);
                    }
                    Modify::AmplSel | Modify::PeakSel => {
                        if self.incr_fine {
                            if self.first_turn {
                                self.dac_level = self.dac_level.trunc();
                            }
                            self.dac_level += acc_float;
                            self.check_limits();
                            self.db = self.dac_level_to_db(self.dac_level);
                        } else {
                            if self.first_turn {
                                self.db = self.db.trunc();
                            }
                            self.db += acc_float;
                            self.dac_level = self.db_to_dac_level(self.db);
                        }
                        self.parse_get_param_for_panel(1);
                    }
                    Modify::WaveSel => {
                        let next_wave =
                            Self::pascal_add_byte(self.waveform.as_byte(), accelerated_delta);
                        self.waveform = Waveform::from_byte(next_wave);
                        self.set_limits();
                        self.check_limits();
                        self.parse_get_param_for_panel(4);
                        if let Waveform::External(index) = self.waveform {
                            self.hw.send_aux_config(index);
                        }
                        if self.waveform == Waveform::Logic {
                            self.dac_level = self.eeprom.init_logic_level_mv
                                / self.pwr_gain.max(0.001)
                                / PEAK_FACTOR;
                            self.db = self.dac_level_to_db(self.dac_level);
                        }
                    }
                    Modify::BurstSel => {
                        self.burst_mode = Self::pascal_add_byte(self.burst_mode, accelerated_delta);
                        self.check_limits();
                        self.parse_get_param_for_panel(5);
                    }
                    Modify::DcSel => {
                        if self.incr_fine {
                            self.offset_mv = self.offset_mv.saturating_add(accelerated_delta * 5);
                        } else {
                            if self.first_turn {
                                self.offset_mv = (self.offset_mv / 100) * 100;
                            }
                            self.offset_mv = self.offset_mv.saturating_add(acc_int10 * 10);
                        }
                        self.parse_get_param_for_panel(20);
                    }
                    Modify::InpSel => {
                        self.display_timer_ticks = 10;
                        let next_range = Self::pascal_add_byte(self.range.as_byte(), scaled_delta);
                        self.range = InputRange::from_byte(next_range);
                        self.check_limits();
                        self.switch_range();
                        self.parse_get_param_for_panel(19);
                    }
                }

                self.check_limits();
                self.apply_output_state();
                self.soll_werte_on_lcd();
                self.first_turn = false;
            }
            PanelEvent::ToggleFine => {
                self.handle_panel_event(PanelEvent::Buttons {
                    enter: true,
                    left: false,
                    right: false,
                });
            }
            PanelEvent::NextModify => {
                self.handle_panel_event(PanelEvent::Buttons {
                    enter: false,
                    left: true,
                    right: false,
                });
            }
            PanelEvent::PrevModify => {
                self.handle_panel_event(PanelEvent::Buttons {
                    enter: false,
                    left: false,
                    right: true,
                });
            }
            PanelEvent::Buttons { enter, left, right } => {
                if !(enter || left || right) {
                    return;
                }

                self.status.busy_flag = true;
                self.changed_flag = true;

                if enter {
                    self.emit_user_srq(67);
                    self.incr_fine = !self.incr_fine;
                }
                if left {
                    self.emit_user_srq(65);
                    self.panel_modify = self.panel_modify.next();
                }
                if right {
                    self.emit_user_srq(66);
                    self.panel_modify = self.panel_modify.prev();
                }

                self.display_timer_ticks = 150;
                self.apply_output_state();
                self.soll_werte_on_lcd();
                self.first_turn = false;
            }
            PanelEvent::IncrTimerElapsed => {
                self.incr_timer_ticks = 20;
                if !self.first_turn {
                    self.emit_user_srq(64);
                }
                self.first_turn = true;
            }
            PanelEvent::DisplayTimerElapsed => {
                self.display_timer_ticks = 25;
                self.incr_fine = false;
                self.status.busy_flag = false;
                self.changed_flag = false;
                self.hw.set_activity_led(false);
                self.soll_werte_on_lcd();
            }
            PanelEvent::ReleaseBusy => {
                self.handle_panel_event(PanelEvent::DisplayTimerElapsed);
            }
        }
    }

    pub fn chores(&mut self) {
        self.apply_output_state();
        self.soll_werte_on_lcd();
    }

    pub fn check_ser(&mut self) {
        while let Some(ch) = self.hw.serial_read() {
            match ch {
                '\u{08}' => {
                    self.ser_input.pop();
                }
                '\r' => {
                    let line = self.ser_input.clone();
                    self.ser_input.clear();
                    self.process_serial_command(&line);
                }
                '\n' => {}
                ch if (' '..='\u{7f}').contains(&ch) => self.ser_input.push(ch),
                _ => {}
            }
        }
    }

    pub fn check_delay(&mut self, delay_steps: u8) {
        for _ in 0..delay_steps {
            self.check_ser();
            self.chores();
        }
    }

    pub fn init_all(&mut self) {
        let mut baud_reg = self.eeprom.ee_ser_baud_reg;
        if !(9..=239).contains(&baud_reg) {
            self.eeprom.ee_ser_baud_reg = 51;
            baud_reg = 51;
        }
        self.serial_baud_reg = baud_reg;
        self.hw.set_serial_baud_register(baud_reg, true);

        self.patch_copy_from_ee();
        self.slave_channel = self.hw.read_slave_channel();
        self.hw.set_activity_led(true);

        self.lcd_present = self.hw.lcd_setup();
        if self.lcd_present {
            self.hw.lcd_define_custom_char(0, LCD_CHARSET_0);
            self.hw.lcd_define_custom_char(1, LCD_CHARSET_1);
            self.hw.lcd_define_custom_char(2, LCD_CHARSET_2);
            self.hw.lcd_write_line(0, VERS3_STR);
            if self.eeprom.ee_initialized != EEPROM_INITIALIZED {
                self.hw.lcd_write_line(1, EE_NOT_PROGRAMMED_STR);
            } else {
                self.hw
                    .lcd_write_line(1, &format!("{ADR_STR}{}", self.slave_channel));
            }
        }

        self.old_range = InputRange::NoRange;
        self.range = InputRange::Ac1V;
        self.switch_range();
        self.hw.delay_ms(1000);
        if self.slave_channel > 0 {
            for _ in 0..self.slave_channel {
                self.hw.set_activity_led(false);
                self.hw.delay_ms(150);
                self.hw.set_activity_led(true);
                self.hw.delay_ms(150);
            }
        }
        self.hw.set_activity_led(false);
        self.status = RuntimeStatus::default();
        self.burst_count = 1;
        self.burst_gate_open = true;
        self.burst_timer_ticks = 1;
        self.current_channel = 255;
        self.panel_modify = Modify::FreqSel;
        self.incr_fine = false;
        self.encoder_delta_accum = 0;
        self.activity_timer_ticks = 0;
        self.display_timer_ticks = 0;
        self.incr_timer_ticks = 0;
        self.first_turn = true;
        self.err_count = 0;
        self.err_flag = false;
        self.changed_flag = true;
        self.ser_input.clear();
        while self.hw.serial_read().is_some() {}
        self.level_range_high = self.dac_level > 1_000.0;
        self.hw.delay_ms(500);
        self.apply_output_state();
        self.hw.delay_ms(250);
        self.apply_output_state();
        self.db = self.dac_level_to_db(self.dac_level);
        self.write_ch_prefix(254);
        self.hw.serial_write(VERS1_STR);
        if self.eeprom.ee_initialized != EEPROM_INITIALIZED {
            self.hw.serial_write(EE_NOT_PROGRAMMED_STR);
        }
        self.ser_crlf();
    }
}
