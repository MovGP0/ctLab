//! Defines ACV state retained across parser, polling-loop, or interrupt operations.

#[allow(unused_imports)]
use super::*;

/// Collects acv state that must survive across polling-loop or interrupt updates.
#[derive(Debug, Clone)]
pub struct AcvState {
    /// Owns the hardware boundary through which this state performs all converter, relay, serial, and LCD access.
    pub(super) hw: MockHardware,

    /// Keeps EEPROM values together so reset and write-enable handling use one source of truth.
    pub(super) eeprom: EepromImage,

    /// Stores the address read from board straps and used to accept or prefix serial frames.
    pub(super) slave_ch: u8,

    /// Caches the eight output-switch bits shifted to the ACV relay and routing register.
    pub(super) switch_state: u8,

    /// Holds the auxiliary-function command byte edited on the panel and transmitted by the bit-banged UART.
    pub(super) aux_cmd: u8,

    /// Selects the consumer/professional S/PDIF mode and 48/96/192 kHz converter setup.
    pub(super) spdif_rate: Spdif,

    /// Keeps the ACV activity LED asserted for 125 systicks after serial or panel input.
    pub(super) activity_timer: Timer8,

    /// Delays restoration of the normal ACV display after a temporary edit view.
    pub(super) display_timer: Timer8,

    /// Sets the interval between ACV bar-graph refreshes to avoid unnecessary LCD traffic.
    pub(super) bar_graph_delay_timer: Timer8,

    /// Detects the pause that ends one encoder gesture and resets first-turn acceleration behavior.
    pub(super) incr_timer: Timer8,

    /// Stores the active ACV relay-gain index used for switch-table lookup and level scaling.
    pub(super) gain: u8,

    /// Remembers the last applied ACV gain so unchanged selections do not rewrite relays.
    pub(super) old_gain: u8,

    /// Contains the raw left-channel level register before gain and millivolt conversion.
    pub(super) left_level: u16,

    /// Contains the raw right-channel level register before gain and millivolt conversion.
    pub(super) right_level: u16,

    /// Contains the calibrated left-channel level in the integer domain used by display and serial output.
    pub(super) left_level_scaled: i32,

    /// Contains the calibrated right-channel level in the integer domain used by display and serial output.
    pub(super) right_level_scaled: i32,

    /// Contains the left-channel level compressed to the 0..255 bar-graph domain.
    pub(super) left_level_byte: u8,

    /// Contains the right-channel level compressed to the 0..255 bar-graph domain.
    pub(super) right_level_byte: u8,

    /// Caches the selected gain's display text, such as the signed decibel value shown on the LCD.
    pub(super) gain_str: String,

    /// Stores the integer numerator used to scale ACV ADC-board level counts.
    pub(super) scale_mult: u16,

    /// Stores the integer denominator paired with `scale_mult` for ACV level conversion.
    pub(super) scale_div: u16,

    /// Stores the enum produced by mnemonic lookup and consumed by command dispatch.
    pub(super) cmd_which: CmdWhich,

    /// Stores command string in the wire or LCD representation expected by the original firmware.
    pub(super) cmd_str: String,

    /// Holds the protocol subchannel selected by the current frame; 255 is the status channel.
    pub(super) sub_ch: u8,

    /// Tracks the most recently addressed channel so short-form commands can omit the address.
    pub(super) current_ch: u8,

    /// Records whether `?` or `!` requested a verbose status response for the active frame.
    pub(super) verbose: bool,

    /// Contains the parsed integer operand until range checking and command execution complete.
    pub(super) param_int: i32,

    /// Contains the parsed byte operand until range checking and command execution complete.
    pub(super) param_byte: u8,

    /// Contains the current CR-terminated command frame without its trailing carriage return.
    pub(super) ser_inp_str: String,

    /// Indexes the next unconsumed byte in the current command frame during token extraction.
    pub(super) ser_inp_ptr: usize,

    /// Selects the front-panel value or visualization currently being edited.
    pub(super) modify: Modify,

    /// Holds the current absolute rotary-encoder counter sampled from hardware.
    pub(super) incr_value: i32,

    /// Holds the previous encoder counter so the polling loop can accumulate signed movement.
    pub(super) old_incr_value: i32,

    /// Marks the encoder's fine-adjust mode selected by the Enter button.
    pub(super) incr_enter: bool,

    /// Suppresses acceleration and coarse rounding on the first encoder movement after a pause.
    pub(super) first_turn: bool,

    /// Accumulates signed raw encoder movement until it reaches the configured detent threshold.
    pub(super) incr_diff: i32,

    /// Stores accelerated encoder movement in tenths for ACV integer parameter updates.
    pub(super) incr_acc_int10: i32,

    /// Sets the number of raw encoder increments required for one accepted detent.
    pub(super) inc_rast: i32,

    /// Keeps the unaccelerated signed encoder step used by menu fields that must change one unit at a time.
    pub(super) incr_diff_byte: u8,

    /// Sets the field width used by the active serial or LCD formatter.
    pub(super) digits: u8,

    /// Sets the number of fractional digits emitted for the active parameter.
    pub(super) nachkomma: u8,

    /// Requests a display refresh after a setter or front-panel edit changes visible state.
    pub(super) changed_flag: bool,

    /// Stores parameter string in the wire or LCD representation expected by the original firmware.
    pub(super) param_str: String,

    /// Caches the packed protocol status byte: error in the low nibble, then unlock, overload, user-request, and busy bits.
    pub(super) status: u8,

