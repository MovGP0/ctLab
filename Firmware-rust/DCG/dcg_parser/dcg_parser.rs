//! Implements the DCG command grammar and calibration-write side effects.

use super::*;

/// Pascal-compatible command interpreter with an explicit state image, allowing parser behavior to be tested without energizing hardware.
pub struct DcgParser {
    /// Requested DCG voltage setpoint in volts parsed or returned through the DCV subchannel.
    pub dc_volt: f32,

    /// Requested DCG current-limit setpoint in amperes parsed or returned through the DCA subchannel.
    pub dc_amp: f32,

    /// Accumulates ah across service intervals so capacity/energy requests do not depend on display refresh timing.
    pub ah: f32,

    /// Accumulates wh across service intervals so capacity/energy requests do not depend on display refresh timing.
    pub wh: f32,

    /// Accumulates dc volt integrated across service intervals so capacity/energy requests do not depend on display refresh timing.
    pub dc_volt_integrated: f32,

    /// Accumulates dc amp integrated across service intervals so capacity/energy requests do not depend on display refresh timing.
    pub dc_amp_integrated: f32,

    /// Measured DCG current in amperes returned by MSA and used for power integration.
    pub curr_amp: f32,

    /// Measured DCG voltage in volts returned by MSV and used for power integration.
    pub curr_volt: f32,

    /// PCV percentage modifier applied to the voltage setpoint without triggering automatic relay range changes.
    pub dc_volt_mod: f32,

    /// PCA percentage modifier applied to the current setpoint without triggering automatic shunt range changes.
    pub dc_amp_mod: f32,

    /// Latest auxiliary supply voltage in volts, used to detect fuse or supply-path loss before enabling the output relay.
    pub input_voltage: f32,

    /// Latest calibrated output voltage in volts, used with the time-aligned current sample for protection, power, and serial measurement replies.
    pub measured_voltage: f32,

    /// Latest calibrated output current in amperes, used for current protection, power calculation, and serial measurement replies.
    pub measured_current: f32,

    /// Latest DCG temperature in degrees Celsius returned by TMP and evaluated by thermal protection.
    pub temperature: f32,

    /// Requested DCG ripple on-phase duration in milliseconds.
    pub pw_on_time: i32,

    /// Requested DCG ripple off-phase duration in milliseconds.
    pub pw_off_time: i32,

    /// Sets the requested voltage drop during the off phase as a percentage of the energized setpoint.
    pub ripple_percent: i32,

    /// Off-phase voltage in volts calculated from the energized setpoint and ripple percentage before DAC quantization.
    pub ripple_voltage: f32,

    /// Suppresses one ripple phase transition while a setpoint or relay update requires a stable output phase.
    pub no_toggle: bool,

    /// Latest unscaled voltage-converter code, retained for `RAW` diagnostics before the active range's offset and volts-per-count factor are applied.
    pub adc_raw_u: u16,

    /// Latest unscaled current-converter code, retained for `RAW` diagnostics before the active shunt's offset and amperes-per-count factor are applied.
    pub adc_raw_i: u16,

    /// Stores raw AVR ADC samples for the six auxiliary board channels used by supply, temperature, and protection calculations.
    pub adc10: [u16; 6],

    /// Calibrated voltage DAC code used during the energized ripple phase.
    pub dac_raw_uon: u16,

    /// Calibrated voltage DAC code used during the off ripple phase.
    pub dac_raw_uoff: u16,

    /// Calibrated current-limit DAC code for the active shunt range.
    pub dac_raw_i: u16,

    /// Maximum raw code of the selected 12- or 16-bit DAC, used to clamp every calibrated output.
    pub dac_max: u16,

    /// Voltage represented by one DAC count after the active converter width and voltage range are applied.
    pub dac_lsb_u: [f32; 2],

    /// Current represented by one DAC count after the active converter width and shunt are applied.
    pub dac_lsb_i: [f32; 4],

    /// Volts represented by one ADC count for each of the two voltage ranges, indexed by `VoltageRange` after offset subtraction.
    pub adc_lsb_u: [f32; 2],

    /// Amperes represented by one ADC count for each of the four shunt ranges, indexed by `CurrentRange` after offset subtraction.
    pub adc_lsb_i: [f32; 4],

    /// Selects modify, which controls the exhaustive branch used by panel handling and output calculation.
    pub modify: Modify,

    /// Counts raw encoder edges toward one logical detent before applying an edit.
    pub inc_rast: i32,

    /// Persisted number of quadrature edges per logical panel encoder detent.
    pub init_inc_rast: f32,

    /// Per-voltage-range zero codes loaded from EEPROM before converting requested volts to raw DAC words.
    pub dac_u_offsets: [i32; 2],

    /// Per-shunt zero codes loaded from EEPROM before converting requested amperes to raw DAC words.
    pub dac_i_offsets: [i32; 4],

