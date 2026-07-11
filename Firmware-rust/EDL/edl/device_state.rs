use super::*;

#[derive(Debug, Clone)]
pub struct DeviceState<H> {
    pub hw: H,
    pub eeprom: EepromData,
    pub scale: ScaleState,
    pub status: RuntimeStatus,
    pub faults: ProtectionFlags,
    pub mode: Mode,
    pub modify: Modify,
    pub output_enabled: bool,
    pub current_set: Float,
    pub current_scale_factor: Float,
    pub power_set: Float,
    pub voltage_cutoff: Float,
    pub resistance_set: Float,
    pub i_percent: i32,
    pub pw_on_time_ms: i32,
    pub pw_off_time_ms: i32,
    pub no_toggle: bool,
    pub shunt_range: u8,
    pub shunt_select: u8,
    pub old_shunt_select: u8,
    pub track_channel: u8,
    pub measured_voltage: Float,
    pub measured_current: Float,
    pub measured_power: Float,
    pub power_on: Float,
    pub power_off: Float,
    pub psoa: Float,
    pub capacity_ah: Float,
    pub capacity_wh: Float,
    pub internal_resistance: Float,
    pub temperature_c: Option<Float>,
    pub current_voltage: Float,
    pub current_amp: Float,
    pub trigger_mask: u8,
    pub trig_on_sema: bool,
    pub trig_off_sema: bool,
    pub fault_timer_byte: u8,
    pub temperature_timer: u8,
    pub service_elapsed_ms: u32,
    pub integration_elapsed_ms: u32,
    pub err_count: i32,
    pub button_number: u8,
    pub slave_channel: u8,
    pub sub_channel: u8,
    pub dac_raw_on: u16,
    pub dac_raw_off: u16,
    pub dac_raw_active: u16,
    pub serial_input: String,
    pub completed_command: Option<String>,
    pub last_measurement: MeasurementSnapshot,
    pub incr_fine: bool,
    pub first_turn: bool,
    pub incr_acc_float: Float,
    pub inc_fine_div: Float,
    pub inc_coarse_div: Float,
}

impl<H: EdlHardware> DeviceState<H> {
    pub fn new(hw: H) -> Self {
        Self::with_eeprom(hw, EepromData::default())
    }

    pub fn with_eeprom(hw: H, eeprom: EepromData) -> Self {
        let mut state = Self {
            hw,
            eeprom,
            scale: ScaleState::default(),
            status: RuntimeStatus::default(),
            faults: ProtectionFlags::default(),
            mode: Mode::OutputOff,
            modify: Modify::LowerMainMenu,
            output_enabled: false,
            current_set: 0.0,
            current_scale_factor: 1.0,
            power_set: 0.0,
            voltage_cutoff: 0.0,
            resistance_set: 100.0,
            i_percent: 100,
            pw_on_time_ms: 1,
            pw_off_time_ms: 0,
            no_toggle: true,
            shunt_range: AUTO_SHUNT_RANGE,
            shunt_select: 0,
            old_shunt_select: u8::MAX,
            track_channel: 255,
            measured_voltage: 0.0,
            measured_current: 0.0,
            measured_power: 0.0,
            power_on: 0.0,
            power_off: 0.0,
            psoa: 0.0,
            capacity_ah: 0.0,
            capacity_wh: 0.0,
            internal_resistance: 0.0,
            temperature_c: None,
            current_voltage: 0.0,
            current_amp: 0.0,
            trigger_mask: 0,
            trig_on_sema: false,
            trig_off_sema: false,
            fault_timer_byte: 0,
            temperature_timer: 0,
            service_elapsed_ms: 0,
            integration_elapsed_ms: 0,
            err_count: 0,
            button_number: 0,
            slave_channel: 0,
            sub_channel: 0,
            dac_raw_on: 0,
            dac_raw_off: 0,
            dac_raw_active: 0,
            serial_input: String::new(),
            completed_command: None,
            last_measurement: MeasurementSnapshot::default(),
            incr_fine: false,
            first_turn: true,
            incr_acc_float: 0.0,
            inc_fine_div: 1_000.0,
            inc_coarse_div: 10.0,
        };
        state.init_scales();
        state
    }