    /// Counts protocol errors returned by `ERC` until that command clears the counter.
    pub(super) err_count: i32,

    /// Marks a parser failure so the next status response reports it once.
    pub(super) err_flag: bool,

    /// Selects the upper display/serial channel after optional left-right swapping.
    pub(super) upper_channel: char,

    /// Selects the lower display/serial channel after optional left-right swapping.
    pub(super) lower_channel: char,

    /// Latches left-channel ADC overload so serial and LCD output substitute the overload marker.
    pub(super) left_overload: bool,

    /// Latches right-channel ADC overload so serial and LCD output substitute the overload marker.
    pub(super) right_overload: bool,
}

impl Default for AcvState {
    /// Builds the pre-initialization ACV state before EEPROM values and board address are restored.
    fn default() -> Self {
        Self {
            hw: MockHardware::default(),
            eeprom: EepromImage::default(),
            slave_ch: 0,
            switch_state: 0,
            aux_cmd: 0,
            spdif_rate: Spdif::C48Khz,
            activity_timer: Timer8 { ticks: 0 },
            display_timer: Timer8 { ticks: 0 },
            bar_graph_delay_timer: Timer8 { ticks: 0 },
            incr_timer: Timer8 { ticks: 0 },
            gain: 0,
            old_gain: u8::MAX,
            left_level: 0,
            right_level: 0,
            left_level_scaled: 0,
            right_level_scaled: 0,
            left_level_byte: 0,
            right_level_byte: 0,
            gain_str: String::new(),
            scale_mult: 0,
            scale_div: 0,
            cmd_which: CmdWhich::Err,
            cmd_str: String::new(),
            sub_ch: 0,
            current_ch: 255,
            verbose: false,
            param_int: 0,
            param_byte: 0,
            ser_inp_str: String::new(),
            ser_inp_ptr: 0,
            modify: Modify::GainSel,
            incr_value: 0,
            old_incr_value: 0,
            incr_enter: false,
            first_turn: true,
            incr_diff: 0,
            incr_acc_int10: 0,
            inc_rast: 4,
            incr_diff_byte: 0,
            digits: 2,
            nachkomma: 1,
            changed_flag: false,
            param_str: String::new(),
            status: 0,
            err_count: 0,
            err_flag: false,
            upper_channel: 'L',
            lower_channel: 'R',
            left_overload: false,
            right_overload: false,
        }
    }
}

impl AcvState {
    /// Creates the ACV power-on state and runs the Pascal initialization sequence to restore EEPROM and configure S/PDIF.
    pub fn new() -> Self {
        let mut state = Self::default();
        state.init_all();
        state
    }

    /// Returns busy flag so the caller can gate the next protocol or conversion step.
    pub(super) fn busy_flag(&self) -> bool {
        self.status & 0x80 != 0
    }

    /// Updates status bit 7, which is reported as `[BUSY]` in the status response.
    pub(super) fn set_busy_flag(&mut self, value: bool) {
        if value {
            self.status |= 0x80;
        } else {
            self.status &= !0x80;
        }
    }

    /// Updates status bit 6, which is reported as `[SRQUSR]` in the status response.
    pub(super) fn set_user_srq_flag(&mut self, value: bool) {
        if value {
            self.status |= 0x40;
        } else {
            self.status &= !0x40;
        }
    }

    /// Returns ee unlocked so the caller can gate the next protocol or conversion step.
    pub(super) fn ee_unlocked(&self) -> bool {
        self.status & 0x10 != 0
    }

    /// Updates status bit 4, allowing EEPROM-changing commands only while the write-enable latch is set.
    pub(super) fn set_ee_unlocked(&mut self, value: bool) {
        if value {
            self.status |= 0x10;
        } else {
            self.status &= !0x10;
        }
    }

    /// Appends text to the active serial frame without changing parser state.
    pub(super) fn ser_out(&mut self, text: &str) {
        self.hw.serial_output.push_str(text);
    }

    /// Bit-bangs one auxiliary serial byte with the edge order expected by the attached device.
    pub(super) fn ser_aux(&mut self, my_byte: u8) {
        // Original code bit-bangs 19200 baud on PB4: start bit, 8 data bits LSB first, stop bit.
        self.hw.aux_serial_log.push(my_byte);
        self.hw.set_aux_serial_line(false);
        for bit in 0..8 {
            self.hw.set_aux_serial_line(my_byte & (1 << bit) != 0);
        }
        self.hw.set_aux_serial_line(true);
    }

    /// Uses multiply-before-divide integer scaling so level conversion keeps precision without pulling floating-point code into ACV.
    pub(super) fn mul_div_int(value: u16, mult: u16, div: u16) -> u16 {
        if div == 0 {
            return 0;
        }
        ((u32::from(value) * u32::from(mult)) / u32::from(div)) as u16
    }

