use super::*;

#[derive(Debug, Clone)]
pub struct DeviceState<H> {
    pub hw: H,
    pub eeprom: EepromData,
    pub scale: CalibrationScale,
    pub status: RuntimeStatus,
    pub faults: FaultFlags,
    pub err_count: u16,
    pub button_number: u8,
    pub main_channel: u8,
    pub sub_channel: u8,
    pub display_sub_channel: u8,
    pub voltage_set: Float,
    pub current_set: Float,
    pub voltage_mod: Float,
    pub current_mod: Float,
    pub measured_voltage: Float,
    pub measured_current: Float,
    pub measured_power: Float,
    pub input_voltage: Float,
    pub capacity_mah: Float,
    pub capacity_mwh: Float,
    pub ripple_percent: Float,
    pub ripple_voltage: Float,
    pub no_toggle: bool,
    pub pw_on_time_ms: u16,
    pub pw_off_time_ms: u16,
    pub pw_counter_ms: u16,
    // 255 disabled tracking in Pascal; 0..7 addressed another PSU module.
    pub track_channel: u8,
    pub current_range: CurrentRange,
    pub voltage_range: VoltageRange,
    pub old_current_range: Option<CurrentRange>,
    pub old_voltage_range: Option<VoltageRange>,
    pub auto_current_range: CurrentRange,
    pub dac_raw_u_on: u16,
    pub dac_raw_u_off: u16,
    pub dac_raw_i: u16,
    pub relay_voltage_low: Float,
    pub relay_voltage_high: Float,
    pub relay_state_high: bool,
    pub old_relay_state_high: bool,
    pub fault_timer: u8,
    pub temperature_timer: u8,
    // Active front-panel edit page for the encoder/button UI.
    pub panel_modify: Modify,
    pub inc_rast: i32,
    pub incr_fine: bool,
    pub first_turn: bool,
    pub incr_diff: i32,
    pub incr_acc_float: Float,
    pub inc_fine_div: Float,
    pub inc_coarse_div: Float,
    pub temperature_c: Option<Float>,
    pub locked: bool,
    pub output_enabled: bool,
    pub ser_input: String,
    pub param_str: String,
}
impl<H: DcgHardware> DeviceState<H> {
    pub fn new(hw: H) -> Self {
        Self {
            hw,
            eeprom: EepromData::default(),
            scale: CalibrationScale::default(),
            status: RuntimeStatus::default(),
            faults: FaultFlags::default(),
            err_count: 0,
            button_number: 0,
            main_channel: 0,
            sub_channel: 0,
            display_sub_channel: 0,
            voltage_set: 5.0,
            current_set: 0.02,
            voltage_mod: 1.0,
            current_mod: 1.0,
            measured_voltage: 0.0,
            measured_current: 0.0,
            measured_power: 0.0,
            input_voltage: 0.0,
            capacity_mah: 0.0,
            capacity_mwh: 0.0,
            ripple_percent: 0.0,
            ripple_voltage: 0.0,
            no_toggle: true,
            pw_on_time_ms: 0,
            pw_off_time_ms: 0,
            pw_counter_ms: 0,
            track_channel: 255,
            current_range: CurrentRange::Dc20mA,
            voltage_range: VoltageRange::ULow,
            old_current_range: None,
            old_voltage_range: None,
            auto_current_range: CurrentRange::Dc20mA,
            dac_raw_u_on: 0,
            dac_raw_u_off: 0,
            dac_raw_i: 0,
            relay_voltage_low: 0.0,
            relay_voltage_high: 0.0,
            relay_state_high: false,
            old_relay_state_high: true,
            fault_timer: 0,
            temperature_timer: 0,
            panel_modify: Modify::Volt,
            inc_rast: 4,
            incr_fine: false,
            first_turn: true,
            incr_diff: 0,
            incr_acc_float: 0.0,
            inc_fine_div: 1_000.0,
            inc_coarse_div: 10.0,
            temperature_c: None,
            locked: false,
            output_enabled: false,
            ser_input: String::new(),
            param_str: String::new(),
        }
    }