    pub fn set_lm75_temp(&mut self) {
        let threshold_c = self.eeprom.init_fan_on_temp();
        if self.lm75_intern_enabled() {
            self.set_one_lm75_temp(LM75_INTERNAL_ADDRESS, threshold_c);
        }
        if self.lm75_extern_enabled() {
            self.set_one_lm75_temp(LM75_EXTERNAL_ADDRESS, threshold_c);
        }
    }

    fn set_one_lm75_temp(&mut self, address: u8, threshold_c: Float) {
        self.hw.lm75_write(
            address,
            LM75_CONFIGURATION_REGISTER,
            &[LM75_INVERTED_OUTPUT_CONFIGURATION],
        );

        let overtemperature = Self::lm75_temperature_bytes(threshold_c);
        self.hw
            .lm75_write(address, LM75_OVERTEMPERATURE_REGISTER, &overtemperature);

        let hysteresis = Self::lm75_temperature_bytes(threshold_c - LM75_HYSTERESIS_C);
        self.hw
            .lm75_write(address, LM75_HYSTERESIS_REGISTER, &hysteresis);
        self.hw.lm75_write(address, LM75_TEMPERATURE_REGISTER, &[]);
    }

    fn lm75_temperature_bytes(temperature_c: Float) -> [u8; 2] {
        let half_degrees = (temperature_c * 2.0) as i16;
        (half_degrees << 7).to_be_bytes()
    }

    pub fn get_one_lm75_temp(&mut self) -> Option<Float> {
        self.hw.read_temp_c()
    }

    pub fn get_lm75_temp(&mut self) {
        self.temperature_c = self.get_one_lm75_temp();
    }

    pub fn init_scale_u(&mut self) {
        if self.mode.is_low_voltage() {
            self.scale.divider_u = self.eeprom.init_low_divider_u();
            self.scale.adc_u_offset = self.eeprom.adc_u_offsets[0];
            self.scale.adc16_lsb_u =
                self.eeprom.adc_u_scales[0] * self.eeprom.uref() * self.scale.divider_u / 65_535.0;
        } else {
            self.scale.divider_u = self.eeprom.init_hi_divider_u();
            self.scale.adc_u_offset = self.eeprom.adc_u_offsets[1];
            self.scale.adc16_lsb_u =
                self.eeprom.adc_u_scales[1] * self.eeprom.uref() * self.scale.divider_u / 65_535.0;
        }
    }

    pub fn init_scales(&mut self) {
        self.trigger_mask = self.eeprom.trig_mask;
        self.scale.options = self.eeprom.init_options();
        self.scale.dac_kind = match self.scale.options & DAC_TYPE_MASK {
            1 => DacKind::Ad5452,
            2 => DacKind::Dac8501,
            3 => DacKind::Dac8811,
            _ => DacKind::Ltc8043,
        };
        self.scale.dac_max = if matches!(self.scale.dac_kind, DacKind::Dac8811) {
            65_535
        } else {
            4_095
        };
        self.init_scale_u();

        let uref = self.eeprom.uref();
        let gain_i = self.eeprom.init_gain_i();
        let dac_max = self.scale.dac_max as Float;
        for index in 0..4 {
            let rsense = self.eeprom.rsense(index);
            self.scale.dac_lsb_i[index] =
                (uref / rsense) / (dac_max * self.eeprom.dac_i_scales[index] * gain_i);
            // The Pascal firmware intentionally reused DACIscales here because
            // the EEPROM DACRscales were known bad.
            self.scale.dac_lsb_r[index] =
                gain_i * rsense * dac_max * self.eeprom.dac_i_scales[index];
            self.scale.adc16_lsb_i[index] =
                (self.eeprom.adc_i_scales[index] * uref / rsense) / 65_535.0 / gain_i;
        }

        self.pw_on_time_ms = self.eeprom.init_ton();
        self.pw_off_time_ms = self.eeprom.init_toff();
        self.i_percent = self.eeprom.init_i_percent();
        self.scale.dc_ohm_min = self.eeprom.rsense(3) * self.scale.divider_u * gain_i * 1.1;
        self.scale.dc_ohm_max = self.eeprom.rsense(0) * self.scale.divider_u * gain_i * 100.0;
        self.set_lm75_temp();
    }