    /// Reads both audio level registers, applies channel swapping, and updates overload flags before display or serial output.
    pub(super) fn get_levels(&mut self) {
        // Read both TRMS channels and derive the raw bargraph bytes plus the
        // gain-dependent millivolt values used by the LCD and remote commands.
        self.right_overload = false;
        self.left_overload = false;
        self.right_level = self.hw.get_adc(4);
        self.left_level = self.hw.get_adc(3);
        self.left_level_byte = (self.left_level >> 2) as u8;
        self.scale_div = ADC_RANGE_SCALES_DIV[self.gain as usize];

        if self.right_level > 1019 {
            self.right_overload = true;
            self.right_level_byte = 255;
            self.right_level_scaled = 0;
        } else {
            self.right_level_byte = (self.right_level >> 2) as u8;
            self.scale_mult = self.eeprom.adc_scales_r[self.gain as usize];
            self.right_level_scaled = i32::from(Self::mul_div_int(
                self.right_level,
                self.scale_mult,
                self.scale_div,
            ));
        }

        if self.left_level > 1019 {
            self.left_overload = true;
            self.left_level_byte = 255;
            self.left_level_scaled = 0;
        } else {
            self.left_level_byte = (self.left_level >> 2) as u8;
            self.scale_mult = self.eeprom.adc_scales_l[self.gain as usize];
            self.left_level_scaled = i32::from(Self::mul_div_int(
                self.left_level,
                self.scale_mult,
                self.scale_div,
            ));
        }
    }

    /// Copies persisted startup choices into live state so initialization and later commands observe the same configuration.
    pub(super) fn patch_copy_from_ee(&mut self) {
        // Load the persisted startup settings into the live state.
        self.inc_rast = self.eeprom.init_inc_rast;
        self.gain = self.eeprom.init_gain;
        self.spdif_rate = self.eeprom.init_rate;
    }

    /// Terminates the current serial response with CRLF because existing clients parse line-delimited frames.
    pub(super) fn ser_crlf(&mut self) {
        self.ser_out("\r\n");
    }

    /// Writes the addressed channel prefix before a payload so every response keeps the Pascal wire framing.
    pub(super) fn write_ch_prefix(&mut self) {
        let mut prefix = String::new();
        let _ = write!(
            prefix,
            "#{}:{}=",
            char::from(b'0' + self.slave_ch),
            self.sub_ch
        );
        self.ser_out(&prefix);
    }

    /// Writes serial inp to the serial, display, or peripheral destination selected by the implementation.
    pub(super) fn write_ser_inp(&mut self) {
        let line = self.ser_inp_str.clone();
        self.ser_out(&line);
        self.ser_crlf();
    }

    /// Encodes the current status and error flags into the Pascal prompt frame returned after commands.
    pub(super) fn ser_prompt(&mut self, my_err: Error, my_status: u8) {
        // Serial replies carry live status bits in the upper part and the current
        // error code in the low bits, matching the original ACV wire protocol.
        if self.verbose || my_err != Error::NoErr {
            self.sub_ch = ERR_SUB_CH;
            self.write_ch_prefix();
            let value = (my_err as u8).wrapping_add(my_status);
            self.ser_out(&value.to_string());
            self.ser_out(" ");
            self.ser_out(my_err.as_str());
            self.ser_crlf();
        }
        if my_err != Error::NoErr {
            self.err_count += 1;
            self.err_flag = true;
        }
    }

    /// Transfers I2C out adr10 using the byte order expected by the attached peripheral.
    pub(super) fn i2c_out_adr10(&mut self, register: u8, data: u8) {
        // The Pascal code uses TWI address 0x10 to reach the CS8406 control path.
        self.hw.twi_out_10(register, data);
    }

    /// Transfers I2C in adr10 using the byte order expected by the attached peripheral.
    pub(super) fn i2c_in_adr10(&self, register: u8) -> u8 {
        self.hw.twi_in_10(register)
    }

    /// Initializes spdif in the same order as the original startup routine.
    pub(super) fn init_spdif(&mut self) {
        // Program the SPDIF transmitter for the selected 48/96/192 kHz clock mode.
        self.i2c_out_adr10(0x04, 0b0000_0000);
        self.i2c_out_adr10(0x01, 0b0000_0001);
        self.i2c_out_adr10(0x12, 0b0000_0000);

        match self.spdif_rate {
            Spdif::C96Khz | Spdif::P96Khz => {
                self.hw.adc_config = 0b0100_0101;
                self.i2c_out_adr10(0x04, 0b0100_0000);
            }
            Spdif::C192Khz | Spdif::P192Khz => {
                self.hw.adc_config = 0b0100_0110;
                self.i2c_out_adr10(0x04, 0b0111_0000);
            }
            _ => {
                self.hw.adc_config = 0b0100_0100;
                self.i2c_out_adr10(0x04, 0b0110_0000);
            }
        }

        self.i2c_out_adr10(0x05, 0b0000_0101);
        // Channel-status bytes follow the consumer-mode layout from the CS8406 notes.
        self.i2c_out_adr10(0x20, 0b0010_0000);
        self.i2c_out_adr10(0x21, 0b0100_0001);
        self.i2c_out_adr10(0x22, 0b0000_0000);
        self.i2c_out_adr10(0x23, 0b0100_1000);
        self.i2c_out_adr10(0x24, 0b1100_0110);
        self.i2c_out_adr10(0x25, 0b1011_0110);
        self.i2c_out_adr10(0x26, 0b1111_0100);
        self.i2c_out_adr10(0x27, 0b1100_0110);
        self.i2c_out_adr10(0x28, 0b1110_0100);
        self.i2c_out_adr10(0x29, 0b0010_1110);
    }

    /// Applies gain as one coherent state and hardware transition.
    pub(super) fn switch_gain(&mut self) {
        // Map the logical gain to the relay/multiplexer pattern on Port B.
        if self.gain == self.old_gain {
            return;
        }
        self.old_gain = self.gain;
        self.switch_state = SWITCH_ARR[self.gain as usize] | PORTB_INIT;
        self.hw.port_b = self.switch_state;
    }