    /// Persisted raw zero-code correction for each of the two voltage ranges, subtracted before voltage scaling.
    pub adc_u_offsets: [i32; 2],

    /// Persisted raw zero-code correction for each of the four current shunts, subtracted before current scaling.
    pub adc_i_offsets: [i32; 4],

    /// Runtime copy of the 25-slot DCG EEPROM option image used by calibration accessors and protected serial writes.
    pub option_array: [f32; OptionSlot::COUNT],

    /// Per-voltage-range gain corrections applied when converting requested volts to DAC counts.
    pub dac_u_scales: [f32; 2],

    /// Per-shunt gain corrections applied when converting requested amperes to DAC counts.
    pub dac_i_scales: [f32; 4],

    /// Persisted gain correction for each voltage range, used to derive calibrated volts per ADC count.
    pub adc_u_scales: [f32; 2],

    /// Persisted gain correction for each current shunt, used to derive calibrated amperes per ADC count.
    pub adc_i_scales: [f32; 4],

    /// Number of parser failures accumulated for the ERC diagnostic response.
    pub err_count: i32,

    /// Stores the AVR UART divisor selected by the protected `SBD` command for reuse after reset.
    pub ee_ser_baud_reg: u8,

    /// Configured multidrop instrument address accepted before SQG/DCG parser command dispatch and emitted in `#channel:subchannel=` replies.
    pub slave_ch: u8,

    /// Address parsed from the current DCG frame before it is compared with the configured slave channel.
    pub current_ch: u8,

    /// Numeric protocol subchannel selected by mnemonic lookup or explicit `VAL` syntax for the current request/set operation.
    pub sub_ch: u8,

    /// Stores the decoded command identity while its subchannel and parameter are validated and dispatched.
    pub cmd_which: CmdWhich,

    /// Marks local panel ownership so a remote mutating command returns `BusyErr` instead of racing the encoder.
    pub busy_flag: bool,

    /// Authorizes exactly the protected EEPROM-setting path after a successful `WEN` command.
    pub ee_unlocked: bool,

    /// Requests one output/display refresh after a setpoint changes, coalescing multiple parser or panel updates.
    pub changed_flag: bool,

    /// Controls whether successful serial commands emit the legacy status prompt in addition to mandatory error replies.
    pub verbose: bool,

    /// Current shunt selected for ADC scaling and current-limit DAC conversion.
    pub i_range: u8,

    /// Caches the previous i range to suppress redundant writes and detect transitions that require safe blanking.
    pub old_i_range: u8,

    /// Automatically chosen current shunt retained separately from a range forced by command syntax.
    pub i_auto_range: u8,

    /// Voltage relay range selected for ADC scaling and voltage DAC conversion.
    pub u_range: u8,

    /// Caches the previous u range to suppress redundant writes and detect transitions that require safe blanking.
    pub old_u_range: u8,

    /// Full-scale voltage in volts derived from the EEPROM option image and used by range/limit checks.
    pub u_max: f32,

    /// Maximum current in amperes for the active shunt, used to clamp DCA and PCA results.
    pub i_max: f32,

    /// Full-scale amperage for each current shunt, indexed by current range during auto-ranging and limit checks.
    pub i_max_array: [f32; 4],

    /// Calibrated voltage threshold around which relay hysteresis changes the DCG voltage range.
    pub switchpoint: f32,

    /// Counts numeric token digits so fixed-width parsing can detect overflow and reproduce Pascal precision.
    pub digits: u8,

    /// Counts digits after the decimal separator to scale the parsed integer into engineering units.
    pub nachkomma: u8,

    /// Stores the parsed floating-point parameter used by engineering-unit setters.
    pub param: f32,

    /// Stores the parsed signed integer parameter used by indexed and timing subchannels.
    pub param_int: i32,

    /// Stores the checked byte-sized parameter used by option, waveform, and selector subchannels.
    pub param_byte: u8,

    /// Buffers param str so partial serial input and framed output remain independent of hardware receive timing.
    pub param_str: String,

    /// Buffers ser inp str so partial serial input and framed output remain independent of hardware receive timing.
    pub ser_inp_str: String,

    /// Buffers ser inp ptr so partial serial input and framed output remain independent of hardware receive timing.
    pub ser_inp_ptr: usize,

    /// Records whether limit enforcement corrected the last parsed value so the response reports `ParamErr`.
    pub check_limit_err: Error,

    /// Countdown keeping the parser-model activity LED asserted for the legacy visible interval.
    pub activity_timer: u8,

    /// Active-low activity LED shadow used by parser-only tests to verify command feedback timing.
    pub led_activity_low: bool,

    /// Counts parser-model display refresh requests caused by setpoint or status changes.
    pub display_refresh_count: u32,

    /// Buffers serial log so partial serial input and framed output remain independent of hardware receive timing.
    pub serial_log: Vec<String>,