    pub fn ser_crlf(&mut self) {
        self.hw.serial_write("\r\n");
    }

    pub fn write_ch_prefix(&mut self) {
        self.hw
            .serial_write(&format!("#{}:{}=", self.slave_channel, self.sub_channel));
    }

    pub fn write_ser_inp(&mut self) {
        self.hw.serial_write(&self.serial_input);
        self.ser_crlf();
    }

    pub fn set_shunt(&mut self, shunt: u8) {
        self.shunt_select = shunt.min(SHUNT_D);
        self.hw.set_shunt(self.shunt_select);
    }

    pub fn calc_range_i(&mut self) -> u8 {
        let mut shunt = 0u8;
        for index in 0..4 {
            if self.current_set > self.eeprom.imax(index) {
                shunt = (shunt + 1).min(SHUNT_D);
            }
        }
        shunt
    }

    pub fn calc_range_r(&mut self) -> u8 {
        let mut shunt = 0u8;
        for index in 0..4 {
            if self.resistance_set
                < (self.eeprom.rsense(index)
                    * self.eeprom.init_gain_i()
                    * self.scale.divider_u
                    * 1.1)
            {
                shunt = (shunt + 1).min(SHUNT_D);
            }
        }
        shunt
    }

    pub fn get_voltage(&mut self, on_phase: bool) {
        self.measured_voltage = self.read_voltage_phase(on_phase);
    }

    pub fn get_current(&mut self, on_phase: bool) {
        self.measured_current = self.read_current_phase(on_phase);
    }

    pub fn read_voltage_phase(&mut self, on_phase: bool) -> Float {
        let phase = self.use_on_phase(on_phase);
        let raw = self.hw.read_voltage_adc16(phase) as i32 + i32::from(self.scale.adc_u_offset);
        raw as Float * self.scale.adc16_lsb_u
    }

    pub fn read_current_phase(&mut self, on_phase: bool) -> Float {
        if !self.output_enabled {
            return 0.0;
        }
        let phase = self.use_on_phase(on_phase);
        let index = self.shunt_select as usize;
        let raw =
            self.hw.read_current_adc16(phase) as i32 + i32::from(self.eeprom.adc_i_offsets[index]);
        raw as Float * self.scale.adc16_lsb_i[index]
    }

    pub fn get_voltage10(&mut self) {
        self.measured_voltage = self.hw.read_voltage_adc10() as Float * 0.01;
    }

    pub fn get_current10(&mut self) {
        self.measured_current = self.hw.read_current_adc10() as Float * 0.01;
    }

    pub fn get_ri(&mut self) -> bool {
        if self.no_toggle || !self.mode.is_current() {
            return false;
        }
        let on_voltage = self.read_voltage_phase(true);
        let off_voltage = self.read_voltage_phase(false);
        let on_current = self.read_current_phase(true);
        let off_current = self.read_current_phase(false);
        let current_delta = on_current - off_current;
        if current_delta <= 0.0 {
            return false;
        }
        self.internal_resistance = (off_voltage - on_voltage) / current_delta;
        true
    }

    pub fn set_level_dac_i(&mut self) {
        self.init_scale_u();
        let mut shunt = self.calc_range_i();
        if self.shunt_range <= SHUNT_D && self.shunt_range >= shunt {
            shunt = self.shunt_range;
        }
        self.apply_shunt_change(shunt);

        let index = self.shunt_select as usize;
        self.dac_raw_on = self.quantize_dac(
            (self.current_set * self.current_scale_factor) / self.scale.dac_lsb_i[index],
            self.eeprom.dac_i_offsets[index],
        );
        let off_scale = self.i_percent as Float / 100.0;
        self.dac_raw_off = self.quantize_dac(
            (self.current_set * self.current_scale_factor * off_scale)
                / self.scale.dac_lsb_i[index],
            self.eeprom.dac_i_offsets[index],
        );
        self.update_output_drive();
    }

    pub fn set_level_dac_r(&mut self) {
        self.init_scale_u();
        let shunt = self.calc_range_r();
        self.apply_shunt_change(shunt);

        let index = self.shunt_select as usize;
        self.dac_raw_on = self.quantize_dac(
            (self.scale.divider_u * self.scale.dac_lsb_r[index]) / self.resistance_set,
            self.eeprom.dac_i_offsets[index],
        );
        self.dac_raw_off = self.dac_raw_on;
        self.update_output_drive();
    }