    pub fn with_eeprom(hw: H, eeprom: EepromData) -> Self {
        let mut state = Self::new(hw);
        state.eeprom = eeprom;
        state
    }

    pub fn set_lm75_temp(&mut self) {
        // Pascal programmed the LM75 fan threshold and a 3 C hysteresis band
        // through Tos/Thyst when the DCP/LM75 option bit was present.
    }

    pub fn get_lm75_temp(&mut self) {
        // The original code polled the LM75 on a slow cadence because the
        // device has about 100 ms conversion latency.
        self.temperature_c = self.hw.read_temp_c();
    }

    pub fn init_scales(&mut self) {
        let options = self.eeprom.init_options();
        let dac16_present = (options & DAC16_PRESENT_BIT) != 0;
        let adc16_present = (options & ADC16_PRESENT_BIT) != 0;
        let dcp_present = (options & DCP_PRESENT_BIT) != 0;

        let uref = self.eeprom.uref();
        let gain_out = self.eeprom.init_gain_out();
        let mut u_fac = if dac16_present { 2.0 * uref } else { uref };
        let dac_steps = if dac16_present { 65_536.0 } else { 4_096.0 };
        let adc_steps = if adc16_present { 65_536.0 } else { 1_024.0 };

        self.scale.options = options;
        self.scale.dac16_present = dac16_present;
        self.scale.adc16_present = adc16_present;
        self.scale.dcp_present = dcp_present;
        self.scale.dac_max = (dac_steps as u32 - 1) as u16;
        self.scale.dac_lsb_u[0] = u_fac * gain_out / (dac_steps * self.eeprom.dac_u_scales[0]);
        self.scale.dac_lsb_u[1] = u_fac * self.eeprom.init_gain_pre() * gain_out
            / (dac_steps * self.eeprom.dac_u_scales[1]);
        for index in 0..2 {
            self.scale.adc_lsb_u[index] =
                self.eeprom.adc_u_fac(index) * self.eeprom.adc_u_scales[index] * uref * gain_out
                    / adc_steps;
        }

        u_fac *= self.eeprom.init_gain_i();
        for index in 0..4 {
            self.scale.dac_lsb_i[index] =
                (u_fac / self.eeprom.rsense(index)) / (dac_steps * self.eeprom.dac_i_scales[index]);
            self.scale.adc_lsb_i[index] = (self.eeprom.adc_i_scales[index] * uref
                / (2.0 * self.eeprom.rsense(index)))
                / adc_steps;
        }

        self.scale.switchpoint = self.eeprom.init_switch_u();
        self.scale.relay_low = self.eeprom.init_hyst_low();
        self.scale.relay_high = self.eeprom.init_hyst_high();
        self.relay_voltage_low = self.scale.relay_low;
        self.relay_voltage_high = self.scale.relay_high;
        self.voltage_mod = 1.0;
        self.ripple_percent = self.eeprom.init_ripple_percent() as Float;
        self.pw_on_time_ms = self.eeprom.init_ton_time();
        self.pw_off_time_ms = self.eeprom.init_toff_time();
        self.pw_counter_ms = self.pw_on_time_ms;
        self.inc_rast = i32::from(self.eeprom.inc_rast_def).max(1);
    }

    pub fn set_shunt(&mut self, range: CurrentRange) {
        self.current_range = range;
        self.hw.set_current_range(range);
    }

    pub fn calc_range_i(&mut self) {
        let mut range = 0usize;
        for index in 0..4 {
            if self.current_set > self.eeprom.imax(index) {
                range = (range + 1).min(3);
            }
        }
        self.current_range = CurrentRange::from_index(range);
    }