    /// Records requested settling delays in the parser model so protected-write sequencing can be asserted without sleeping.
    pub delay_log: Vec<u16>,
}
impl Default for DcgParser {
    /// Creates the parser model with factory DCG limits, calibration arrays, clear error state, and a locked EEPROM-write latch.
    fn default() -> Self {
        Self {
            dc_volt: 0.0,
            dc_amp: 0.0,
            ah: 0.0,
            wh: 0.0,
            dc_volt_integrated: 0.0,
            dc_amp_integrated: 0.0,
            curr_amp: 0.0,
            curr_volt: 0.0,
            dc_volt_mod: 1.0,
            dc_amp_mod: 1.0,
            input_voltage: 0.0,
            measured_voltage: 0.0,
            measured_current: 0.0,
            temperature: 0.0,
            pw_on_time: DEFAULT_OPTION_ARRAY[OptionSlot::InitialRippleOnTime.index()] as i32,
            pw_off_time: DEFAULT_OPTION_ARRAY[OptionSlot::InitialRippleOffTime.index()] as i32,
            ripple_percent: 0,
            ripple_voltage: 0.0,
            no_toggle: true,
            adc_raw_u: 0,
            adc_raw_i: 0,
            adc10: [0; 6],
            dac_raw_uon: 0,
            dac_raw_uoff: 0,
            dac_raw_i: 0,
            dac_max: DEFAULT_DAC_MAX,
            dac_lsb_u: [1.0; 2],
            dac_lsb_i: [1.0; 4],
            adc_lsb_u: [1.0; 2],
            adc_lsb_i: [1.0; 4],
            modify: Modify::Ampere,
            inc_rast: 0,
            init_inc_rast: 0.0,
            dac_u_offsets: [0; 2],
            dac_i_offsets: [0; 4],
            adc_u_offsets: [0; 2],
            adc_i_offsets: [0; 4],
            option_array: DEFAULT_OPTION_ARRAY,
            dac_u_scales: [1.0; 2],
            dac_i_scales: [1.0; 4],
            adc_u_scales: [1.0; 2],
            adc_i_scales: [1.0; 4],
            err_count: 0,
            ee_ser_baud_reg: 0,
            slave_ch: 0,
            current_ch: 0,
            sub_ch: 0,
            cmd_which: CmdWhich::Err,
            busy_flag: false,
            ee_unlocked: false,
            changed_flag: false,
            verbose: false,
            i_range: 0,
            old_i_range: 0,
            i_auto_range: 0,
            u_range: 0,
            old_u_range: 0,
            u_max: DEFAULT_U_MAX,
            i_max: DEFAULT_I_MAX,
            i_max_array: DEFAULT_I_MAX_ARRAY,
            switchpoint: DEFAULT_SWITCHPOINT,
            digits: 0,
            nachkomma: 0,
            param: 0.0,
            param_int: 0,
            param_byte: 0,
            param_str: String::new(),
            ser_inp_str: String::new(),
            ser_inp_ptr: 0,
            check_limit_err: Error::NoErr,
            activity_timer: 0,
            led_activity_low: false,
            display_refresh_count: 0,
            serial_log: Vec::new(),
            delay_log: Vec::new(),
        }
    }
}
impl DcgParser {
    /// Converts the parser's integer milli-unit representation to engineering units before validation and display formatting.
    pub fn param_div_1000(&mut self) {
        self.param /= 1000.0;
    }

    /// Converts engineering units back to the integer milli-unit representation used by the legacy serial parameter path.
    pub fn param_mul_1000(&mut self) {
        self.param *= 1000.0;
    }

    // Device-specific parser branch.

