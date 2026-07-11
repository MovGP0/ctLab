use super::*;

#[derive(Debug, Clone)]
/// Source-faithful serial command state used to validate parser behavior independently of hardware.
pub struct EdlParser {
    /// Complete serial line being decoded; command lookup and checksum operate on this exact text.
    pub ser_inp_str: String,

    /// Byte cursor advanced by token extraction so later parser stages resume at the correct delimiter.
    pub ser_inp_ptr: usize,

    /// Raw right-hand-side token retained for numeric conversion, echo, and text commands.
    pub param_str: String,

    /// Transport-neutral replies accumulated in the same order the Pascal serial writes occurred.
    pub output_lines: Vec<String>,

    /// Minimum integer width selected by each getter before formatting.
    pub digits: u8,

    /// Decimal-place count selected from command, unit, and shunt resolution.
    pub nachkomma: u8,

    /// Controller channel used when a frame omits an explicit address.
    pub current_ch: i32,

    /// Address parsed from the current frame and reused in response prefixes.
    pub slave_ch: i32,

    /// Concrete operation after mnemonic offset and argument resolution.
    pub sub_ch: i32,

    /// Floating-point view of the active parameter or getter result.
    pub param: f64,

    /// Signed-integer view used by timing, masks, options, and raw calibration commands.
    pub param_int: i32,

    /// Range-checked byte view used by small EEPROM and hardware selections.
    pub param_byte: u8,

    /// Requests explicit success/error prompts rather than silent setter completion.
    pub verbose: bool,

    /// Rejects commands that would collide with timing-sensitive firmware work.
    pub busy_flag: bool,

    /// Signals that a setter changed display-visible state and requires an LCD refresh.
    pub changed_flag: bool,

    /// Specific limit result reported after a setter is clamped.
    pub check_limit_err: PromptCode,

    /// Requested load state reflected by the ENA protocol subchannel.
    pub output_enable: bool,

    /// Current multiplexer enable derived from mode and output state.
    pub mpxena: bool,

    /// Voltage-range multiplexer selection associated with the active mode.
    pub mode_mpx: bool,

    /// Latched low-input condition exposed through status commands.
    pub low_volt: bool,

    /// Disables on/off ripple phase alternation for continuous operation.
    pub no_toggle: bool,

    /// Current shunt used for live ADC and DAC scale lookup.
    pub shunt_select: u8,

    /// Previously applied shunt used to avoid redundant relay operations.
    pub old_shunt_select: u8,

    /// Manual shunt request or automatic-range sentinel.
    pub shunt_range: u8,

    /// Regulation law and voltage range encoded by protocol mode numbers.
    pub mode_select: Mode,

    /// Front-panel menu target returned and set through display subchannels.
    pub modify: Modify,

    /// Constant-current setpoint in amperes.
    pub dc_amp: f64,

    /// Constant-power setpoint in watts.
    pub dc_watt: f64,

    /// Low-voltage cutoff setpoint in volts.
    pub dc_volt: f64,

    /// Constant-resistance setpoint in ohms.
    pub dc_ohm: f64,

    /// Ripple/off-phase current multiplier derived from percentage settings.
    pub dc_amp_mod: f64,

    /// Integrated charge reported by MAH after conversion to milliamp-hours.
    pub ah: f64,

    /// Integrated energy reported by MWH after conversion to milliwatt-hours.
    pub wh: f64,

    /// Duty-cycle-weighted measured power used by the live-value getter.
    pub ptot: f64,

    /// Scaled voltage captured during the active ripple phase.
    pub voltage_on: f64,

    /// Scaled current captured during the active ripple phase.
    pub current_on: f64,

    /// Scaled voltage captured during the inactive ripple phase.
    pub voltage_off: f64,

    /// Scaled current captured during the inactive ripple phase.
    pub current_off: f64,

    /// Configured active ripple duration.
    pub pw_on_time: i32,

    /// Configured inactive ripple duration.
    pub pw_off_time: i32,

    /// Active current percentage used to derive ripple amplitude.
    pub i_percent: i32,

    /// Raw 16-bit active-phase voltage sample exposed for diagnostics.
    pub ad16_temp_u_on: u16,

    /// Raw 16-bit active-phase current sample exposed for diagnostics.
    pub ad16_temp_i_on: u16,

    /// Quantized DAC code for the active phase.
    pub dac_temp_on: u16,

    /// Quantized DAC code for the inactive phase.
    pub dac_temp_off: u16,

    /// DAC code currently selected for hardware output.
    pub dac_temp: u16,

    /// Raw AVR ADC channels backing RAW diagnostic subchannels.
    pub adc10: [u16; ADC10_COUNT],

    /// Per-shunt current-DAC zero corrections editable only while EEPROM is unlocked.
    pub daci_offsets: [i32; DACI_COUNT],

    /// Low/high voltage ADC zero corrections.
    pub adcu_offsets: [i32; ADCU_COUNT],

    /// Per-shunt current ADC zero corrections.
    pub adci_offsets: [i32; ADCI_COUNT],