    pub fn set_level_dac(&mut self) {
        if self.scale.dac_lsb_u[0] == 0.0 || self.scale.dac_lsb_i[0] == 0.0 {
            self.init_scales();
        }
        self.calc_range_i();
        if Some(self.current_range) != self.old_current_range {
            self.dac_raw_i = 0;
            self.hw.set_current_dac_raw(0);
            self.hw.delay_ms(4);
            self.set_shunt(self.current_range);
        }
        self.old_current_range = Some(self.current_range);
        self.auto_current_range = self.current_range;

        let current_index = self.current_range as usize;
        self.dac_raw_i = self.quantize_dac(
            (self.current_set * self.current_mod) / self.scale.dac_lsb_i[current_index],
            self.eeprom.dac_i_offsets[current_index],
        );
        self.hw.set_current_dac_raw(self.dac_raw_i);

        let voltage_range = if self.voltage_set > self.scale.switchpoint {
            VoltageRange::UHigh
        } else {
            VoltageRange::ULow
        };
        self.voltage_range = voltage_range;
        if Some(voltage_range) != self.old_voltage_range {
            self.voltage_mod = 1.0;
            self.dac_raw_u_on = 0;
            self.dac_raw_u_off = 0;
            self.hw.set_voltage_dac_raw(0);
            self.hw.set_voltage_dac_off_raw(0);
            self.hw.delay_ms(4);
            self.hw.set_voltage_range(voltage_range);
        }
        self.old_voltage_range = Some(voltage_range);

        let voltage_index = voltage_range as usize;
        self.dac_raw_u_on = self.quantize_dac(
            (self.voltage_set * self.voltage_mod) / self.scale.dac_lsb_u[voltage_index],
            self.eeprom.dac_u_offsets[voltage_index],
        );
        self.hw.set_voltage_dac_raw(self.dac_raw_u_on);

        let off_raw = if self.pw_off_time_ms > 0 && self.ripple_percent > 0.0 {
            (u32::from(self.dac_raw_u_on) * (100 - self.ripple_percent as i32).max(0) as u32 / 100)
                as u16
        } else {
            self.dac_raw_u_on
        };
        self.dac_raw_u_off = off_raw;
        self.hw.set_voltage_dac_off_raw(off_raw);
    }

    pub(super) fn quantize_dac(&self, raw_without_offset: Float, offset: i16) -> u16 {
        (raw_without_offset + 0.5 + Float::from(offset)).clamp(0.0, self.scale.dac_max as Float)
            as u16
    }

    pub fn get_voltage(&mut self) -> Float {
        let adc = if self.scale.adc16_present {
            self.hw.read_adc16_voltage()
        } else {
            self.hw.read_adc10(3) as u16
        };
        let raw = adc as i32 + self.eeprom.adc_u_offsets[self.voltage_range as usize] as i32;
        let value = raw as Float * self.scale.adc_lsb_u[self.voltage_range as usize];
        self.measured_voltage = value;
        value
    }

    pub fn get_input_voltage(&mut self) {
        self.input_voltage = self.hw.read_adc10(5) as Float * self.eeprom.uref() * 0.01855;
    }

    pub fn get_current(&mut self) -> Float {
        let adc = if self.scale.adc16_present {
            self.hw.read_adc16_current()
        } else {
            self.hw.read_adc10(4) as u16
        };
        let raw = adc as i32 + self.eeprom.adc_i_offsets[self.current_range as usize] as i32;
        let value = raw as Float * self.scale.adc_lsb_i[self.current_range as usize];
        self.measured_current = value;
        self.measured_power = self.measured_voltage * self.measured_current;
        value
    }

    pub fn inc_fac_i(&mut self) {
        self.inc_coarse_div = 100.0;
        self.inc_fine_div = if self.current_set >= 1.0 {
            1_000.0
        } else {
            10_000.0
        };
    }

    pub fn inc_fac_u(&mut self) {
        self.inc_coarse_div = 10.0;
        self.inc_fine_div = if self.voltage_set >= 1.0 {
            100.0
        } else {
            1_000.0
        };
    }

    pub fn round_inc_param(&mut self) {
        if self.incr_fine {
            return;
        }

        match self.panel_modify {
            Modify::Volt => {
                self.voltage_set =
                    Self::round_to_increment_divisor(self.voltage_set, self.inc_coarse_div);
            }
            Modify::Ampere => {
                self.current_set =
                    Self::round_to_increment_divisor(self.current_set, self.inc_coarse_div);
            }
            _ => {}
        }
        self.first_turn = false;
    }