    /// Formats the selected runtime or calibration value using the subchannel's protocol units and precision.
    pub fn parse_get_param(&mut self) {
        self.digits = 1;
        self.nachkomma = 4;

        match self.sub_ch {
            0 => {
                self.param = self.dc_volt;
                self.write_param_ser();
            }
            1 => {
                self.param = self.dc_amp;
                self.nachkomma = 7u8.saturating_sub(self.i_range);
                self.write_param_ser();
            }
            2 => {
                self.param = self.dc_amp;
                self.param_mul_1000();
                self.nachkomma = 2;
                self.write_param_ser();
            }
            3 => {
                self.param = self.dc_amp;
                self.param_mul_1000();
                self.param_mul_1000();
                self.nachkomma = 0;
                self.write_param_ser();
            }
            7 => {
                self.param = self.ah;
                self.write_param_ser();
            }
            8 => {
                self.param = self.wh;
                self.write_param_ser();
            }
            10 => {
                self.get_voltage();
                self.write_param_ser();
            }
            11 => {
                self.get_current();
                self.nachkomma = 8u8.saturating_sub(self.i_range);
                self.write_param_ser();
            }
            12 => {
                self.get_current();
                self.param_mul_1000();
                self.nachkomma = 2;
                self.write_param_ser();
            }
            13 => {
                self.get_current();
                self.param_mul_1000();
                self.param_mul_1000();
                self.nachkomma = 0;
                self.write_param_ser();
            }
            15 => {
                self.get_input_voltage();
                self.write_param_ser();
            }
            16 => {
                self.param = self.dc_volt_integrated;
                self.write_param_ser();
            }
            17 => {
                self.param = self.dc_amp_integrated;
                self.nachkomma = 8u8.saturating_sub(self.i_range);
                self.write_param_ser();
            }
            18 => {
                self.param = self.curr_amp * self.curr_volt;
                self.write_param_ser();
            }
            20 => {
                self.param = self.dc_volt_mod * 100.0;
                self.write_param_ser();
            }
            21..=23 => {
                self.param = self.dc_amp_mod * 100.0;
                self.write_param_ser();
            }
            27 => {
                self.param_int = self.pw_on_time;
                self.write_param_int_ser();
            }
            28 => {
                self.param_int = self.pw_off_time;
                self.write_param_int_ser();
            }
            29 => {
                self.param_int = self.ripple_percent;
                self.write_param_int_ser();
            }
            50 => {
                self.param_int = i32::from(self.adc_raw_u);
                self.write_param_int_ser();
            }
            51 => {
                self.param_int = i32::from(self.adc_raw_i);
                self.write_param_int_ser();
            }
            52 => {
                self.param_int = i32::from(self.get_adc10(3));
                self.write_param_int_ser();
            }
            53 => {
                self.param_int = i32::from(self.get_adc10(4));
                self.write_param_int_ser();
            }
            54 => {
                self.param_int = i32::from(self.get_adc10(5));
                self.write_param_int_ser();
            }
            70 => {
                self.param_int = i32::from(self.dac_raw_uon);
                self.write_param_int_ser();
            }
            71 => {
                self.param_int = i32::from(self.dac_raw_i);
                self.write_param_int_ser();
            }
            80 => {
                self.param_int = self.modify as i32;
                self.write_param_int_ser();
            }
            89 => {
                self.param_int = self.inc_rast;
                self.write_param_int_ser();
            }
            99 => {
                self.get_voltage();
                self.sub_ch = 10;
                self.write_param_ser();
                self.get_current();
                self.sub_ch = 11;
                self.write_param_ser();
                self.get_input_voltage();
                self.sub_ch = 15;
                self.write_param_ser();
            }
            100 | 101 => {
                self.param_int = self.dac_u_offsets[(self.sub_ch - 100) as usize];
                self.write_param_int_ser();
            }
            102..=105 => {
                self.param_int = self.dac_i_offsets[(self.sub_ch - 102) as usize];
                self.write_param_int_ser();
            }
            110 | 111 => {
                self.param_int = self.adc_u_offsets[(self.sub_ch - 110) as usize];
                self.write_param_int_ser();
            }
            112..=115 => {
                self.param_int = self.adc_i_offsets[(self.sub_ch - 112) as usize];
                self.write_param_int_ser();
            }
            150..=174 => {
                self.param = self.option_array[(self.sub_ch - 150) as usize];
                self.write_param_ser();
            }
            200 | 201 => {
                self.param = self.dac_u_scales[(self.sub_ch - 200) as usize];
                self.nachkomma = 5;
                self.write_param_ser();
            }
            202..=205 => {
                self.param = self.dac_i_scales[(self.sub_ch - 202) as usize];
                self.nachkomma = 5;
                self.write_param_ser();
            }
            210 | 211 => {
                self.param = self.adc_u_scales[(self.sub_ch - 210) as usize];
                self.nachkomma = 5;
                self.write_param_ser();
            }
            212..=215 => {
                self.param = self.adc_i_scales[(self.sub_ch - 212) as usize];
                self.nachkomma = 5;
                self.write_param_ser();
            }
            233 => {
                self.param = self.temperature;
                self.nachkomma = 1;
                self.write_param_ser();
            }
            251 => {
                self.param_int = self.err_count;
                self.write_param_int_ser();
            }
            252 => {
                self.param_int = i32::from(self.ee_ser_baud_reg);
                self.write_param_int_ser();
            }
            253 => {
                self.serial_log.push(self.ser_inp_str.clone());
            }
            254 => {
                self.write_framed_text_ser(VERS1_STR);
            }
            250 | 255 => {
                self.serprompt(Error::NoErr);
            }
            _ => {
                self.serprompt(Error::ParamErr);
            }
        }
    }