    /// Appends the prepared parameter text after the channel prefix and terminates the response with CRLF.
    pub(super) fn write_param_str_ser(&mut self) {
        self.write_ch_prefix();
        let param = self.param_str.clone();
        self.ser_out(&param);
        self.ser_crlf();
    }

    /// Converts to string into the representation used on the wire or display.
    pub(super) fn param_to_str(&mut self) {
        self.param_str = self.param_int.to_string();
    }

    /// Converts to string scaled into the representation used on the wire or display.
    pub(super) fn param_to_str_scaled(&mut self) {
        if self.gain > 4 {
            let value = format!("{:>3}", self.param_int);
            let mut chars: Vec<char> = value.chars().collect();
            if chars.len() >= 3 {
                chars.insert(2, '.');
            }
            self.param_str = chars.into_iter().collect();
        } else {
            self.param_str = format!("{:>4}", self.param_int);
        }
    }

    /// Converts the active integer parameter to decimal text and emits it as a framed serial response.
    pub(super) fn write_param_ser(&mut self) {
        self.param_to_str();
        self.write_param_str_ser();
    }

    /// Converts the active byte parameter without sign extension and emits it as a framed serial response.
    pub(super) fn write_param_byte_ser(&mut self) {
        self.param_str = self.param_byte.to_string();
        self.write_param_str_ser();
    }

    /// Renders soll werte on LCD into the fixed LCD cells used by the front panel.
    pub(super) fn soll_werte_on_lcd(&mut self) {
        self.digits = 2;
        self.nachkomma = 1;
        let mut my_modify = self.modify;

        if !self.bar_graph_delay_timer.is_zero()
            && matches!(self.modify, Modify::LevelBarDispl | Modify::MvDispl)
        {
            my_modify = Modify::GainSel;
        }

        match my_modify {
            Modify::MvDispl => {
                if self.incr_enter {
                    self.eeprom.init_gain = self.gain;
                }
                // Display the scaled TRMS reading in mV, or the overload marker.
                self.get_levels();
                let left = if self.left_level_byte > 253 {
                    format!("{}{}", self.upper_channel, OVERLOAD_STR)
                } else {
                    self.param_int = self.left_level_scaled;
                    self.param_to_str_scaled();
                    format!("{}{}{}", self.upper_channel, self.param_str, MV_STR)
                };
                let right = if self.right_level_byte > 253 {
                    format!("{}{}", self.lower_channel, OVERLOAD_STR)
                } else {
                    self.param_int = self.right_level_scaled;
                    self.param_to_str_scaled();
                    format!("{}{}{}", self.lower_channel, self.param_str, MV_STR)
                };
                self.hw.lcd_write_line(0, left);
                self.hw.lcd_write_line(1, right);
            }
            Modify::LevelBarDispl => {
                if self.incr_enter {
                    self.eeprom.init_gain = self.gain;
                }
                // Same measurement path as the mV view, but rendered as the PM-8 bargraph panel.
                self.get_levels();
                self.hw
                    .lcd_write_bargraph_line(0, self.upper_channel, self.left_level_byte);
                self.hw
                    .lcd_write_bargraph_line(1, self.lower_channel, self.right_level_byte);
            }
            Modify::GainSel => {
                if self.incr_enter {
                    self.eeprom.init_gain = self.gain;
                }
                let my_gain = i32::from(self.gain) * 10 - 20;
                self.gain_str = format!("{:+3}", my_gain);
                self.hw
                    .lcd_write_line(0, format!("{}{}{}", self.gain_str, DB_STR, LCD_CURSOR_CHAR));
                self.hw.lcd_write_line(1, GAIN_SEL_STR.to_string());
            }
            Modify::AuxCmdSel => {
                if self.incr_enter {
                    self.eeprom.init_aux_cmd = self.aux_cmd;
                }
                self.hw.lcd_write_line(
                    0,
                    format!("{AUX_CMD_SEL_STR} {:02X} {LCD_CURSOR_CHAR}", self.aux_cmd),
                );
                self.hw.lcd_write_line(1, AUX_CMD_STR.to_string());
            }
            Modify::RateSel => {
                if self.incr_enter {
                    self.eeprom.init_rate = self.spdif_rate;
                }
                self.hw.lcd_write_line(
                    0,
                    format!(
                        "{}{LCD_CURSOR_CHAR}",
                        RATE_STR_ARR[self.spdif_rate as usize]
                    ),
                );
                self.hw.lcd_write_line(1, RATE_SEL_STR.to_string());
            }
        }

        if self.incr_enter {
            self.hw.lcd_write_line(1, MEMORIZED_STR.to_string());
            self.display_timer.set(100);
        }
        self.incr_enter = false;
    }

    /// Validates limits before dependent hardware state is changed.
    pub(super) fn check_limits(&mut self) -> bool {
        // Report whether a caller tried to step beyond the legal gain/rate range.
        let mut out_of_range = false;

        if self.gain > 127 {
            self.gain = 0;
            out_of_range = true;
        }
        if self.gain > 7 {
            self.gain = 7;
            out_of_range = true;
        }
        if self.spdif_rate as u8 > 5 {
            self.spdif_rate = Spdif::P192Khz;
            out_of_range = true;
        }

        out_of_range
    }