    pub fn set_acc_param(&mut self) {
        let divisor = if self.incr_fine {
            self.inc_fine_div
        } else {
            self.inc_coarse_div
        };
        let delta = self.incr_acc_float / divisor;

        match self.panel_modify {
            Modify::Volt => {
                self.voltage_set += delta;
                self.voltage_mod = 1.0;
            }
            Modify::Ampere => {
                self.current_set += delta;
                self.current_mod = 1.0;
            }
            _ => {}
        }
    }

    pub fn apply_encoder_delta(&mut self, raw_delta: i32) -> bool {
        self.incr_diff = self.incr_diff.saturating_add(raw_delta);
        let inc_rast = self.inc_rast.max(1);
        if self.incr_diff.abs() < inc_rast {
            return false;
        }

        let scaled_delta = self.incr_diff / inc_rast;
        self.incr_diff = 0;
        let accelerated_delta = Self::accelerated_encoder_delta(scaled_delta);
        self.incr_acc_float = accelerated_delta as Float;
        self.button_number = 4;

        match self.panel_modify {
            Modify::Volt => {
                if self.first_turn {
                    self.inc_fac_u();
                    self.round_inc_param();
                    self.ser_prompt(ErrorCode::UserReq);
                }
                self.set_acc_param();
                self.check_limits();
                self.sub_channel = 0;
                self.write_param_ser(self.voltage_set);
            }
            Modify::Ampere => {
                self.calc_range_i();
                if self.first_turn {
                    self.inc_fac_i();
                    self.round_inc_param();
                    self.ser_prompt(ErrorCode::UserReq);
                }
                self.set_acc_param();
                self.check_limits();
                self.sub_channel = 1;
                self.write_param_ser(self.current_set);
            }
            Modify::TOn => {
                self.mark_first_encoder_turn();
                self.pw_on_time_ms =
                    Self::add_signed_u16(self.pw_on_time_ms, accelerated_delta * 2);
                self.check_limits();
                self.sub_channel = 27;
                self.write_param_int_ser(i32::from(self.pw_on_time_ms));
            }
            Modify::TOff => {
                self.mark_first_encoder_turn();
                self.pw_off_time_ms =
                    Self::add_signed_u16(self.pw_off_time_ms, accelerated_delta * 2);
                self.check_limits();
                self.sub_channel = 28;
                self.write_param_int_ser(i32::from(self.pw_off_time_ms));
            }
            Modify::Ripple => {
                self.mark_first_encoder_turn();
                self.ripple_percent += accelerated_delta as Float;
                self.check_limits();
                self.sub_channel = 29;
                self.write_param_int_ser(self.ripple_percent as i32);
            }
            Modify::TrackCh => {
                self.track_channel = Self::pascal_add_byte(self.track_channel, accelerated_delta);
                self.check_limits();
            }
            Modify::CapMenu | Modify::PwrMenu => {}
        }

        self.set_level_dac();
        true
    }

    pub(super) fn mark_first_encoder_turn(&mut self) {
        if self.first_turn {
            self.button_number = 4;
            self.ser_prompt(ErrorCode::UserReq);
            self.first_turn = false;
        }
    }

    pub(super) fn accelerated_encoder_delta(scaled_delta: i32) -> i32 {
        let sign = scaled_delta.signum();
        let index = (scaled_delta.unsigned_abs() as usize).min(INCR_ACC_ARRAY.len() - 1);
        sign * INCR_ACC_ARRAY[index]
    }

    pub(super) fn round_to_increment_divisor(value: Float, divisor: Float) -> Float {
        (value * divisor).round() / divisor
    }

    pub(super) fn add_signed_u16(value: u16, delta: i32) -> u16 {
        let adjusted = i32::from(value).saturating_add(delta);
        adjusted.clamp(0, i32::from(u16::MAX)) as u16
    }

    pub(super) fn pascal_add_byte(value: u8, delta: i32) -> u8 {
        value.wrapping_add(delta as u8)
    }

    pub fn ser_crlf(&mut self) {
        self.hw.serial_write("\r\n");
    }

    pub fn write_ch_prefix(&mut self) {
        self.hw
            .serial_write(&format!("{}:{}=", self.main_channel, self.sub_channel));
    }