    /// Applies a parsed value to its owning setting, enforcing EEPROM unlock and recalculation side effects where required.
    pub fn parse_set_param(&mut self) {
        if self.busy_flag {
            // The Pascal parser rejects writes while a measurement/update cycle
            // is active so callers get a deterministic `BusyErr` instead of a
            // half-applied setting.
            self.serprompt(Error::BusyErr);
            return;
        }

        self.changed_flag = true;

        match self.sub_ch {
            0 => {
                self.dc_volt = self.param;
            }
            1 => {
                self.dc_amp = self.param;
            }
            2 => {
                self.param_div_1000();
                self.dc_amp = self.param;
            }
            3 => {
                self.param_div_1000();
                self.param_div_1000();
                self.dc_amp = self.param;
            }
            7 | 8 => {
                self.ah = 0.0;
                self.wh = 0.0;
            }
            20 => {
                self.dc_volt_mod = self.param / 100.0;
            }
            21..=23 => {
                self.dc_amp_mod = self.param / 100.0;
            }
            27 => {
                self.pw_on_time = self.param_int;
            }
            28 => {
                self.pw_off_time = self.param_int;
            }
            29 => {
                self.ripple_percent = self.param_int;
            }
            80 => {
                self.modify = Modify::from_byte(self.param_byte).unwrap_or(self.modify);
                self.werte_on_lcd();
            }
            89 => {
                if self.ee_unlocked {
                    self.inc_rast = self.param_int;
                    self.init_inc_rast = self.param;
                } else {
                    self.serprompt(Error::LockedErr);
                    return;
                }
            }
            100..=115 | 200..=215 => {
                if self.ee_unlocked {
                    match self.sub_ch {
                        100 | 101 => {
                            self.dac_u_offsets[(self.sub_ch - 100) as usize] = self.param_int;
                        }
                        102..=105 => {
                            self.dac_i_offsets[(self.sub_ch - 102) as usize] = self.param_int;
                        }
                        110 | 111 => {
                            self.adc_u_offsets[(self.sub_ch - 110) as usize] = self.param_int;
                        }
                        112..=115 => {
                            self.adc_i_offsets[(self.sub_ch - 112) as usize] = self.param_int;
                        }
                        200 | 201 => {
                            self.dac_u_scales[(self.sub_ch - 200) as usize] = self.param;
                        }
                        202..=205 => {
                            self.dac_i_scales[(self.sub_ch - 202) as usize] = self.param;
                        }
                        210 | 211 => {
                            self.adc_u_scales[(self.sub_ch - 210) as usize] = self.param;
                        }
                        212..=215 => {
                            self.adc_i_scales[(self.sub_ch - 212) as usize] = self.param;
                        }
                        _ => {}
                    }
                    self.init_scales();
                    self.mdelay(3);
                } else {
                    self.serprompt(Error::LockedErr);
                    return;
                }
            }
            150..=174 => {
                if self.ee_unlocked {
                    self.option_array[(self.sub_ch - 150) as usize] = self.param;
                    self.init_scales();
                    self.mdelay(3);
                } else {
                    self.serprompt(Error::LockedErr);
                    return;
                }
            }
            250 => {}
            251 => {
                self.err_count = self.param_int;
            }
            252 => {
                if self.ee_unlocked {
                    self.ee_ser_baud_reg = self.param_byte;
                } else {
                    self.serprompt(Error::LockedErr);
                    return;
                }
            }
            _ => {
                self.serprompt(Error::ParamErr);
                return;
            }
        }

        self.ee_unlocked = false;
        if self.sub_ch == 250 {
            self.ee_unlocked = true;
        }

        self.check_limits();
        if self.verbose || (self.check_limit_err != Error::NoErr) {
            self.serprompt(self.check_limit_err);
        }
        self.set_level_dac();
    }

    // General parser branch.

    /// Matches mnemonics case-insensitively against the ordered protocol table, returning `Err` rather than borrowing another command's index.
    pub fn cmd_to_index(&mut self) -> CmdWhich {
        CmdWhich::from_mnemonic(&self.param_str)
    }

    // Extract either a command token or a numeric parameter token from SerInpStr.
    // Returns true for parameter tokens, false for command tokens.

    /// Extracts one command or numeric token using the permissive character ranges accepted by the Pascal parser.
    pub fn parse_extract(&mut self) -> bool {
        self.param_str.clear();
        let bytes = self.ser_inp_str.as_bytes();

        // Ignore inter-token whitespace before deciding whether the next field
        // is a command mnemonic or a numeric / wildcard parameter.
        while self.ser_inp_ptr < bytes.len() && bytes[self.ser_inp_ptr] == b' ' {
            self.ser_inp_ptr += 1;
        }

        if self.ser_inp_ptr >= bytes.len() {
            return false;
        }

        // Parameter tokens may start with `*` for Omni addressing or with a
        // digit / sign / decimal marker for normal numeric sub-channels.
        let is_param = matches!(bytes[self.ser_inp_ptr], b'*'..=b'9');

        if is_param {
            while self.ser_inp_ptr < bytes.len() {
                let byte = bytes[self.ser_inp_ptr];
                if matches!(byte, b'*'..=b'9') {
                    self.param_str.push(byte as char);
                    self.ser_inp_ptr += 1;
                } else {
                    return true;
                }
            }
            true
        } else {
            while self.ser_inp_ptr < bytes.len() {
                let byte = bytes[self.ser_inp_ptr];
                if byte.is_ascii_alphabetic() {
                    self.param_str.push(byte as char);
                    self.ser_inp_ptr += 1;
                } else {
                    return false;
                }
            }
            false
        }
    }