    pub fn set_level_dac_p(&mut self) {
        let on_voltage = self.read_voltage_phase(true);
        if on_voltage > 0.0 {
            self.current_set = self.power_set / on_voltage;
        }
        self.check_limits();
        self.set_level_dac_i();
    }

    pub fn ser_prompt(&mut self, err: ErrorCode) {
        let frame = self.status_frame(err);
        self.hw.serial_write(&frame);
    }

    pub fn status_frame(&mut self, err: ErrorCode) -> String {
        self.sub_channel = ERR_SUBCH;
        let mut status = self.status.flag_bits();
        if err == ErrorCode::UserReq {
            status |= (self.button_number & 0x0f) | 0x40;
        } else if self.faults.any() || err == ErrorCode::NoErr {
            status |= self.faults.bits();
        } else {
            status |= err as u8;
            if err != ErrorCode::NoErr {
                self.err_count += 1;
            }
        }

        let mut frame = format!("#{}:{}={}", self.slave_channel, ERR_SUBCH, status);
        if self.faults.any() {
            for (flag, label) in [
                (self.faults.over_power, FAULT_STR_ARR[0]),
                (self.faults.fuse_blown, FAULT_STR_ARR[1]),
                (self.faults.over_voltage, FAULT_STR_ARR[2]),
                (self.faults.over_temp, FAULT_STR_ARR[3]),
                (self.faults.low_volt, FAULT_STR_ARR[4]),
            ] {
                if flag {
                    frame.push(' ');
                    frame.push_str(label);
                }
            }
        } else {
            frame.push(' ');
            frame.push_str(ERR_STR_ARR[(err as usize).min(ERR_STR_ARR.len() - 1)]);
        }
        frame.push_str("\r\n");
        frame
    }

    pub fn inc_fac_i(&mut self) {
        self.inc_coarse_div = 100.0;
        self.inc_fine_div = if self.current_set >= 1.0 {
            1_000.0
        } else {
            10_000.0
        };
    }

    pub fn inc_fac_r(&mut self) {
        self.inc_coarse_div = if self.resistance_set >= 1_000.0 {
            0.01
        } else if self.resistance_set >= 100.0 {
            0.1
        } else if self.resistance_set >= 10.0 {
            1.0
        } else {
            10.0
        };
        self.inc_fine_div = self.inc_coarse_div * 100.0;
    }

    pub fn inc_fac_p(&mut self) {
        self.inc_coarse_div = 10.0;
        self.inc_fine_div = if self.power_set >= 10.0 {
            100.0
        } else {
            1_000.0
        };
    }

    pub fn inc_fac_v(&mut self) {
        self.inc_coarse_div = 10.0;
        self.inc_fine_div = if self.voltage_cutoff >= 10.0 {
            100.0
        } else {
            1_000.0
        };
    }

    pub fn round_inc_param(&mut self) {
        if self.incr_fine {
            return;
        }

        match self.modify {
            Modify::LowerMainMenu if self.mode.is_current() => {
                self.current_set =
                    Self::round_to_increment_divisor(self.current_set, self.inc_coarse_div);
            }
            Modify::LowerMainMenu if self.mode.is_resistance() => {
                self.resistance_set =
                    Self::round_to_increment_divisor(self.resistance_set, self.inc_coarse_div);
            }
            Modify::LowerMainMenu if self.mode.is_power() => {
                self.power_set =
                    Self::round_to_increment_divisor(self.power_set, self.inc_coarse_div);
            }
            Modify::UpperMainMenu => {
                self.voltage_cutoff =
                    Self::round_to_increment_divisor(self.voltage_cutoff, self.inc_coarse_div);
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
        self.incr_acc_float /= divisor;

        match self.modify {
            Modify::LowerMainMenu if self.mode.is_current() => {
                self.current_set += self.incr_acc_float;
            }
            Modify::LowerMainMenu if self.mode.is_resistance() => {
                self.resistance_set += self.incr_acc_float;
            }
            Modify::LowerMainMenu if self.mode.is_power() => {
                self.power_set += self.incr_acc_float;
            }
            Modify::UpperMainMenu => {
                self.voltage_cutoff += self.incr_acc_float;
            }
            _ => {}
        }
    }

    fn round_to_increment_divisor(value: Float, divisor: Float) -> Float {
        (value * divisor).round() / divisor
    }

    pub fn param_to_str(&self, value: Float) -> String {
        format!("{value:.6}")
    }

    pub fn set_cursor(&mut self, _full_cursor: bool) {}

    pub fn spannung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(0, &format!("U {:>6.3}", self.voltage_cutoff));
    }

    pub fn ist_spannung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(0, &format!("U {:>6.3}", self.measured_voltage));
    }