    pub fn write_ser_inp(&mut self) {
        self.hw.serial_write(&self.ser_input);
        self.ser_crlf();
    }

    pub fn ser_prompt(&mut self, err: ErrorCode) {
        let frame = self.status_frame(err);
        self.hw.serial_write(&frame);
    }

    pub fn status_frame(&mut self, err: ErrorCode) -> String {
        self.sub_channel = ERR_SUB_CH;
        let mut status = self.status.as_byte() & 0xf0;
        if err == ErrorCode::UserReq {
            status |= (self.button_number & 0x0f) | 0x40;
        } else if self.faults.any() || err == ErrorCode::NoErr {
            status |= self.faults.bits();
        } else {
            status |= err as u8;
            self.err_count = self.err_count.saturating_add(1);
        }

        let mut frame = format!("{}:{}={}", self.main_channel, ERR_SUB_CH, status);
        if self.faults.any() {
            for (flag, label) in [
                (self.faults.over_power, FAULT_STR_ARR[0]),
                (self.faults.fuse_blown, FAULT_STR_ARR[1]),
                (self.faults.over_voltage, FAULT_STR_ARR[2]),
                (self.faults.over_temp, FAULT_STR_ARR[3]),
            ] {
                if flag {
                    frame.push(' ');
                    frame.push_str(label);
                }
            }
        } else {
            let index = (err as usize).min(ERR_STR_ARR.len().saturating_sub(1));
            frame.push(' ');
            frame.push_str(ERR_STR_ARR[index]);
            if self.status.overload_flag {
                frame.push_str(" [ICONST]");
            }
        }
        frame.push_str("\r\n");
        frame
    }

    pub fn param_to_str(&self, value: Float) -> String {
        format!("{value:.3}")
    }

    pub fn send_track_cmd(&mut self) {
        if self.track_channel == 255 {
            return;
        }

        self.hw.serial_write(&format!(
            "{}:0={}!\r\n",
            self.track_channel,
            self.param_to_str(self.voltage_set)
        ));
        self.hw.serial_write(&format!(
            "{}:1={}!\r\n",
            self.track_channel,
            self.param_to_str(self.current_set)
        ));
    }

    pub fn set_cursor(&mut self, _full_cursor: bool) {}