    /// Parses one addressed or implicit-channel command and routes it through request/set handling while preserving echo and checksum semantics.
    pub fn parse_sub_ch(&mut self) {
        if self.ser_inp_str.is_empty() {
            self.serprompt(Error::NoErr);
            return;
        }

        // Accepted forms mirror the Pascal parser: `MainCh:CMD SubCh?`,
        // `MainCh:SubCh=Value`, and the short form without `VAL` and/or
        // without `MainCh:`, which then reuses the previously addressed channel.
        let has_main_ch = self.ser_inp_str.contains(':');
        let is_request = !self.ser_inp_str.contains('=');
        let first_char = self.ser_inp_str.as_bytes()[0] as char;
        let is_omni = first_char == '*';
        let is_result = first_char == '#';

        if is_result {
            self.write_ser_inp();
            return;
        }

        self.ser_inp_ptr = 0;
        if has_main_ch {
            let _is_param = self.parse_extract();
            self.ser_inp_ptr = self.ser_inp_ptr.saturating_add(1);
            if is_omni {
                // Omni commands are forwarded to the bus unchanged instead of
                // rebinding `current_ch` locally.
                self.write_ser_inp();
            } else if let Ok(channel) = self.param_str.parse::<u8>() {
                self.current_ch = channel;
            } else {
                self.serprompt(Error::ParamErr);
                return;
            }
        }

        if !is_omni && has_main_ch && self.current_ch != self.slave_ch {
            // Frames with an explicit foreign main channel are forwarded and
            // not interpreted locally.
            self.write_ser_inp();
            return;
        }

        // `?` or `!` requests a verbose reply in the original serial protocol.
        self.verbose = self.ser_inp_str.contains('!') || self.ser_inp_str.contains('?');

        if let Some(check_pos) = self.ser_inp_str.find('$') {
            // The protocol uses a trailing two-digit hex XOR checksum; the `$`
            // marker itself is not included in the XOR span.
            let checksum_slice = self.ser_inp_str.get(check_pos + 1..check_pos + 3);
            let Some(checksum_text) = checksum_slice else {
                self.serprompt(Error::ChecksumErr);
                return;
            };

            let Some(checksum_in) = Self::hex_to_int(checksum_text) else {
                self.serprompt(Error::ChecksumErr);
                return;
            };

            let mut checksum = 0u8;
            for byte in self.ser_inp_str.as_bytes().iter().take(check_pos) {
                checksum ^= *byte;
            }

            if checksum != checksum_in {
                self.serprompt(Error::ChecksumErr);
                return;
            }
        }

        self.activity_timer = 255;
        self.led_activity_low = true;

        let sub_ch_offset = if self.parse_extract() {
            // Direct sub-channel form: the extracted token already is the
            // absolute sub-channel number.
            0
        } else {
            self.cmd_which = self.cmd_to_index();
            if self.cmd_which == CmdWhich::Err {
                self.serprompt(Error::SyntaxErr);
                return;
            }

            // Mnemonic commands contribute the block base; the next extracted
            // token adds the per-command sub-channel offset.
            let offset = self.cmd_which.default_subchannel();
            let _is_param = self.parse_extract();
            offset
        };

        let sub_ch_base = self.param_str.parse::<u16>().unwrap_or(0);
        self.sub_ch = sub_ch_base.saturating_add(u16::from(sub_ch_offset)) as u8;

        if is_request {
            self.parse_get_param();
        } else {
            // Set commands require an explicit `=` followed by a parseable
            // numeric payload.
            let Some(equal_pos) = self.ser_inp_str.find('=') else {
                self.serprompt(Error::ParamErr);
                return;
            };

            self.ser_inp_ptr = equal_pos + 1;
            if self.ser_inp_ptr < self.ser_inp_str.len() && self.parse_extract() {
                if let Ok(value) = self.param_str.parse::<f32>() {
                    self.param = value;
                    self.param_int = value as i32;
                    self.param_byte = self.param_int as u8;
                } else {
                    self.serprompt(Error::ParamErr);
                    return;
                }
            } else {
                self.serprompt(Error::ParamErr);
                return;
            }
            self.parse_set_param();
        }
    }

    /// Formats the current floating-point parameter, writes the addressed channel/subchannel prefix, then appends CR/LF.
    pub(super) fn write_param_ser(&mut self) {
        self.serial_log.push(format!(
            "{}:{}={:.*}",
            self.current_ch, self.sub_ch, self.nachkomma as usize, self.param
        ));
    }

    /// Writes the addressed channel/subchannel prefix, a base-10 signed integer parameter, and CR/LF.
    pub(super) fn write_param_int_ser(&mut self) {
        self.serial_log.push(format!(
            "{}:{}={}",
            self.current_ch, self.sub_ch, self.param_int
        ));
    }