    /// Parses get parameter and updates only the state owned by that protocol phase.
    pub(super) fn parse_get_param(&mut self) {
        // Subchannels expose sample-rate, gain, live levels, calibration tables,
        // status, and identity values from the original command set.
        let my_index = self.sub_ch % 10;
        self.digits = 2;
        self.nachkomma = 1;

        match self.sub_ch {
            8 => {
                self.param_byte = self.spdif_rate as u8;
                self.write_param_byte_ser();
            }
            10 => {
                self.get_levels();
                self.param_int = self.left_level_scaled;
                if self.left_overload {
                    self.param_str = "-9999 [OVERLD]".to_string();
                } else {
                    self.param_to_str_scaled();
                }
                self.write_param_str_ser();
            }
            11 => {
                self.get_levels();
                self.param_int = self.right_level_scaled;
                if self.right_overload {
                    self.param_str = "-9999 [OVERLD]".to_string();
                } else {
                    self.param_to_str_scaled();
                }
                self.write_param_str_ser();
            }
            19 => {
                self.param_byte = self.gain;
                self.write_param_byte_ser();
            }
            50 => {
                self.get_levels();
                self.param_int = i32::from(self.left_level);
                self.write_param_ser();
            }
            51 => {
                self.get_levels();
                self.param_int = i32::from(self.right_level);
                self.write_param_ser();
            }
            80 => {
                self.param_byte = self.modify as u8;
                self.write_param_byte_ser();
            }
            89 => {
                self.param_byte = self.inc_rast as u8;
                self.write_param_byte_ser();
            }
            99 => {
                // "ALL" returns both RMS channels as two consecutive replies.
                self.get_levels();
                self.param_int = self.left_level_scaled;
                if self.left_level_byte > 253 {
                    self.param_str = "-9999 [OVERLD]".to_string();
                } else {
                    self.param_to_str_scaled();
                }
                self.sub_ch = 10;
                self.write_param_str_ser();

                self.param_int = self.right_level_scaled;
                if self.right_level_byte > 253 {
                    self.param_str = "-9999 [OVERLD]".to_string();
                } else {
                    self.param_to_str_scaled();
                }
                self.sub_ch = 11;
                self.write_param_str_ser();
            }
            150 => {
                self.param_byte = self.eeprom.init_gain;
                self.write_param_byte_ser();
            }
            151 => {
                self.param_byte = self.eeprom.init_rate as u8;
                self.write_param_byte_ser();
            }
            152 => {
                self.param_byte = u8::from(self.eeprom.init_lr_swap);
                self.write_param_byte_ser();
            }
            200..=207 => {
                self.param_int = i32::from(self.eeprom.adc_scales_l[my_index as usize]);
                self.write_param_ser();
            }
            210..=217 => {
                self.param_int = i32::from(self.eeprom.adc_scales_r[my_index as usize]);
                self.write_param_ser();
            }
            230 => {
                self.param_byte = self.i2c_in_adr10(0x7f);
                self.write_param_byte_ser();
            }
            251 => {
                // Error count since reset.
                self.param_int = self.err_count;
                self.write_param_ser();
            }
            252 => {
                // Stored UART divisor; it only takes effect after the next reset.
                self.param_byte = self.eeprom.ee_ser_baud_reg;
                self.write_param_byte_ser();
            }
            253 => {
                // Serial test: echo the input line unchanged.
                let line = self.ser_inp_str.clone();
                self.ser_out(&line);
                self.ser_crlf();
            }
            254 => {
                self.write_ch_prefix();
                self.ser_out(VERS1_STR);
                self.ser_crlf();
            }
            250 | 255 => {
                self.ser_prompt(Error::NoErr, self.status);
            }
            _ => self.ser_prompt(Error::ParamErr, 0),
        }
    }

    /// Parses set parameter and updates only the state owned by that protocol phase.
    pub(super) fn parse_set_param(&mut self) {
        let my_index = self.sub_ch % 10;

        if self.busy_flag() {
            self.ser_prompt(Error::BusyErr, 0);
            return;
        }

        self.changed_flag = true;

        match self.sub_ch {
            8 => {
                self.spdif_rate = Self::spdif_from_byte(self.param_byte);
                self.check_limits();
                self.init_spdif();
            }
            9 => self.ser_aux(self.param_byte),
            19 => {
                self.gain = self.param_byte;
                self.check_limits();
            }
            20 => {}
            80 => {
                // Select which front-panel display mode the LCD should show.
                if self.param_byte > Modify::LevelBarDispl as u8 {
                    self.ser_prompt(Error::ParamErr, 0);
                    return;
                }
                self.modify = Self::modify_from_byte(self.param_byte);
            }
            89 => {
                // Number of encoder pulses that make one detent step.
                if self.ee_unlocked() {
                    self.inc_rast = self.param_int;
                    self.eeprom.init_inc_rast = self.inc_rast;
                } else {
                    self.ser_prompt(Error::LockedErr, 0);
                    return;
                }
            }
            150..=152 | 200..=217 => {
                if self.ee_unlocked() {
                    match self.sub_ch {
                        150 => self.eeprom.init_gain = self.param_byte,
                        151 => self.eeprom.init_rate = Self::spdif_from_byte(self.param_byte),
                        152 => self.eeprom.init_lr_swap = self.param_byte != 0,
                        200..=207 => {
                            self.eeprom.adc_scales_l[my_index as usize] = self.param_int as u16
                        }
                        210..=217 => {
                            // Original Pascal writes ADCscalesL here as well. Kept intentionally.
                            self.eeprom.adc_scales_l[my_index as usize] = self.param_int as u16
                        }
                        _ => {}
                    }
                } else {
                    self.ser_prompt(Error::LockedErr, 0);
                    return;
                }
            }
            251 => self.err_count = self.param_int,
            252 => {
                // Baud-rate changes are stored now but only applied after reboot.
                if self.ee_unlocked() {
                    self.eeprom.ee_ser_baud_reg = self.param_byte;
                } else {
                    self.ser_prompt(Error::LockedErr, 0);
                    return;
                }
            }
            250 => {}
            _ => {
                self.ser_prompt(Error::ParamErr, 0);
                return;
            }
        }

        self.set_ee_unlocked(false);
        // Subchannel 250 temporarily unlocks EEPROM-backed settings.
        if self.sub_ch == 250 {
            self.set_ee_unlocked(true);
        }

        if self.check_limits() {
            self.ser_prompt(Error::ParamErr, self.status);
        } else {
            self.ser_prompt(Error::NoErr, self.status);
        }
        self.switch_gain();
    }