    pub fn soll_spannung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(0, &format!("Us{:>6.3}", self.voltage_cutoff));
    }

    pub fn param_div_1000(&self, value: Float) -> Float {
        value / 1000.0
    }

    pub fn param_mul_1000(&self, value: Float) -> Float {
        value * 1000.0
    }

    pub fn prefix_r(&self) -> &'static str {
        "Ohm"
    }

    pub fn prefix_i(&self, ma_display: bool) -> &'static str {
        if ma_display {
            "mA"
        } else {
            "A"
        }
    }

    pub fn strom_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("I {:>6.3}", self.current_set));
    }

    pub fn widerstand_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("R {:>6.2}", self.resistance_set));
    }

    pub fn ist_strom_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("I {:>6.3}", self.measured_current));
    }

    pub fn soll_strom_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("Is{:>6.3}", self.current_set));
    }

    pub fn ist_leistung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("P {:>6.2}", self.measured_power));
    }

    pub fn soll_leistung_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("Ps{:>6.2}", self.power_set));
    }

    pub fn cap_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("Ah {:>5.2}", self.capacity_ah));
    }

    pub fn ri_on_lcd(&mut self) {
        self.hw
            .lcd_write_line(1, &format!("Ri {:>5.2}", self.internal_resistance));
    }

    pub fn werte_on_lcd(&mut self) {
        match self.mode {
            Mode::OutputOff | Mode::IHiVolt | Mode::ILoVolt => {
                self.ist_spannung_on_lcd();
                self.ist_strom_on_lcd();
            }
            Mode::RHiVolt | Mode::RLoVolt => {
                self.ist_spannung_on_lcd();
                self.widerstand_on_lcd();
            }
            Mode::PHiVolt | Mode::PLoVolt => {
                self.ist_spannung_on_lcd();
                self.ist_leistung_on_lcd();
            }
        }
    }

    pub fn write_param_ser(&mut self, value: Float) {
        self.hw.serial_write(&self.param_to_str(value));
    }

    pub fn write_param_int_ser(&mut self, value: i32) {
        self.hw.serial_write(&value.to_string());
    }

    pub fn check_limits(&mut self) -> ErrorCode {
        let mut err = ErrorCode::NoErr;
        self.no_toggle = false;

        if self.resistance_set < self.scale.dc_ohm_min {
            self.resistance_set = self.scale.dc_ohm_min;
            err = ErrorCode::ParamErr;
        }
        if self.resistance_set > self.scale.dc_ohm_max {
            self.resistance_set = self.scale.dc_ohm_max;
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
        if self.i_percent < 0 {
            self.i_percent = 0;
            err = ErrorCode::ParamErr;
        }
        if self.i_percent >= 100 {
            self.i_percent = 100;
            self.no_toggle = true;
            err = ErrorCode::ParamErr;
        }
        if self.pw_on_time_ms < 1 {
            self.pw_on_time_ms = 1;
            err = ErrorCode::ParamErr;
        }
        if self.pw_off_time_ms < 0 {
            self.pw_off_time_ms = 0;
            err = ErrorCode::ParamErr;
        }
        if self.pw_off_time_ms == 0 {
            self.no_toggle = true;
        }
        if self.track_channel > 127 {
            self.track_channel = 255;
        } else if self.track_channel > 7 {
            self.track_channel = 7;
        }
        if self.power_set > self.eeprom.pmax() {
            self.power_set = self.eeprom.pmax();
        }
        if self.power_set < 0.0 {
            self.power_set = 0.0;
            err = ErrorCode::ParamErr;
        }
        let max_voltage = self.active_voltage_limit();
        if self.voltage_cutoff > max_voltage {
            self.voltage_cutoff = max_voltage;
        }
        if self.voltage_cutoff < 0.0 {
            self.voltage_cutoff = 0.0;
            err = ErrorCode::ParamErr;
        }
        if self.mode.is_resistance() {
            self.no_toggle = true;
        }
        err
    }

    pub fn fault_check(&mut self) {
        self.poll_temperature();
        let on_voltage = self.read_voltage_phase(true);
        self.faults.over_temp = self.temperature_c.unwrap_or(0.0) > TEMPERATURE_MAX_C;
        self.faults.over_voltage = on_voltage > self.active_voltage_limit();
        self.faults.over_power = self.psoa > self.eeprom.pmax();
        if on_voltage < self.voltage_cutoff && self.voltage_cutoff > 0.0 {
            self.faults.low_volt = true;
            self.output_enabled = false;
        }
        self.status.overload_flag = self.faults.any();
        self.update_output_drive();
    }

    pub fn chores(&mut self) {
        self.service_cycle();
    }

    pub fn service_cycle(&mut self) {
        if self.mode.is_power() {
            self.set_level_dac_p();
        }

        let measurements = self.refresh_measurements();
        self.last_measurement = measurements;
        self.measured_voltage = measurements.voltage_on;
        self.measured_current = measurements.current_on;
        self.power_on = measurements.power_on;
        self.power_off = measurements.power_off;
        self.measured_power = measurements.power_avg;
        self.current_amp = measurements.current_on;
        self.current_voltage = measurements.voltage_on;

        self.psoa = match self.mode {
            Mode::OutputOff => 0.0,
            Mode::RHiVolt | Mode::RLoVolt => {
                if self.resistance_set > 0.0 {
                    measurements.voltage_on * measurements.voltage_on / self.resistance_set
                } else {
                    0.0
                }
            }
            Mode::PHiVolt | Mode::PLoVolt => self.eeprom.pmax(),
            Mode::IHiVolt | Mode::ILoVolt => measurements.voltage_on * self.current_set,
        };

        let _ = self.get_ri();
        self.fault_check();
        self.werte_on_lcd();
        self.emit_periodic_telemetry();
        self.emit_trigger_telemetry();
    }

    pub fn check_ser(&mut self) {
        self.service_step(20);
    }

    pub fn check_delay(&mut self, delay_ms: u8) {
        for _ in 0..delay_ms {
            self.service_step(20);
        }
    }

    pub fn service_step(&mut self, elapsed_ms: u32) {
        self.service_elapsed_ms += elapsed_ms;
        self.integration_elapsed_ms += elapsed_ms;

        while self.service_elapsed_ms >= SERVICE_INTERVAL_MS {
            self.service_elapsed_ms -= SERVICE_INTERVAL_MS;
            self.service_cycle();
        }

        while self.integration_elapsed_ms >= INTEGRATION_INTERVAL_MS {
            self.integration_elapsed_ms -= INTEGRATION_INTERVAL_MS;
            self.integrate_energy();
        }
    }

    pub fn init_all(&mut self) {
        self.mode = Mode::IHiVolt;
        self.status = RuntimeStatus::default();
        self.faults = ProtectionFlags::default();
        self.output_enabled = true;
        self.current_scale_factor = 1.0;
        self.current_set = 0.0;
        self.power_set = 0.0;
        self.voltage_cutoff = self.eeprom.init_volt();
        self.resistance_set = 100.0;
        self.shunt_range = AUTO_SHUNT_RANGE;
        self.old_shunt_select = u8::MAX;
        self.modify = Modify::LowerMainMenu;
        self.service_elapsed_ms = 0;
        self.integration_elapsed_ms = 0;
        self.fault_timer_byte = 0;
        self.temperature_timer = 0;
        self.trig_on_sema = false;
        self.trig_off_sema = false;
        self.serial_input.clear();
        self.completed_command = None;
        self.incr_fine = false;
        self.first_turn = true;
        self.incr_acc_float = 0.0;

        self.init_scales();
        self.set_level_dac_i();
        self.fault_check();

        self.current_scale_factor = 1.0;
        self.current_set = self.eeprom.init_amp();
        self.resistance_set = 100.0;
        self.i_percent = self.eeprom.init_i_percent();
        self.pw_on_time_ms = self.eeprom.init_ton();
        self.pw_off_time_ms = self.eeprom.init_toff();
        self.check_limits();
        self.set_level_dac_i();
        self.reset_energy_counters();

        self.hw.lcd_write_line(0, VERS3_STR);
        if self.eeprom.ee_initialised == 0xAA55 {
            self.hw
                .lcd_write_line(1, &format!("Adr {}", self.slave_channel));
        } else {
            self.hw.lcd_write_line(1, "EEPROM EMPTY!");
        }

        self.hw.serial_write(VERS1_STR);
        self.ser_crlf();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.init_scales();
        match self.mode {
            Mode::OutputOff => {
                self.output_enabled = false;
                self.set_level_dac_i();
            }
            Mode::IHiVolt | Mode::ILoVolt => {
                self.output_enabled = true;
                self.set_level_dac_i();
            }
            Mode::RHiVolt | Mode::RLoVolt => {
                self.output_enabled = true;
                self.set_level_dac_r();
            }
            Mode::PHiVolt | Mode::PLoVolt => {
                self.output_enabled = true;
                self.set_level_dac_p();
            }
        }
    }

    pub fn set_mode_from_raw(&mut self, raw: u8) -> ErrorCode {
        if let Some(mode) = Mode::from_raw(raw) {
            self.mode = mode;
            self.check_limits()
        } else {
            self.mode = Mode::OutputOff;
            ErrorCode::ParamErr
        }
    }

    pub fn set_trigger_mask(&mut self, mask: u8) {
        self.trigger_mask = mask;
        self.eeprom.trig_mask = mask;
    }

    pub fn clear_low_voltage_fault(&mut self, reset_cutoff: bool) {
        self.faults.low_volt = false;
        if reset_cutoff {
            self.voltage_cutoff = 0.0;
        }
        self.output_enabled = true;
        self.fault_check();
    }

    pub fn set_voltage_cutoff(&mut self, voltage: Float) {
        self.clear_low_voltage_fault(false);
        self.voltage_cutoff = voltage;
        self.check_limits();
    }

    pub fn set_fuse_blown(&mut self, blown: bool) {
        self.faults.fuse_blown = blown;
        self.status.overload_flag = self.faults.any();
        self.update_output_drive();
    }

    pub fn reset_energy_counters(&mut self) {
        self.capacity_ah = 0.0;
        self.capacity_wh = 0.0;
    }

    pub fn push_serial_char(&mut self, ch: char) -> Option<String> {
        self.service_step(20);
        match ch {
            '\u{08}' => {
                self.serial_input.pop();
                None
            }
            '\r' => {
                let command = core::mem::take(&mut self.serial_input);
                self.completed_command = Some(command.clone());
                Some(command)
            }
            '\u{20}'..='\u{7f}' => {
                self.serial_input.push(ch);
                None
            }
            _ => None,
        }
    }

    pub fn take_completed_command(&mut self) -> Option<String> {
        self.completed_command.take()
    }

    fn use_on_phase(&self, requested_on_phase: bool) -> bool {
        self.no_toggle || requested_on_phase
    }

    fn trigin_enabled(&self) -> bool {
        (self.trigger_mask & 0x01) != 0
    }

    fn trigauto_enabled(&self) -> bool {
        (self.trigger_mask & 0x02) != 0
    }

    fn lm75_intern_enabled(&self) -> bool {
        (self.scale.options & LM75_INTERNAL_BIT) != 0
    }

    fn lm75_extern_enabled(&self) -> bool {
        (self.scale.options & LM75_EXTERNAL_BIT) != 0
    }

    fn active_voltage_limit(&self) -> Float {
        if self.mode.is_low_voltage() {
            self.eeprom.voltage_limit_lo()
        } else {
            self.eeprom.voltage_limit_hi()
        }
    }

    fn apply_shunt_change(&mut self, shunt: u8) {
        if shunt != self.old_shunt_select {
            self.dac_raw_active = 0;
            self.hw.set_dac_raw(0);
        }
        self.set_shunt(shunt);
        self.old_shunt_select = self.shunt_select;
    }

    fn quantize_dac(&self, value: Float, offset: i16) -> u16 {
        let raw = value.round() as i32 + i32::from(offset);
        raw.clamp(0, i32::from(self.scale.dac_max)) as u16
    }

    fn update_output_drive(&mut self) {
        let enabled = self.mode != Mode::OutputOff && self.output_enabled && !self.faults.any();
        self.hw.set_output_enabled(enabled);
        if enabled {
            self.dac_raw_active = self.dac_raw_on;
            self.hw.set_dac_raw(self.dac_raw_active);
        } else {
            self.dac_raw_active = 0;
            self.hw.set_dac_raw(0);
        }
    }

    fn refresh_measurements(&mut self) -> MeasurementSnapshot {
        let voltage_on = self.read_voltage_phase(true);
        let current_on = self.read_current_phase(true);
        let voltage_off = self.read_voltage_phase(false);
        let current_off = self.read_current_phase(false);
        let power_on = if self.output_enabled {
            voltage_on * current_on
        } else {
            0.0
        };
        let power_off = if self.output_enabled {
            voltage_off * current_off
        } else {
            0.0
        };
        let total_time = (self.pw_on_time_ms + self.pw_off_time_ms).max(1) as Float;
        let power_avg = if self.output_enabled {
            (power_on * self.pw_on_time_ms.max(0) as Float
                + power_off * self.pw_off_time_ms.max(0) as Float)
                / total_time
        } else {
            0.0
        };

        MeasurementSnapshot {
            voltage_on,
            current_on,
            voltage_off,
            current_off,
            power_on,
            power_off,
            power_avg,
        }
    }

    fn emit_periodic_telemetry(&mut self) {
        if self.fault_timer_byte == 0 {
            if self.trigauto_enabled() {
                self.emit_measurement_channel(10, self.last_measurement.voltage_on);
                self.emit_measurement_channel(11, self.last_measurement.current_on);
                if !self.no_toggle {
                    self.emit_measurement_channel(15, self.last_measurement.voltage_off);
                    self.emit_measurement_channel(16, self.last_measurement.current_off);
                }
            }
            if self.faults.any() {
                self.ser_prompt(ErrorCode::OvlErr);
            }
            self.fault_timer_byte = PERIODIC_TELEMETRY_CYCLES;
        }
        self.fault_timer_byte = self.fault_timer_byte.saturating_sub(1);
    }

    fn emit_trigger_telemetry(&mut self) {
        if !self.trigin_enabled() {
            return;
        }

        if self.hw.read_trigger_in() {
            if !self.trig_on_sema {
                self.emit_measurement_channel(10, self.last_measurement.voltage_on);
                self.emit_measurement_channel(11, self.last_measurement.current_on);
                self.trig_on_sema = true;
                self.trig_off_sema = false;
            }
        } else if !self.trig_off_sema {
            self.emit_measurement_channel(15, self.last_measurement.voltage_off);
            self.emit_measurement_channel(16, self.last_measurement.current_off);
            self.trig_off_sema = true;
            self.trig_on_sema = false;
        }
    }

    fn emit_measurement_channel(&mut self, sub_channel: u8, value: Float) {
        self.sub_channel = sub_channel;
        self.write_ch_prefix();
        self.hw.serial_write(&self.param_to_str(value));
        self.ser_crlf();
    }

    fn integrate_energy(&mut self) {
        if self.pw_off_time_ms == 0 || self.i_percent == 100 {
            self.capacity_ah += self.current_amp / (3600.0 * 5.0);
            self.capacity_wh += self.current_amp * self.current_voltage / (3600.0 * 5.0);
        } else {
            self.reset_energy_counters();
        }
    }

    fn poll_temperature(&mut self) {
        if !(self.lm75_intern_enabled() || self.lm75_extern_enabled()) {
            self.temperature_c = Some(0.0);
            return;
        }
        if self.temperature_timer == 0 {
            self.temperature_timer = TEMPERATURE_POLL_CYCLES;
            self.get_lm75_temp();
        }
        self.temperature_timer = self.temperature_timer.saturating_sub(1);
    }
}