    /// Writes the current addressed reply prefix followed by supplied protocol text and CR/LF.
    pub(super) fn write_framed_text_ser(&mut self, text: &str) {
        self.serial_log
            .push(format!("#{}:{}={}", self.slave_ch, self.sub_ch, text));
    }

    /// Emits the DCG parser status prompt when verbosity or an error requires it and updates error accounting.
    pub(super) fn serprompt(&mut self, error: Error) {
        self.serial_log.push(error.as_str().to_owned());
    }

    /// Echoes the stored or supplied serial input text verbatim, then terminates the echo with the legacy CR/LF pair.
    pub(super) fn write_ser_inp(&mut self) {
        self.serial_log.push(self.ser_inp_str.clone());
    }

    /// Selects the configured voltage converter path, applies offset and per-range scale, and stores the engineering-unit result for power/protection.
    pub(super) fn get_voltage(&mut self) {
        self.param = self.measured_voltage;
    }

    /// Selects the configured current converter path, applies shunt-specific offset and scale, then updates measured power from the paired voltage.
    pub(super) fn get_current(&mut self) {
        self.param = self.measured_current;
    }

    /// Converts the auxiliary ADC10 supply reading through the board divider so relay and fuse checks compare physical volts.
    pub(super) fn get_input_voltage(&mut self) {
        self.param = self.input_voltage;
    }

    /// Selects the one-based AVR ADC channel, waits for mux settling, starts conversion, polls completion, then combines ADCL before ADCH as required by the AVR latch rule.
    pub(super) fn get_adc10(&self, channel: usize) -> u16 {
        self.adc10.get(channel).copied().unwrap_or(0)
    }

    /// Renders werte in its fixed panel position so updates do not disturb the other row.
    pub(super) fn werte_on_lcd(&mut self) {
        self.display_refresh_count = self.display_refresh_count.saturating_add(1);
    }

    /// Rebuilds calibration factors from EEPROM and active hardware options so later ADC/DAC conversions use one coherent scale set.
    pub(super) fn init_scales(&mut self) {
        let init_gain_pre = self.option_array[OptionSlot::PreamplifierGain.index()];
        let init_gain_out = self.option_array[OptionSlot::OutputStageGain.index()];
        let init_gain_i = self.option_array[OptionSlot::CurrentMeasurementGain.index()];
        let u_ref = self.option_array[OptionSlot::ReferenceVoltage.index()];
        let init_options = self.option_array[OptionSlot::InstalledHardware.index()] as u8;
        let dac_16_present = crate::dcg::HardwareOption::Ltc1655Dac.is_set_in(init_options);
        let adc_16_present = crate::dcg::HardwareOption::Ltc1864Adc.is_set_in(init_options);
        let dac_max_exclusive_u32 = if dac_16_present { 65536 } else { 4096 };
        let dac_max_exclusive = dac_max_exclusive_u32 as f32;
        let adc_max_exclusive = if adc_16_present { 65536.0 } else { 1024.0 };
        let u_fac = if dac_16_present { 2.0 * u_ref } else { u_ref };

        self.dac_lsb_u[0] = u_fac * init_gain_out / (dac_max_exclusive * self.dac_u_scales[0]);
        self.dac_lsb_u[1] =
            u_fac * init_gain_pre * init_gain_out / (dac_max_exclusive * self.dac_u_scales[1]);

        self.adc_lsb_u[0] = self.option_array[OptionSlot::LowVoltageAdcDivider.index()]
            * self.adc_u_scales[0]
            * u_ref
            * init_gain_out
            / adc_max_exclusive;
        self.adc_lsb_u[1] = self.option_array[OptionSlot::HighVoltageAdcDivider.index()]
            * self.adc_u_scales[1]
            * u_ref
            * init_gain_out
            / adc_max_exclusive;

        let current_u_fac = u_fac * init_gain_i;
        for range in 0..4 {
            let r_sense =
                self.option_array[OptionSlot::SenseResistance2mA.index() + range];
            self.dac_lsb_i[range] =
                (current_u_fac / r_sense) / (dac_max_exclusive * self.dac_i_scales[range]);
            self.adc_lsb_i[range] =
                (self.adc_i_scales[range] * u_ref / (2.0 * r_sense)) / adc_max_exclusive;
        }

        self.dac_max = (dac_max_exclusive_u32 - 1) as u16;
        self.u_max = self.option_array[OptionSlot::MaximumVoltage.index()];
        self.i_max_array = [
            self.option_array[OptionSlot::MaximumCurrent2mA.index()],
            self.option_array[OptionSlot::MaximumCurrent20mA.index()],
            self.option_array[OptionSlot::MaximumCurrent200mA.index()],
            self.option_array[OptionSlot::MaximumCurrent2A.index()],
        ];
        self.i_max = self.i_max_array[3];
        self.switchpoint = self.option_array[OptionSlot::VoltageRangeSwitchpoint.index()];
        self.dc_volt_mod = 1.0;
        self.ripple_percent = self.option_array[OptionSlot::InitialRipplePercent.index()] as i32;
        self.pw_on_time = self.option_array[OptionSlot::InitialRippleOnTime.index()] as i32;
        self.pw_off_time = self.option_array[OptionSlot::InitialRippleOffTime.index()] as i32;
    }