    /// Parses the current command token into its case-insensitive semantic command enum.
    pub(super) fn cmd_to_index(&mut self) -> CmdWhich {
        CmdWhich::from_str(&self.param_str)
    }

    /// Parses extract and updates only the state owned by that protocol phase.
    pub(super) fn parse_extract(&mut self) -> bool {
        // Integer-only token extraction: digits form parameters, letters form commands.
        self.param_str.clear();
        let bytes = self.ser_inp_str.as_bytes();
        let mut is_param = false;

        // Skip leading spaces before the next token.
        while self.ser_inp_ptr < bytes.len() && bytes[self.ser_inp_ptr] == b' ' {
            self.ser_inp_ptr += 1;
        }

        if self.ser_inp_ptr >= bytes.len() {
            return false;
        }

        let first = bytes[self.ser_inp_ptr];
        if (b'*'..=b'9').contains(&first) {
            is_param = true;
            while self.ser_inp_ptr < bytes.len() {
                let my_char = bytes[self.ser_inp_ptr] as char;
                if my_char.is_ascii_digit() {
                    self.param_str.push(my_char);
                    self.ser_inp_ptr += 1;
                } else {
                    return is_param;
                }
            }
        } else {
            while self.ser_inp_ptr < bytes.len() {
                let my_char = bytes[self.ser_inp_ptr] as char;
                if my_char >= 'A' {
                    self.param_str.push(my_char);
                    self.ser_inp_ptr += 1;
                } else {
                    return is_param;
                }
            }
        }

        is_param
    }