    pub fn ist_leistung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(0, &format!("P {:>6.2}", self.measured_power));
    }

    pub fn cap_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("Ah {:>5.2}", self.capacity_mah));
    }

    pub fn spannung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(0, &format!("U {:>6.3}", self.voltage_set));
    }

    pub fn ist_spannung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(0, &format!("U {:>6.3}", self.measured_voltage));
    }

    pub fn soll_spannung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(0, &format!("Us{:>6.3}", self.voltage_set));
    }

    pub fn prefix_i(&self, ma_display: bool) -> &'static str {
        if ma_display {
            "mA"
        } else {
            "A"
        }
    }

    pub fn param_str_on_lcd_lower(&mut self) {
        self.hw.lcd_write_line(1, &self.param_str);
    }

    pub fn faults_on_lcd(&mut self) {
        // The lower LCD row showed compact fault mnemonics; in the original
        // firmware the overload bit also doubled as a current-limit indicator.
        if self.status.overload_flag {
            self.hw.lcd_write_line(1, FAULT_STR_ARR[0]);
        }
    }

    pub fn strom_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("I {:>6.3}", self.current_set));
    }

    pub fn ist_strom_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("I {:>6.3}", self.measured_current));
    }

    pub fn soll_strom_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("Is{:>6.3}", self.current_set));
    }

    pub fn integer_on_lcd(&mut self, value: i32) {
        self.hw.lcd_write_line(1, &format!("{value:>8}"));
    }

    pub fn options_on_lcd(&mut self) {
        self.hw.lcd_write_line(1, "OPT");
    }

    pub fn werte_on_lcd(&mut self) {
        // The default panel page showed measured U/I. In ripple mode Pascal
        // alternated the voltage readout between the main setpoint and the
        // reduced off-time value while not in current limiting.
        self.ist_spannung_on_lcd();
        self.ist_strom_on_lcd();
    }

    pub fn write_param_ser(&mut self, value: Float) {
        self.write_ch_prefix();
        self.hw.serial_write(&self.param_to_str(value));
        self.ser_crlf();
    }

    pub fn write_param_int_ser(&mut self, value: i32) {
        self.write_ch_prefix();
        self.hw.serial_write(&value.to_string());
        self.ser_crlf();
    }

    pub fn check_limits(&mut self) -> ErrorCode {
        if self.locked {
            return ErrorCode::LockedErr;
        }
        let mut err = ErrorCode::NoErr;
        if self.voltage_set < 0.0 {
            self.voltage_set = 0.0;
            err = ErrorCode::ParamErr;
        }
        if self.voltage_set > self.eeprom.umax() {
            self.voltage_set = self.eeprom.umax();
            err = ErrorCode::ParamErr;
        }
        if self.current_set < 0.0 {
            self.current_set = 0.0;
            err = ErrorCode::ParamErr;
        }
        if self.current_set > self.eeprom.imax(3) {
            self.current_set = self.eeprom.imax(3);
            err = ErrorCode::ParamErr;
        }
        if self.pw_on_time_ms < 2 {
            self.pw_on_time_ms = 2;
            err = ErrorCode::ParamErr;
        }
        if self.ripple_percent < 0.0 {
            self.ripple_percent = 0.0;
            err = ErrorCode::ParamErr;
        }
        if self.ripple_percent > 100.0 {
            self.ripple_percent = 100.0;
            err = ErrorCode::ParamErr;
        }
        if self.track_channel >= 128 {
            self.track_channel = 255;
        } else if self.track_channel > 7 {
            self.track_channel = 7;
        }
        self.no_toggle = self.ripple_percent == 0.0;
        self.ripple_voltage = if self.no_toggle {
            0.0
        } else {
            self.ripple_percent * self.voltage_set / 100.0
        };
        err
    }

    pub fn switch_relais(&mut self) {
        if self.faults.over_temp || self.faults.over_voltage {
            return;
        }

        if self.relay_state_high != self.old_relay_state_high {
            self.hw.set_input_relay_high(self.relay_state_high);
            self.hw.delay_ms(10);
        }
        self.old_relay_state_high = self.relay_state_high;
    }

    pub fn fault_check(&mut self) {
        if self.scale.dcp_present {
            if self.temperature_timer == 0 {
                self.temperature_timer = 20;
                self.get_lm75_temp();
            }
            self.temperature_timer = self.temperature_timer.saturating_sub(1);
        } else {
            self.temperature_c = Some(0.0);
        }

        if self.temperature_c.unwrap_or(0.0) > 70.0 {
            self.faults.over_temp = true;
            self.hw.set_input_relay_high(false);
        } else {
            if self.faults.over_temp {
                self.relay_state_high = false;
                self.old_relay_state_high = true;
            }
            self.faults.over_temp = false;
        }

        self.get_input_voltage();
        let allowed_output_voltage = self.input_voltage - 2.0;
        if self.measured_voltage > allowed_output_voltage {
            self.faults.over_voltage = true;
            self.hw.set_input_relay_high(false);
        } else {
            if self.faults.over_voltage {
                self.relay_state_high = false;
                self.old_relay_state_high = true;
            }
            self.faults.over_voltage = false;
        }

        self.faults.fuse_blown = allowed_output_voltage < 5.0;
        if self.faults.fuse_blown {
            self.faults.over_voltage = false;
        }

        if self.faults.any() {
            self.status.overload_flag = true;
        }
    }

    pub fn chores(&mut self) {
        let previous_current = self.measured_current;
        let current = self.get_current();
        self.measured_current = (previous_current * 7.0 + current) / 8.0;
        let previous_voltage = self.measured_voltage;
        let voltage = self.get_voltage();
        self.measured_voltage = (previous_voltage * 7.0 + voltage) / 8.0;
        self.measured_power = self.measured_voltage * self.measured_current;

        let current_limit_sense = self.hw.current_limit_sense();
        let relay_voltage = if current_limit_sense {
            self.voltage_set
        } else {
            self.measured_voltage
        };

        if relay_voltage > self.relay_voltage_high && !self.status.overload_flag {
            self.relay_state_high = true;
        }
        if relay_voltage < self.relay_voltage_low {
            self.relay_state_high = false;
        }

        self.fault_check();
        if !self.faults.any() {
            self.status.overload_flag = !current_limit_sense;
        }
        if self.fault_timer == 0 {
            if self.faults.any() {
                self.ser_prompt(ErrorCode::OvlErr);
            }
            self.fault_timer = 10;
        }
        self.fault_timer = self.fault_timer.saturating_sub(1);

        self.switch_relais();
        self.no_toggle = self.ripple_percent == 0.0;
        self.werte_on_lcd();
    }

    pub fn check_ser(&mut self) {
        while let Some(input) = self.hw.serial_read_timeout(20) {
            if (' '..='~').contains(&input) {
                self.ser_input.push(input);
            }
            if input == '\u{8}' {
                self.ser_input.pop();
            }
            if input == '\r' {
                self.chores();
                self.parse_serial_command();
                self.ser_input.clear();
            }
        }
        self.chores();
    }

    pub(super) fn parse_serial_command(&mut self) {
        let mut command = self.ser_input.trim().to_string();
        if command.is_empty() {
            return;
        }

        let verbose = command.ends_with('?') || command.ends_with('!');
        if verbose {
            command.pop();
        }

        let (address, body) = match command.split_once(':') {
            Some((address, body)) => (address, body),
            None => {
                self.ser_prompt(ErrorCode::SyntaxErr);
                return;
            }
        };
        let Ok(address) = address.parse::<u8>() else {
            self.ser_prompt(ErrorCode::SyntaxErr);
            return;
        };
        if address != self.main_channel && address != 255 {
            return;
        }

        let (selector, value) = match body.split_once('=') {
            Some((selector, value)) => (selector, Some(value.trim())),
            None => (body, None),
        };
        let Some(sub_channel) = Self::parse_sub_channel_selector(selector.trim()) else {
            self.ser_prompt(ErrorCode::SyntaxErr);
            return;
        };
        self.sub_channel = sub_channel;

        let result = if let Some(value) = value {
            self.apply_serial_value(sub_channel, value)
        } else {
            Ok(())
        };
        let parsed_value = result.is_ok() && value.is_some();

        let err = match result {
            Ok(()) => self.check_limits(),
            Err(err) => err,
        };
        if parsed_value && err != ErrorCode::LockedErr {
            self.set_level_dac();
            self.send_track_cmd();
        }

        if value.is_none() || verbose || err != ErrorCode::NoErr {
            if err == ErrorCode::NoErr {
                self.write_serial_value(sub_channel);
            } else {
                self.ser_prompt(err);
            }
        }
    }

    pub(super) fn parse_sub_channel_selector(selector: &str) -> Option<u8> {
        if let Ok(sub_channel) = selector.parse::<u8>() {
            return Some(sub_channel);
        }
        let selector = selector.to_ascii_uppercase();
        CMD_STR_ARR
            .iter()
            .position(|command| *command == selector)
            .map(|index| CMD2_SUB_CH_ARR[index])
    }

    pub(super) fn apply_serial_value(&mut self, sub_channel: u8, value: &str) -> Result<(), ErrorCode> {
        match sub_channel {
            0 => self.voltage_set = value.parse().map_err(|_| ErrorCode::ParamErr)?,
            1 => self.current_set = value.parse().map_err(|_| ErrorCode::ParamErr)?,
            20 => {
                self.voltage_mod = value.parse::<Float>().map_err(|_| ErrorCode::ParamErr)? / 100.0
            }
            21 => {
                self.current_mod = value.parse::<Float>().map_err(|_| ErrorCode::ParamErr)? / 100.0
            }
            27 => self.pw_on_time_ms = value.parse().map_err(|_| ErrorCode::ParamErr)?,
            28 => self.pw_off_time_ms = value.parse().map_err(|_| ErrorCode::ParamErr)?,
            29 => self.ripple_percent = value.parse().map_err(|_| ErrorCode::ParamErr)?,
            80 => {
                self.panel_modify = match value.parse::<u8>().map_err(|_| ErrorCode::ParamErr)? {
                    0 => Modify::Ampere,
                    1 => Modify::Volt,
                    2 => Modify::Ripple,
                    3 => Modify::TOn,
                    4 => Modify::TOff,
                    5 => Modify::TrackCh,
                    6 => Modify::CapMenu,
                    7 => Modify::PwrMenu,
                    _ => return Err(ErrorCode::ParamErr),
                };
            }
            253 => self.main_channel = value.parse().map_err(|_| ErrorCode::ParamErr)?,
            _ => return Err(ErrorCode::ParamErr),
        }
        Ok(())
    }

    pub(super) fn write_serial_value(&mut self, sub_channel: u8) {
        match sub_channel {
            0 => self.write_param_ser(self.voltage_set),
            1 => self.write_param_ser(self.current_set),
            7 => self.write_param_ser(self.capacity_mah),
            8 => self.write_param_ser(self.capacity_mwh),
            10 => self.write_param_ser(self.measured_voltage),
            11 => self.write_param_ser(self.measured_current),
            18 => self.write_param_ser(self.measured_power),
            27 => self.write_param_int_ser(i32::from(self.pw_on_time_ms)),
            28 => self.write_param_int_ser(i32::from(self.pw_off_time_ms)),
            29 => self.write_param_int_ser(self.ripple_percent as i32),
            80 => self.write_param_int_ser(self.panel_modify as i32),
            233 => {
                self.get_lm75_temp();
                self.write_param_ser(self.temperature_c.unwrap_or(0.0));
            }
            253 => self.write_param_int_ser(i32::from(self.main_channel)),
            254 => {
                self.write_ch_prefix();
                self.hw.serial_write(VERS1_STR);
                self.ser_crlf();
            }
            255 => self.ser_prompt(ErrorCode::NoErr),
            _ => self.ser_prompt(ErrorCode::ParamErr),
        }
    }

    pub fn check_delay(&mut self, _delay_ms: u8) {
        // Delay loops called CheckSer repeatedly so long waits did not starve
        // the command parser, LCD refresh, or periodic measurement updates.
        for _ in 0.._delay_ms {
            self.check_ser();
        }
    }

    pub fn on_tick_timer(&mut self) {
        if self.ripple_percent > 0.0 {
            self.capacity_mah = 0.0;
            self.capacity_mwh = 0.0;
            return;
        }

        if self.measured_current < 0.00001 {
            self.measured_current = 0.0;
        }
        self.capacity_mah += self.measured_current / (3600.0 * 5.0);
        self.capacity_mwh += self.measured_current * self.measured_voltage / (3600.0 * 5.0);
    }

    pub fn init_all(&mut self) {
        self.init_scales();
        self.status = RuntimeStatus::default();
        self.faults = FaultFlags::default();
        self.err_count = 0;
        self.button_number = 0;
        self.current_mod = 1.0;
        self.voltage_mod = 1.0;
        self.voltage_set = 0.0;
        self.current_set = self.eeprom.init_amp();
        self.old_voltage_range = None;
        self.old_current_range = None;
        self.calc_range_i();
        self.set_level_dac();
        self.panel_modify = Modify::Volt;
        self.sub_channel = 254;
        self.write_ch_prefix();
        self.hw.serial_write(VERS1_STR);
        self.ser_crlf();
        self.output_enabled = true;
        self.hw.set_output_enabled(self.output_enabled);
        self.hw.delay_ms(10);
        self.voltage_set = self.eeprom.init_volt();
        self.set_level_dac();
        self.old_relay_state_high = true;
        self.relay_state_high = false;
        self.switch_relais();
        self.err_count = 0;
        self.hw.delay_ms(200);
        self.fault_check();
        if self.faults.fuse_blown {
            self.ser_prompt(ErrorCode::FuseErr);
        }
        self.capacity_mah = 0.0;
        self.capacity_mwh = 0.0;
        self.no_toggle = self.ripple_percent == 0.0;
    }
}