    /// Records the requested post-write settling interval through the hardware seam used by the parser model.
    pub(super) fn mdelay(&mut self, milliseconds: u16) {
        self.delay_log.push(milliseconds);
    }

    /// Normalizes unsafe or unrepresentable settings before they reach DAC or relay calculations, returning an error when the requested value had to be corrected.
    pub(super) fn check_limits(&mut self) {
        self.check_limit_err = Error::NoErr;

        if self.dc_volt < 0.0 {
            self.dc_volt = 0.0;
            self.check_limit_err = Error::ParamErr;
        }

        if self.dc_volt > self.u_max {
            self.dc_volt = self.u_max;
            self.check_limit_err = Error::ParamErr;
        }

        if self.dc_amp < 0.0 {
            self.dc_amp = 0.0;
            self.check_limit_err = Error::ParamErr;
        }

        if self.dc_amp > self.i_max {
            self.dc_amp = self.i_max;
            self.check_limit_err = Error::ParamErr;
        }

        if self.pw_on_time < 2 {
            self.pw_on_time = 2;
            self.check_limit_err = Error::ParamErr;
        }

        if self.pw_off_time < 0 {
            self.pw_off_time = 0;
            self.check_limit_err = Error::ParamErr;
        }

        if self.ripple_percent < 0 {
            self.ripple_percent = 0;
            self.check_limit_err = Error::ParamErr;
        }

        if self.ripple_percent > 100 {
            self.ripple_percent = 100;
            self.check_limit_err = Error::ParamErr;
        }

        self.no_toggle = self.ripple_percent == 0;
        self.ripple_voltage = if self.no_toggle {
            0.0
        } else {
            self.ripple_percent as f32 * self.dc_volt / 100.0
        };
    }

    /// Computes calibrated on/off DAC words and blanks output during range changes to avoid delivering a transient pulse.
    pub(super) fn set_level_dac(&mut self) {
        self.calc_range_i();

        if self.i_range != self.old_i_range {
            self.dac_raw_i = 0;
            self.mdelay(4);
        }
        self.old_i_range = self.i_range;
        self.i_auto_range = self.i_range;

        let i_range = self.i_range as usize;
        let current_lsb = self.dac_lsb_i[i_range];
        let current_dac = if current_lsb > 0.0 {
            (self.dc_amp * self.dc_amp_mod / current_lsb) + 0.5 + self.dac_i_offsets[i_range] as f32
        } else {
            0.0
        };
        self.dac_raw_i = self.clamp_dac(current_dac);

        self.u_range = u8::from(self.dc_volt > self.switchpoint);
        if self.u_range != self.old_u_range {
            self.dc_volt_mod = 1.0;
            self.dac_raw_uon = 0;
            self.dac_raw_uoff = 0;
            self.mdelay(4);
        }
        self.old_u_range = self.u_range;

        let u_range = self.u_range as usize;
        let voltage_lsb = self.dac_lsb_u[u_range];
        let voltage_dac = if voltage_lsb > 0.0 {
            (self.dc_volt * self.dc_volt_mod / voltage_lsb)
                + 0.5
                + self.dac_u_offsets[u_range] as f32
        } else {
            0.0
        };
        self.dac_raw_uon = self.clamp_dac(voltage_dac);
        self.dac_raw_uoff = if self.pw_off_time > 0 && self.ripple_percent > 0 {
            let reduced = u32::from(self.dac_raw_uon) * (100u32 - self.ripple_percent as u32) / 100;
            reduced.min(u32::from(self.dac_max)) as u16
        } else {
            self.dac_raw_uon
        };
    }

    /// Derives range i from calibrated limits instead of hard-coding a board range.
    pub(super) fn calc_range_i(&mut self) {
        self.i_range = 0;
        for (range, max_current) in self.i_max_array.iter().enumerate() {
            if self.dc_amp > *max_current {
                self.i_range = (range as u8).saturating_add(1).min(3);
            }
        }
    }

    /// Clamps a computed raw code to the selected DAC's valid range so calibration cannot wrap the hardware word.
    pub(super) fn clamp_dac(&self, value: f32) -> u16 {
        value.round().clamp(0.0, self.dac_max as f32) as u16
    }

    /// Decodes checksum nybbles explicitly because the wire checksum is hexadecimal even though parameters are decimal.
    pub(super) fn hex_to_int(text: &str) -> Option<u8> {
        u8::from_str_radix(text, 16).ok()
    }
}