    /// Parses sub channel and updates only the state owned by that protocol phase.
    pub(super) fn parse_sub_ch(&mut self) {
        // Pre-parse the incoming line, reject traffic for other channels, and then
        // dispatch either a direct subchannel access or a named command.
        if self.ser_inp_str.is_empty() {
            self.ser_prompt(Error::NoErr, 0);
            return;
        }

        let has_main_ch = self.ser_inp_str.contains(':');
        let is_request = !self.ser_inp_str.contains('=');
        let first_char = self.ser_inp_str.chars().next().unwrap_or_default();
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
                self.write_ser_inp();
            } else if let Ok(ch) = self.param_str.parse::<u8>() {
                self.current_ch = ch;
            }
        }

        if !is_omni && self.current_ch != self.slave_ch && has_main_ch {
            self.write_ser_inp();
            return;
        }

        self.verbose = self.ser_inp_str.contains('!') || self.ser_inp_str.contains('?');
        if let Some(checksum_pos) = self.ser_inp_str.find('$') {
            // XOR checksum covers everything before '$'; the '$xx' suffix is excluded.
            let checksum_hex = self
                .ser_inp_str
                .get(checksum_pos + 1..checksum_pos + 3)
                .unwrap_or_default();
            let checksum_in = u8::from_str_radix(checksum_hex, 16).unwrap_or_default();
            let checksum = self.ser_inp_str[..checksum_pos]
                .bytes()
                .fold(0u8, |acc, byte| acc ^ byte);
            if checksum != checksum_in {
                self.ser_prompt(Error::ChecksumErr, 0);
                return;
            }
        }

        self.activity_timer.set(25);
        self.hw.led_activity = false;

        let sub_ch_offset = if self.parse_extract() {
            0
        } else {
            self.cmd_which = self.cmd_to_index();
            if self.cmd_which == CmdWhich::Err {
                self.ser_prompt(Error::SyntaxErr, 0);
                return;
            }
            let Some(offset) = self.cmd_which.sub_channel_offset() else {
                self.ser_prompt(Error::SyntaxErr, 0);
                return;
            };
            let _ = self.parse_extract();
            offset
        };

        let base_sub_ch = match self.param_str.parse::<u8>() {
            Ok(value) => value,
            Err(_) => {
                self.ser_prompt(Error::ParamErr, 0);
                return;
            }
        };
        self.sub_ch = base_sub_ch.wrapping_add(sub_ch_offset);

        if is_request {
            self.parse_get_param();
        } else if let Some(eq_pos) = self.ser_inp_str.find('=') {
            self.ser_inp_ptr = eq_pos + 1;
            if self.parse_extract() {
                match self.param_str.parse::<i32>() {
                    Ok(value) => {
                        self.param_int = value;
                        self.param_byte = value as u8;
                    }
                    Err(_) => {
                        self.ser_prompt(Error::ParamErr, 0);
                        return;
                    }
                }
            } else {
                self.ser_prompt(Error::ParamErr, 0);
                return;
            }
            self.parse_set_param();
        } else {
            self.ser_prompt(Error::ParamErr, 0);
        }
    }

    /// Executes chores to service pending serial, trigger, measurement, and panel work without reordering them.
    pub(super) fn chores(&mut self) {
        // The structural ACV port has no additional foreground maintenance yet.
    }

    /// Handles serial char as one bounded polling-loop or interrupt service step.
    pub(super) fn process_serial_char(&mut self, my_char: char) {
        // Keep only printable 7-bit ASCII and treat carriage return as end-of-command.
        if (' '..='\u{7f}').contains(&my_char) {
            self.ser_inp_str.push(my_char);
        }
        if my_char == '\u{8}' {
            self.ser_inp_str.pop();
        }
        if my_char == '\r' {
            self.parse_sub_ch();
            self.ser_inp_str.clear();
        }
    }

    /// Queues serial char for the next bounded consumer without changing unrelated state.
    pub fn push_serial_char(&mut self, my_char: char) {
        self.hw.serial_input.push_back(my_char);
    }

    /// Validates serial before dependent hardware state is changed.
    pub fn check_ser(&mut self) {
        while let Some(my_char) = self.hw.serial_read_timeout(2) {
            self.process_serial_char(my_char);
        }
    }

    /// Validates delay before dependent hardware state is changed.
    pub(super) fn check_delay(&mut self, my_delay: u8) {
        // The Pascal firmware services serial input during UI delays.
        for _ in 0..my_delay {
            self.check_ser();
            self.chores();
        }
    }

    /// Debounces and decodes masked button sample before changing front-panel state or emitting a user request.
    pub(super) fn masked_button_sample(button_temp: u8) -> u8 {
        button_temp | BUTTON_UNUSED_BITS_MASK
    }

    /// Debounces and decodes front panel button event before changing front-panel state or emitting a user request.
    pub(super) fn front_panel_button_event(&mut self, button_temp: Option<u8>) -> Option<u8> {
        let button_temp = Self::masked_button_sample(button_temp?);
        self.hw.button_temp = button_temp;

        if button_temp == BUTTON_RELEASED {
            self.hw.button_debounce_sample = BUTTON_RELEASED;
            self.hw.button_waiting_for_release = false;
            return None;
        }

        if self.hw.button_waiting_for_release {
            return None;
        }

        if self.hw.button_debounce_sample == button_temp {
            self.hw.button_waiting_for_release = true;
            Some(button_temp)
        } else {
            self.check_delay(1);
            self.hw.button_debounce_sample = button_temp;
            None
        }
    }

    /// Initializes all in the same order as the original startup routine.
    pub(super) fn init_all(&mut self) {
        self.hw.port_b = PORTB_INIT;
        self.hw.port_c = PORTC_INIT;
        self.hw.port_d = PORTD_INIT;

        if !(9..=239).contains(&self.eeprom.ee_ser_baud_reg) {
            self.eeprom.ee_ser_baud_reg = 51;
        }
        self.hw.uart_baud_reg = self.eeprom.ee_ser_baud_reg;
        self.hw.uart_double_speed = true;

        self.patch_copy_from_ee();
        self.hw.adc_config = 0;
        self.slave_ch = (!self.hw.pin_d) >> 5;
        self.hw.led_activity = false;
        self.hw.lcd_present = true;
        // The original boot code uploads custom LCD glyphs before showing version/address.
        self.hw.lcd_write_line(0, VERS3_STR.to_string());
        self.hw
            .lcd_write_line(1, format!("{ADR_STR}{}", char::from(b'0' + self.slave_ch)));

        if self.eeprom.ee_initialized != EE_INITIALIZED_MAGIC {
            // Empty EEPROM falls back to the built-in defaults from the Pascal image.
            self.eeprom = EepromImage::default();
            self.patch_copy_from_ee();
        }

        self.switch_gain();
        self.hw.led_activity = true;
        self.status = 0;
        self.incr_value = 0;
        self.old_incr_value = 0;
        self.incr_diff = 0;
        self.incr_enter = false;
        self.modify = Modify::GainSel;
        self.soll_werte_on_lcd();
        self.modify = Modify::LevelBarDispl;
        self.first_turn = true;
        self.sub_ch = 254;
        self.write_ch_prefix();
        self.ser_out(VERS1_STR);
        if self.eeprom.ee_initialized != EE_INITIALIZED_MAGIC {
            self.ser_out(EE_NOT_PROGRAMMED_STR);
        }
        self.ser_crlf();
        self.current_ch = 255;
        self.err_count = 0;
        self.changed_flag = true;
        self.bar_graph_delay_timer.set(150);
        self.aux_cmd = self.eeprom.init_aux_cmd;
        self.ser_aux(self.aux_cmd);
        self.hw.adc_config = 0b0100_0000;
        self.init_spdif();

        if self.eeprom.init_lr_swap {
            self.upper_channel = 'R';
            self.lower_channel = 'L';
        } else {
            self.upper_channel = 'L';
            self.lower_channel = 'R';
        }
    }

    /// Executes main loop step to service pending serial, trigger, measurement, and panel work without reordering them.
    pub fn main_loop_step(&mut self, new_rotary_value: i32, button_temp: Option<u8>) {
        self.check_ser();
        self.hw.rotary_value = new_rotary_value;

        if self.activity_timer.is_zero() {
            self.hw.led_activity = true;
        }

        if self.hw.lcd_present && !self.hw.serial_pending() {
            self.incr_value = self.hw.rotary_value;

            if self.incr_value != self.old_incr_value {
                self.activity_timer.set(25);
                self.hw.led_activity = false;
                self.incr_diff += self.incr_value - self.old_incr_value;
                self.old_incr_value = self.incr_value;
                self.incr_timer.set(20);

                // The hardware encoder resolves in two-count steps, so changes are
                // only applied once enough pulses have accumulated for one detent.
                if self.incr_diff.abs() >= self.inc_rast {
                    self.changed_flag = true;
                    self.set_busy_flag(true);
                    self.incr_diff /= self.inc_rast;
                    self.incr_diff_byte = self.incr_diff as u8;

                    // Fast turns accelerate by doubling the effective step size.
                    if self.incr_diff.abs() > 1 {
                        self.incr_diff *= 2;
                    }
                    if self.incr_diff.abs() > 2 {
                        self.incr_diff *= 2;
                    }

                    self.incr_acc_int10 = self.incr_diff * 10;
                    self.display_timer.set(10);

                    if self.first_turn {
                        self.ser_prompt(Error::NoErr, self.status.wrapping_add(67));
                    }

                    match self.modify {
                        Modify::AuxCmdSel => {
                            self.aux_cmd = self.aux_cmd.wrapping_add(self.incr_diff as u8);
                            self.sub_ch = 9;
                            self.parse_get_param();
                            // Forward the helper command to the attached ULD/aux device.
                            self.ser_aux(self.aux_cmd);
                        }
                        Modify::RateSel => {
                            let next = (self.spdif_rate as i32 + i32::from(self.incr_diff_byte))
                                .clamp(0, 5) as u8;
                            self.spdif_rate = Self::spdif_from_byte(next);
                            self.check_limits();
                            self.init_spdif();
                        }
                        Modify::GainSel | Modify::MvDispl | Modify::LevelBarDispl => {
                            self.display_timer.set(10);
                            self.bar_graph_delay_timer.set(75);
                            self.gain = self.gain.wrapping_add(self.incr_diff_byte);
                            self.check_limits();
                            self.switch_gain();
                            self.sub_ch = 19;
                            self.parse_get_param();
                        }
                    }

                    self.incr_diff = 0;
                    self.check_limits();
                    self.soll_werte_on_lcd();
                    self.first_turn = false;
                }
            }

            self.check_delay(1);

            if let Some(button_temp) = self.front_panel_button_event(button_temp) {
                // Front-panel buttons are wired active-low.
                self.changed_flag = true;
                self.set_busy_flag(true);

                let button_enter = button_temp & (1 << 3) == 0;
                let button_left = button_temp & (1 << 5) == 0;
                let button_right = button_temp & (1 << 4) == 0;

                if button_enter {
                    self.ser_prompt(Error::NoErr, self.status.wrapping_add(67));
                    self.incr_enter = true;
                }
                if button_left {
                    self.ser_prompt(Error::NoErr, self.status.wrapping_add(65));
                    self.modify = Self::next_modify(self.modify);
                }
                if button_right {
                    self.ser_prompt(Error::NoErr, self.status.wrapping_add(66));
                    self.modify = Self::prev_modify(self.modify);
                }

                self.display_timer.set(10);
                self.soll_werte_on_lcd();
                self.first_turn = false;
            }
        }

        if self.incr_timer.is_zero() {
            self.incr_timer.set(20);
            if !self.first_turn {
                self.ser_prompt(Error::NoErr, self.status.wrapping_add(64));
            }
            self.first_turn = true;
        }

        if self.display_timer.is_zero() && self.hw.lcd_present {
            self.display_timer.set(10);
            self.set_busy_flag(false);
            self.soll_werte_on_lcd();
            self.changed_flag = false;
        }

        self.activity_timer.tick();
        self.display_timer.tick();
        self.bar_graph_delay_timer.tick();
        self.incr_timer.tick();
    }

    /// Maps spdif from byte into the typed state used internally, rejecting or defaulting unsupported wire values as the implementation specifies.
    pub(super) fn spdif_from_byte(value: u8) -> Spdif {
        match value {
            1 => Spdif::C96Khz,
            2 => Spdif::C192Khz,
            3 => Spdif::P48Khz,
            4 => Spdif::P96Khz,
            5 => Spdif::P192Khz,
            _ => Spdif::C48Khz,
        }
    }

    /// Maps modify from byte into the typed state used internally, rejecting or defaulting unsupported wire values as the implementation specifies.
    pub(super) fn modify_from_byte(value: u8) -> Modify {
        match value {
            0 => Modify::AuxCmdSel,
            1 => Modify::RateSel,
            2 => Modify::GainSel,
            3 => Modify::LevelBarDispl,
            4 => Modify::MvDispl,
            _ => Modify::GainSel,
        }
    }

    /// Moves the front-panel selection by one valid menu item while skipping values that have no editable display.
    pub(super) fn next_modify(value: Modify) -> Modify {
        match value {
            Modify::AuxCmdSel => Modify::RateSel,
            Modify::RateSel => Modify::GainSel,
            Modify::GainSel => Modify::LevelBarDispl,
            Modify::LevelBarDispl => Modify::MvDispl,
            Modify::MvDispl => Modify::AuxCmdSel,
        }
    }

    /// Moves the front-panel selection by one valid menu item while skipping values that have no editable display.
    pub(super) fn prev_modify(value: Modify) -> Modify {
        match value {
            Modify::AuxCmdSel => Modify::MvDispl,
            Modify::RateSel => Modify::AuxCmdSel,
            Modify::GainSel => Modify::RateSel,
            Modify::LevelBarDispl => Modify::GainSel,
            Modify::MvDispl => Modify::LevelBarDispl,
        }
    }
}