    /// Indexed hardware and boot options retained in Pascal EEPROM order.
    pub option_array: [f64; OPTION_ARRAY_LEN],

    /// Per-shunt current-DAC gain calibration factors.
    pub daci_scales: [f64; DACI_COUNT],

    /// Derived amperes represented by one DAC code for each shunt.
    pub dac_lsb_i: [f64; DACI_COUNT],

    /// Derived resistance-mode numerator represented by one DAC code for each shunt.
    pub dac_lsb_r: [f64; DACI_COUNT],

    /// Maximum raw code selected from the installed DAC type.
    pub dac_max: u16,

    /// Calibrated lower resistance clamp preventing excessive current.
    pub dc_ohm_min: f64,

    /// Calibrated upper resistance clamp retaining useful DAC resolution.
    pub dc_ohm_max: f64,

    /// Active high/low voltage divider ratio.
    pub divider_u: f64,

    /// Low-range voltage ADC calibration multiplier.
    pub adc16_u_scale_low: f64,

    /// High-range voltage ADC calibration multiplier.
    pub adc16_u_scale_high: f64,

    /// Per-shunt amperes represented by one 16-bit ADC code.
    pub adc16_lsb_i: [f64; ADCI_COUNT],

    /// Per-shunt amperes represented by one AVR ADC code.
    pub adc10_lsb_i: [f64; ADCI_COUNT],

    /// Per-shunt current ADC gain calibration factors.
    pub adci_scales: [f64; ADCI_COUNT],

    /// Internal LM75 reading returned by temperature subchannels.
    pub temperature: f64,

    /// External LM75 reading retained separately for board diagnostics.
    pub temperature_extern: f64,

    /// Live trigger and temperature-device option bits.
    pub trig_mask: u8,

    /// EEPROM mirror updated by trigger option setters.
    pub ee_trig_mask: u8,

    /// Protocol error count returned and cleared through ERC.
    pub err_count: i32,

    /// EEPROM UART divisor selected by the SBD command.
    pub ee_ser_baud_reg: u8,

    /// Authorization latch guarding calibration and option writes.
    pub ee_unlocked: bool,

    /// Live encoder detent/raster setting used by front-panel movement.
    pub inc_rast: i32,

    /// EEPROM-backed encoder raster restored on startup.
    pub init_inc_rast: f64,

    /// Identification payload returned by IDN.
    pub vers1_str: &'static str,

    /// Countdown keeping the activity LED asserted after valid traffic.
    pub activity_timer: u8,

    /// Current activity LED state exposed to the runtime adapter.
    pub led_activity: bool,

    /// Observable count of parser-triggered display refreshes.
    pub display_refresh_count: u32,

}

impl Default for EdlParser {
    /// Constructs parser state matching the Pascal globals before EEPROM and live hardware values are loaded.
    fn default() -> Self {
        Self {
            ser_inp_str: String::new(),
            ser_inp_ptr: 0,
            param_str: String::new(),
            output_lines: Vec::new(),
            digits: 1,
            nachkomma: 4,
            current_ch: 0,
            slave_ch: 0,
            sub_ch: 0,
            param: 0.0,
            param_int: 0,
            param_byte: 0,
            verbose: false,
            busy_flag: false,
            changed_flag: false,
            check_limit_err: PromptCode::NoErr,
            output_enable: false,
            mpxena: false,
            mode_mpx: false,
            low_volt: false,
            no_toggle: false,
            shunt_select: 0,
            old_shunt_select: 0,
            shunt_range: 0,
            mode_select: Mode::OutputOff,
            modify: Modify::LowerMainMenu,
            dc_amp: 0.0,
            dc_watt: 0.0,
            dc_volt: 0.0,
            dc_ohm: 0.0,
            dc_amp_mod: 1.0,
            ah: 0.0,
            wh: 0.0,
            ptot: 0.0,
            voltage_on: 0.0,
            current_on: 0.0,
            voltage_off: 0.0,
            current_off: 0.0,
            pw_on_time: 10,
            pw_off_time: 0,
            i_percent: 0,
            ad16_temp_u_on: 0,
            ad16_temp_i_on: 0,
            dac_temp_on: 0,
            dac_temp_off: 0,
            dac_temp: 0,
            adc10: [0; ADC10_COUNT],
            daci_offsets: [0; DACI_COUNT],
            adcu_offsets: [0; ADCU_COUNT],
            adci_offsets: [0; ADCI_COUNT],
            option_array: DEFAULT_OPTION_ARRAY,
            daci_scales: [1.0; DACI_COUNT],
            dac_lsb_i: [1.0; DACI_COUNT],
            dac_lsb_r: [1.0; DACI_COUNT],
            dac_max: DEFAULT_DAC_MAX,
            dc_ohm_min: DEFAULT_OPTION_ARRAY[OPT_RSENSE_BASE + 3]
                * DEFAULT_OPTION_ARRAY[OPT_GAIN_I]
                * 1.1,
            dc_ohm_max: DEFAULT_OPTION_ARRAY[OPT_RSENSE_BASE]
                * DEFAULT_OPTION_ARRAY[OPT_GAIN_I]
                * 100.0,
            divider_u: 1.0,
            adc16_u_scale_low: 1.0,
            adc16_u_scale_high: 1.0,
            adc16_lsb_i: [0.0; ADCI_COUNT],
            adc10_lsb_i: [0.0; ADCI_COUNT],
            adci_scales: [1.0; ADCI_COUNT],
            temperature: 0.0,
            temperature_extern: 0.0,
            trig_mask: 0,
            ee_trig_mask: 0,
            err_count: 0,
            ee_ser_baud_reg: 0,
            ee_unlocked: false,
            inc_rast: 0,
            init_inc_rast: 0.0,
            vers1_str: "",
            activity_timer: 0,
            led_activity: false,
            display_refresh_count: 0,
        }
    }
}

impl EdlParser {
    /// Maps the resolved subchannel to a live value and formats it with the Pascal precision rules.
    pub fn parse_get_param(&mut self) {
        self.digits = 1;
        self.nachkomma = 4;

        match self.sub_ch {
            0 => {
                self.param = if self.output_enable { 1.0 } else { 0.0 };
                self.write_param_ser();
            }
            1 => {
                self.param = self.dc_amp;
                self.nachkomma = 7_u8.saturating_sub(self.shunt_select);
                self.write_param_ser();
            }
            2 => {
                self.param = self.dc_amp;
                self.param_mul_1000();
                self.nachkomma = 2;
                self.write_param_ser();
            }
            3 => {
                // mcb extension: expose the computed power value directly in watts.
                self.param = self.dc_watt;
                self.nachkomma = 2;
                self.write_param_ser();
            }
            4 => {
                // mcb extension: report the configured low-voltage cutoff threshold.
                self.param = self.dc_volt;
                self.nachkomma = 2;
                self.write_param_ser();
            }
            5 => {
                self.param = self.dc_ohm;
                self.nachkomma = 1_u8.saturating_add(self.shunt_select);
                self.write_param_ser();
            }
            7 => {
                // mcb extension: accumulated discharge capacity in Ah.
                self.param = self.ah;
                self.write_param_ser();
            }
            8 => {
                // mcb extension: accumulated discharge energy in Wh.
                self.param = self.wh;
                self.write_param_ser();
            }
            9 => {
                self.param_int = i32::from(self.shunt_select);
                self.write_param_int_ser();
            }
            10 => {
                self.get_voltage(true);
                self.write_param_ser();
            }
            11 => {
                self.get_current(true);
                self.nachkomma = 8_u8.saturating_sub(self.shunt_select);
                self.write_param_ser();
            }
            12 => {
                self.get_current(true);
                self.param_mul_1000();
                self.nachkomma = 2;
                self.write_param_ser();
            }
            15 => {
                self.get_voltage(false);
                self.write_param_ser();
            }
            16 => {
                self.get_current(false);
                self.nachkomma = 8_u8.saturating_sub(self.shunt_select);
                self.write_param_ser();
            }
            17 => {
                self.get_current(false);
                self.param_mul_1000();
                self.nachkomma = 2;
                self.write_param_ser();
            }
            18 => {
                self.param = self.ptot;
                self.write_param_ser();
            }
            19 => {
                self.param_int = self.mode_to_i32();
                self.write_param_int_ser();
            }
            21 | 22 => {
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
                self.param_int = self.i_percent;
                self.write_param_int_ser();
            }
            50 => {
                self.disable_ints();
                self.param_int = i32::from(self.ad16_temp_u_on);
                self.enable_ints();
                self.write_param_int_ser();
            }
            51 => {
                self.disable_ints();
                self.param_int = i32::from(self.ad16_temp_i_on);
                self.enable_ints();
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
            70 => {
                self.param_int = i32::from(self.dac_temp_on);
                self.write_param_int_ser();
            }
            71 => {
                self.param_int = i32::from(self.dac_temp_off);
                self.write_param_int_ser();
            }
            72 => {
                self.param_int = i32::from(self.dac_temp);
                self.write_param_int_ser();
            }
            80 => {
                self.param_int = self.modify_to_i32();
                self.write_param_int_ser();
            }
            89 => {
                self.param_int = self.inc_rast;
                self.write_param_int_ser();
            }
            99 => {
                // ALL request: return the four live measurement channels as a burst.
                self.get_voltage(true);
                self.sub_ch = 10;
                self.write_param_ser();
                self.get_current(true);
                self.sub_ch = 11;
                self.write_param_ser();
                self.get_voltage(false);
                self.sub_ch = 15;
                self.write_param_ser();
                self.get_current(false);
                self.sub_ch = 16;
                self.write_param_ser();
            }
            100 | 101 => {
                self.param_int = 0;
                self.write_param_int_ser();
            }
            102..=105 => {
                self.param_int = self.daci_offsets[(self.sub_ch - 102) as usize];
                self.write_param_int_ser();
            }
            110..=111 => {
                self.param_int = self.adcu_offsets[(self.sub_ch - 110) as usize];
                self.write_param_int_ser();
            }
            112..=115 => {
                self.param_int = self.adci_offsets[(self.sub_ch - 112) as usize];
                self.write_param_int_ser();
            }
            150..=171 => {
                self.param = self.option_array[(self.sub_ch - 150) as usize];
                self.write_param_ser();
            }
            200 | 201 => {
                self.param = 0.0;
                self.write_param_ser();
            }
            202..=205 => {
                self.param = self.daci_scales[(self.sub_ch - 202) as usize];
                self.nachkomma = 5;
                self.write_param_ser();
            }
            210 => {
                self.param = self.adc16_u_scale_low;
                self.nachkomma = 5;
                self.write_param_ser();
            }
            211 => {
                self.param = self.adc16_u_scale_high;
                self.nachkomma = 5;
                self.write_param_ser();
            }
            212..=215 => {
                self.param = self.adci_scales[(self.sub_ch - 212) as usize];
                self.nachkomma = 5;
                self.write_param_ser();
            }
            233 => {
                self.param = self.temperature;
                self.nachkomma = 1;
                self.write_param_ser();
            }
            234 => {
                self.param = self.temperature_extern;
                self.nachkomma = 1;
                self.write_param_ser();
            }
            240 => {
                self.param_int = i32::from(self.trig_mask);
                self.write_param_int_ser();
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
                // Serial test hook: echo the raw input line back unchanged.
                self.output_lines.push(self.ser_inp_str.clone());
            }
            254 => {
                // Version/value request uses the normal channel prefix before the banner string.
                let prefix = self.write_ch_prefix();
                self.output_lines
                    .push(format!("{prefix}{}", self.vers1_str));
            }
            250 | 255 => {
                self.serprompt(PromptCode::NoErr);
            }
            _ => {
                self.serprompt(PromptCode::ParamErr);
            }
        }
    }

    /// Validates and applies setters, including calibration locks, limits, DAC refresh, and EEPROM mirrors.
    pub fn parse_set_param(&mut self) {
        if self.busy_flag {
            // Writes are rejected while the load is in a protected busy phase.
            self.serprompt(PromptCode::BusyErr);
            return;
        }

        self.changed_flag = true;

        match self.sub_ch {
            0 => {
                self.output_enable = self.param != 0.0;
                if self.mode_select == Mode::OutputOff {
                    self.mpxena = false;
                } else {
                    self.mpxena = self.output_enable;
                }
            }
            1 => {
                self.dc_amp = self.param;
            }
            2 => {
                self.param_div_1000();
                self.dc_amp = self.param;
            }
            3 => {
                // mcb extension: set the constant-power target in watts.
                self.dc_watt = self.param;
            }
            4 => {
                // mcb extension: arming a low-voltage threshold also forces the output on.
                self.low_volt = false;
                self.output_enable = true;
                self.dc_volt = self.param;
            }
            5 => {
                self.dc_ohm = self.param;
            }
            7 | 8 => {
                // mcb extension: writing either counter clears both accumulated totals.
                self.ah = 0.0;
                self.wh = 0.0;
            }
            9 => {
                // 4..255 selects autoranging in the original firmware.
                self.shunt_range = self.param_int as u8;
            }
            19 => {
                // Changing mode can force an immediate shutdown; enabling happens later in SetDAC.
                self.mode_select = Mode::from(self.param_byte);
                if self.mode_select == Mode::OutputOff {
                    self.mpxena = false;
                    self.output_enable = false;
                    self.set_level_dac_i();
                } else {
                    self.output_enable = true;
                }
            }
            21 | 22 => {
                self.dc_amp_mod = self.param / 100.0;
            }
            27 => {
                self.pw_on_time = self.param_int;
            }
            28 => {
                self.pw_off_time = self.param_int;
            }
            29 => {
                self.i_percent = self.param_int;
            }
            70 => {
                self.disable_ints();
                self.dac_temp_on = self.param_int as u16;
                self.enable_ints();
                if self.verbose {
                    self.serprompt(PromptCode::NoErr);
                }
                // Raw DAC debug writes must not trigger any additional output switching.
                return;
            }
            71 => {
                self.disable_ints();
                self.dac_temp_off = self.param_int as u16;
                self.enable_ints();
                if self.verbose {
                    self.serprompt(PromptCode::NoErr);
                }
                // Raw DAC debug writes must not trigger any additional output switching.
                return;
            }
            80 => {
                self.modify = Modify::from(self.param_byte);
                self.werte_on_lcd();
            }
            89 | 100..=115 | 200..=223 => {
                if !self.ee_unlocked {
                    // Calibration and EEPROM-backed parameters stay locked until sub-channel 250.
                    self.serprompt(PromptCode::LockedErr);
                    return;
                }

                match self.sub_ch {
                    89 => {
                        self.init_inc_rast = self.param;
                        self.inc_rast = self.param_int;
                    }
                    100 | 101 => {}
                    102..=105 => {
                        self.daci_offsets[(self.sub_ch - 102) as usize] = self.param_int;
                    }
                    110..=111 => {
                        self.adcu_offsets[(self.sub_ch - 110) as usize] = self.param_int;
                    }
                    112..=115 => {
                        self.adci_offsets[(self.sub_ch - 112) as usize] = self.param_int;
                    }
                    200 | 201 => {}
                    202..=205 => {
                        self.daci_scales[(self.sub_ch - 202) as usize] = self.param;
                    }
                    210 => {
                        self.adc16_u_scale_low = self.param;
                    }
                    211 => {
                        self.adc16_u_scale_high = self.param;
                    }
                    212..=215 => {
                        self.adci_scales[(self.sub_ch - 212) as usize] = self.param;
                    }
                    _ => {}
                }

                // Mirror the firmware's short settle time after recalculating scales.
                self.init_scales();
                self.mdelay(3);
            }
            150..=171 => {
                if !self.ee_unlocked {
                    self.serprompt(PromptCode::LockedErr);
                    return;
                }

                self.option_array[(self.sub_ch - 150) as usize] = self.param;
                self.init_scales();
                self.mdelay(3);
            }
            240 => {
                self.trig_mask = self.param_int as u8;
                self.ee_trig_mask = self.trig_mask;
            }
            250 => {}
            251 => {
                self.err_count = self.param_int;
            }
            252 => {
                if !self.ee_unlocked {
                    self.serprompt(PromptCode::LockedErr);
                    return;
                }
                // Baud-rate EEPROM changes only take effect after the next reset.
                self.ee_ser_baud_reg = self.param_byte;
            }
            _ => {
                self.serprompt(PromptCode::ParamErr);
                return;
            }
        }

        self.ee_unlocked = self.sub_ch == 250;
        self.check_limits();

        if self.verbose {
            // CheckLimitErr is a Pascal variable holding NoErr or ParamErr after clamping.
            self.serprompt(self.check_limit_err);
        }

        // mcb modes select which control loop has to refresh the DAC target after a write.
        match self.mode_select {
            Mode::RhiVolt | Mode::RloVolt => self.set_level_dac_r(),
            Mode::IhiVolt | Mode::IloVolt => self.set_level_dac_i(),
            Mode::PhiVolt | Mode::PloVolt => self.set_level_dac_p(),
            Mode::OutputOff | Mode::Unknown(_) => {}
        }
    }

    /// Resolves the mnemonic through the compiler-checked command enum without allocation.
    pub fn cmd_to_index(&mut self) -> CmdWhich {
        CmdWhich::from_str(&self.param_str)
    }

    /// Consumes one numeric or quoted token from the serial cursor and records whether conversion succeeded.
    pub fn parse_extract(&mut self) -> bool {
        self.param_str.clear();

        let bytes = self.ser_inp_str.as_bytes().to_vec();
        let mut ptr = self.ser_inp_ptr.min(bytes.len());

        while ptr < bytes.len() && bytes[ptr] == b' ' {
            // The Pascal parser explicitly skips leading blanks before every token.
            ptr += 1;
        }

        if ptr >= bytes.len() {
            self.ser_inp_ptr = ptr;
            return false;
        }

        let is_param = (b'*'..=b'9').contains(&bytes[ptr]);

        while ptr < bytes.len() {
            let byte = bytes[ptr];
            let keep = if is_param {
                // Parameters accept digits plus the wildcard/ASCII punctuation span used by the firmware.
                (b'*'..=b'9').contains(&byte)
            } else {
                // Commands consume letters until a digit or separator terminates the token.
                byte >= b'A'
            };

            if !keep {
                break;
            }

            self.param_str.push(byte as char);
            ptr += 1;
        }

        self.ser_inp_ptr = ptr;
        is_param
    }

    /// Parses channel, command, argument, checksum, and value before dispatching one complete protocol line.
    pub fn parse_sub_ch(&mut self) -> Vec<String> {
        self.output_lines.clear();

        if self.ser_inp_str.is_empty() {
            self.serprompt(PromptCode::NoErr);
            return self.output_lines.clone();
        }

        let has_main_ch = self.ser_inp_str.contains(':');
        let is_request = !self.ser_inp_str.contains('=');
        // '=' means a setter, otherwise the frame is treated as a read/query.
        let first = self.ser_inp_str.as_bytes()[0];
        let is_omni = first == b'*';
        let is_result = first == b'#';

        if is_result {
            // Result frames are forwarded unchanged instead of being parsed again.
            self.write_ser_inp();
            return self.output_lines.clone();
        }

        self.ser_inp_ptr = 0;

        if has_main_ch {
            let _is_param = self.parse_extract();
            self.ser_inp_ptr = self.ser_inp_ptr.saturating_add(1);

            if is_omni {
                // Omni commands are echoed onward before local handling.
                self.write_ser_inp();
            } else if let Some(channel) = self.parse_i32(&self.param_str) {
                self.current_ch = channel;
            }
        }

        if !is_omni && self.current_ch != self.slave_ch && has_main_ch {
            // Addressed traffic for another slave is passed through untouched.
            self.write_ser_inp();
            return self.output_lines.clone();
        }

        // '!' or '?' requests the verbose response style used by the original protocol.
        self.verbose = self.ser_inp_str.contains('!') || self.ser_inp_str.contains('?');

        if let Some(check_pos) = self.ser_inp_str.find('$') {
            // XOR checksum covers everything before '$'; the '$xx' trailer is not included.
            let checksum_text = self.ser_inp_str.get(check_pos + 1..check_pos + 3);
            let Some(checksum_text) = checksum_text else {
                self.serprompt(PromptCode::CheckSumErr);
                return self.output_lines.clone();
            };

            let Some(checksum_in) = self.hex_to_u8(checksum_text) else {
                self.serprompt(PromptCode::CheckSumErr);
                return self.output_lines.clone();
            };

            let mut checksum_calc = 0_u8;
            for byte in self.ser_inp_str.as_bytes().iter().take(check_pos) {
                checksum_calc ^= *byte;
            }

            if checksum_calc != checksum_in {
                // Reject the frame immediately on checksum mismatch.
                self.serprompt(PromptCode::CheckSumErr);
                return self.output_lines.clone();
            }
        }

        self.set_systimer(255);
        self.led_activity = false;
        // Any valid local frame refreshes the activity timer and clears the LED.

        let mut used_command = false;
        let sub_ch_offset = if self.parse_extract() {
            // Numeric first token means direct sub-channel addressing.
            0_i32
        } else {
            // Otherwise parse a textual command and translate it through the command table.
            used_command = true;
            let cmd_which = self.cmd_to_index();
            let Some(offset) = cmd_which.sub_channel_offset() else {
                self.serprompt(PromptCode::SyntaxErr);
                return self.output_lines.clone();
            };

            let _is_param = self.parse_extract();
            // Commands carry a second token that selects the concrete sub-channel.
            i32::from(offset)
        };

        let sub_ch_base = if used_command && self.param_str.is_empty() {
            0
        } else {
            let Some(value) = self.parse_i32(&self.param_str) else {
                self.serprompt(PromptCode::ParamErr);
                return self.output_lines.clone();
            };
            value
        };

        self.sub_ch = sub_ch_base + sub_ch_offset;
        // Command aliases are normalized into the same absolute sub-channel space.

        if is_request {
            // Request path only resolves the current value; no payload parsing follows.
            self.parse_get_param();
            return self.output_lines.clone();
        }

        let Some(eq_pos) = self.ser_inp_str.find('=') else {
            self.serprompt(PromptCode::ParamErr);
            return self.output_lines.clone();
        };

        self.ser_inp_ptr = eq_pos + 1;

        if !self.parse_extract() {
            self.serprompt(PromptCode::ParamErr);
            return self.output_lines.clone();
        }

        let Some(value) = self.parse_f64(&self.param_str) else {
            self.serprompt(PromptCode::ParamErr);
            return self.output_lines.clone();
        };

        // Setter path keeps the Pascal convention of exposing float, int, and byte views together.
        self.param = value;
        self.param_int = value as i32;
        self.param_byte = self.param_int as u8;
        self.parse_set_param();

        self.output_lines.clone()
    }

    /// Formats the floating-point working parameter using selected width and decimal precision.
    fn write_param_ser(&mut self) {
        self.output_lines.push(format!(
            "{}={:.*}",
            self.sub_ch, self.nachkomma as usize, self.param
        ));
    }

    /// Formats integer working values without floating-point conversion.
    fn write_param_int_ser(&mut self) {
        self.output_lines
            .push(format!("{}={}", self.sub_ch, self.param_int));
    }

    /// Appends a verbose prompt label only when the current frame requested one.
    fn serprompt(&mut self, code: PromptCode) {
        self.output_lines.push(code.as_str().to_owned());
    }

    /// Echoes the original input for the protocol echo subchannel.
    fn write_ser_inp(&mut self) {
        self.output_lines.push(self.ser_inp_str.clone());
    }

    /// Builds the addressed result prefix from parsed main and subchannels.
    fn write_ch_prefix(&self) -> String {
        format!("{}:", self.current_ch)
    }

    /// Converts base units to milli-units for legacy MAH/MWH/current replies.
    fn param_mul_1000(&mut self) {
        self.param *= 1000.0;
    }

    /// Converts milli-unit setter values back to base units.
    fn param_div_1000(&mut self) {
        self.param /= 1000.0;
    }

    /// Scales the stored raw phase sample through the active voltage divider calibration.
    fn get_voltage(&mut self, on_time: bool) {
        self.param = if on_time {
            self.voltage_on
        } else {
            self.voltage_off
        };
    }

    /// Scales the stored raw phase sample through shunt-specific current calibration.
    fn get_current(&mut self, on_time: bool) {
        self.param = if on_time {
            self.current_on
        } else {
            self.current_off
        };
    }

    /// Returns a bounded raw diagnostic channel, using zero for unsupported indices.
    fn get_adc10(&self, channel: u8) -> u16 {
        self.adc10.get(usize::from(channel)).copied().unwrap_or(0)
    }

    /// Marks the Pascal critical-section boundary in the hardware-independent parser model.
    fn disable_ints(&mut self) {
        // The parser model records the boundary but has no interrupt controller.
    }

    /// Marks restoration of interrupts after parser-side calibration updates.
    fn enable_ints(&mut self) {
        // The parser model records the boundary but has no interrupt controller.
    }

    /// Records display invalidation caused by a setter without depending on an LCD backend.
    fn werte_on_lcd(&mut self) {
        self.display_refresh_count = self.display_refresh_count.saturating_add(1);
    }

    /// Recomputes current, voltage, resistance, and DAC factors immediately after calibration or option changes.
    pub(super) fn init_scales(&mut self) {
        let gain_i = self.option_array[OPT_GAIN_I];
        let u_ref = self.option_array[OPT_U_REF];
        let dac_type = (self.option_array[OPT_INIT_OPTIONS] as u8) & 0b0000_0011;
        let dac_max = if dac_type == 3 {
            65_535
        } else {
            DEFAULT_DAC_MAX
        };
        let dac_max_float = f64::from(dac_max);

        self.dac_max = dac_max;

        for index in 0..DACI_COUNT {
            let r_sense = self.option_array[OPT_RSENSE_BASE + index];

            self.dac_lsb_i[index] =
                (u_ref / r_sense) / (dac_max_float * self.daci_scales[index] * gain_i);

            // The EDL firmware intentionally uses the current DAC scale here; the DACR EEPROM
            // values were not reliable in the Pascal implementation.
            self.dac_lsb_r[index] = gain_i * r_sense * dac_max_float * self.daci_scales[index];
            self.adc16_lsb_i[index] =
                (self.adci_scales[index] * u_ref / r_sense) / ADC_MAX_16 / gain_i;
            self.adc10_lsb_i[index] =
                (self.adci_scales[index] * u_ref / r_sense) / ADC_MAX_10 / gain_i;
        }

        self.dc_ohm_min = self.option_array[OPT_RSENSE_BASE + 3] * self.divider_u * gain_i * 1.1;
        self.dc_ohm_max = self.option_array[OPT_RSENSE_BASE] * self.divider_u * gain_i * 100.0;
    }

    /// Represents the three-millisecond analog settling interval after calibration changes; this parser-only model has no clock to advance.
    fn mdelay(&mut self, _ms: u16) {
        // The parser-only model has no hardware clock to advance.
    }

    /// Clamps setpoints and timing and records the exact verbose limit outcome.
    fn check_limits(&mut self) {
        self.check_limit_err = PromptCode::NoErr;
        self.no_toggle = false;

        if self.dc_ohm < self.dc_ohm_min {
            self.dc_ohm = self.dc_ohm_min;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.dc_ohm > self.dc_ohm_max {
            self.dc_ohm = self.dc_ohm_max;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.dc_amp < 0.0 {
            self.dc_amp = 0.0;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.dc_amp > self.imax() {
            self.dc_amp = self.imax();
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.i_percent < 0 {
            self.i_percent = 0;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.i_percent >= 100 {
            self.no_toggle = true;
            self.i_percent = 100;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.pw_on_time < 1 {
            self.pw_on_time = 1;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.pw_off_time < 0 {
            self.pw_off_time = 0;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.pw_off_time == 0 {
            self.no_toggle = true;
        }

        if matches!(self.mode_select, Mode::Unknown(value) if value >= 128) {
            self.mode_select = Mode::OutputOff;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if matches!(self.mode_select, Mode::Unknown(_)) {
            self.mode_select = Mode::PloVolt;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if self.dc_watt > self.pmax() {
            self.dc_watt = self.pmax();
        }

        if self.dc_watt < 0.0 {
            self.dc_watt = 0.0;
            self.check_limit_err = PromptCode::ParamErr;
        }

        let voltage_limit = self.active_voltage_limit();
        if self.dc_volt > voltage_limit {
            self.dc_volt = voltage_limit;
        }

        if self.dc_volt < 0.0 {
            self.dc_volt = 0.0;
            self.check_limit_err = PromptCode::ParamErr;
        }

        if matches!(self.mode_select, Mode::RhiVolt | Mode::RloVolt) {
            self.no_toggle = true;
        }
    }

    /// Derives and quantizes resistance-mode current using the latest voltage and selected shunt.
    fn set_level_dac_r(&mut self) {
        self.init_scales();
        self.shunt_select = self.calc_range_r();
        self.old_shunt_select = self.shunt_select;

        let index = self.shunt_select as usize;
        self.dac_temp_on = self.quantize_dac(
            (self.divider_u * self.dac_lsb_r[index]) / self.dc_ohm,
            self.daci_offsets[index],
        );
        self.dac_temp_off = self.dac_temp_on;
        self.mode_mpx = false;
        self.update_mpxena();
    }

    /// Quantizes current-mode on/off DAC codes from setpoint and ripple percentage.
    fn set_level_dac_i(&mut self) {
        self.init_scales();
        let mut shunt = self.calc_range_i();
        if self.shunt_range <= SHUNT_D && self.shunt_range >= shunt {
            shunt = self.shunt_range;
        }
        self.shunt_select = shunt;
        self.old_shunt_select = self.shunt_select;

        let index = self.shunt_select as usize;
        self.dac_temp_on = self.quantize_dac(
            (self.dc_amp * self.dc_amp_mod) / self.dac_lsb_i[index],
            self.daci_offsets[index],
        );

        let percent = f64::from(self.i_percent) / 100.0;
        self.dac_temp_off = self.quantize_dac(
            (self.dc_amp * self.dc_amp_mod * percent) / self.dac_lsb_i[index],
            self.daci_offsets[index],
        );
        self.mode_mpx = true;
        self.update_mpxena();
    }

    /// Derives current from requested power and measured voltage before DAC quantization.
    fn set_level_dac_p(&mut self) {
        self.get_voltage(true);
        if self.param > 0.0 {
            self.dc_amp = self.dc_watt / self.param;
        }
        self.set_level_dac_i();
    }

    /// Selects the most sensitive shunt that still covers the current setpoint.
    fn calc_range_i(&self) -> u8 {
        let mut shunt = 0_u8;
        for index in 0..DACI_COUNT {
            if self.dc_amp > self.option_array[OPT_IMAX_BASE + index] {
                shunt = shunt.saturating_add(1).min(SHUNT_D);
            }
        }
        shunt
    }

    /// Selects a shunt that keeps resistance-derived current in range.
    fn calc_range_r(&self) -> u8 {
        for index in 0..DACI_COUNT {
            let threshold = self.option_array[OPT_RSENSE_BASE + index] * self.divider_u;
            if self.dc_ohm >= threshold {
                return index as u8;
            }
        }
        SHUNT_D
    }

    /// Applies calibration offset and clamps the result to installed DAC resolution.
    fn quantize_dac(&self, raw: f64, offset: i32) -> u16 {
        let value = (raw + 0.5) as i32 + offset;
        value.clamp(0, i32::from(self.dac_max)) as u16
    }

    /// Updates output multiplexer enable from mode, output state, and protection state.
    fn update_mpxena(&mut self) {
        self.mpxena = if self.mode_select == Mode::OutputOff {
            false
        } else {
            self.output_enable
        };
    }

    /// Returns the calibrated maximum current for the selected shunt.
    fn imax(&self) -> f64 {
        self.option_array[OPT_IMAX_BASE + 3]
    }

    /// Returns the option-backed maximum safe power.
    fn pmax(&self) -> f64 {
        self.option_array[OPT_PMAX]
    }

    /// Chooses the low/high voltage clamp paired with the selected mode.
    fn active_voltage_limit(&self) -> f64 {
        match self.mode_select {
            Mode::IhiVolt | Mode::RhiVolt | Mode::PhiVolt => self.option_array[OPT_UMAX_HI],
            Mode::OutputOff | Mode::IloVolt | Mode::RloVolt | Mode::PloVolt | Mode::Unknown(_) => {
                self.option_array[OPT_UMAX_LO]
            }
        }
    }

    /// Reloads the activity countdown to 255 after a valid local frame so LED decay follows traffic.
    fn set_systimer(&mut self, value: u8) {
        self.activity_timer = value;
    }

    /// Parses signed integer syntax without accepting partial tokens.
    fn parse_i32(&self, text: &str) -> Option<i32> {
        text.trim().parse().ok()
    }

    /// Parses decimal protocol syntax without accepting partial tokens.
    fn parse_f64(&self, text: &str) -> Option<f64> {
        text.trim().parse().ok()
    }

    /// Decodes exactly the hexadecimal byte format used by checksum and raw setters.
    fn hex_to_u8(&self, text: &str) -> Option<u8> {
        u8::from_str_radix(text.trim(), 16).ok()
    }

    /// Encodes parser mode variants using Pascal wire discriminants.
    fn mode_to_i32(&self) -> i32 {
        match self.mode_select {
            Mode::OutputOff => 0,
            Mode::IhiVolt => 1,
            Mode::IloVolt => 2,
            Mode::RhiVolt => 3,
            Mode::RloVolt => 4,
            Mode::PhiVolt => 5,
            Mode::PloVolt => 6,
            Mode::Unknown(value) => i32::from(value),
        }
    }

    /// Encodes menu variants using Pascal DSP subchannel values.
    fn modify_to_i32(&self) -> i32 {
        match self.modify {
            Modify::LowerMainMenu => 0,
            Modify::UpperMainMenu => 1,
            Modify::ModeMenu => 2,
            Modify::TOn => 3,
            Modify::TOff => 4,
            Modify::IOff => 5,
            Modify::TempMenu => 6,
            Modify::RiMenu => 7,
            Modify::CapMenu => 8,
            Modify::PwrCurMenu => 9,
            Modify::Unknown(value) => i32::from(value),
        }
    }
}
