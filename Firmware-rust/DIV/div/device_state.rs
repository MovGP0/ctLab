//! Defines DIV state retained across parser, polling-loop, or interrupt operations.

#[allow(unused_imports)]
use super::*;

/// Collects device state that must survive across polling-loop or interrupt updates.
#[derive(Debug, Clone)]
pub struct DeviceState<H> {
    /// Owns the hardware boundary through which this state performs all converter, relay, serial, and LCD access.
    pub hw: H,

    /// Keeps EEPROM values together so reset and write-enable handling use one source of truth.
    pub eeprom: EepromData,

    /// Stores the address read from board straps and used to accept or prefix serial frames.
    pub slave_ch: u8,

    /// Holds the protocol subchannel selected by the current frame; 255 is the status channel.
    pub sub_ch: u8,

    /// Tracks the most recently addressed channel so short-form commands can omit the address.
    pub current_ch: u8,

    /// Tracks range so conversion, relay, and formatting decisions agree.
    pub range: DivRange,

    /// Selects direct, fast, or slow AD24 data for the live DIV display.
    pub lcd_integrate: u8,

    /// Sets the number of raw encoder increments required for one accepted detent.
    pub inc_rast: i16,

    /// Holds the calibrated LTC2400 result used for the primary display and serial response.
    pub measured_value: Float,

    /// Holds the calibrated ADC10 result used by RMS, peak, and auxiliary subchannels.
    pub measured_aux: Float,
    // Raised by either the external INT2 edge or the periodic auto-trigger timer.

    /// Latches a DIV trigger until the selected measurement responses have been emitted.
    pub trigger_pending: bool,

    /// Counts auto trigger elapsed ms in systicks until the corresponding nonblocking action is due.
    pub auto_trigger_elapsed_ms: u16,

    /// Contains trigger outputs in protocol order until the serial or hardware sink accepts it.
    pub trigger_outputs: [u8; 3],

    /// Tracks trigger output count within the fixed-capacity sequence used by this routine.
    pub trigger_output_count: u8,

    /// Tracks current range config so conversion, relay, and formatting decisions agree.
    pub current_range_config: RangeRelayConfig,
    // Pascal kept separate fast/slow integrated AD24 accumulators for quieter display reads.

    /// Holds the previous two-sample LTC2400 average in the converter's midscale-biased count domain.
    pub integrate_24_fast: i64,

    /// Holds the slower LTC2400 integration accumulator used for the quiet display mode.
    pub integrate_24_slow: i64,

    /// Latches the LTC2400 negative/clipping status associated with the current sample.
    pub overload_negative: bool,

    /// Latches positive overrange from the LTC2400 status bits or ADC10 full-scale test.
    pub overload_positive: bool,

    /// Rejects state-changing commands while initialization, calibration, or a panel operation owns the device.
    pub busy_flag: bool,

    /// Adds the user-service-request bit to the next status response after a panel action.
    pub user_srq_flag: bool,

    /// Arms EEPROM and calibration changes after `WEN` until the firmware clears the latch.
    pub ee_unlocked: bool,

    /// Carries the exact validation result into the next serial status response.
    pub check_limit_err: ErrorCode,

    /// Holds the decoded front-panel key number, with zero meaning no accepted press.
    pub button_number: u8,

    /// Counts protocol errors returned by `ERC` until that command clears the counter.
    pub err_count: i16,

    /// Counts fault timer ticks in systicks until the corresponding nonblocking action is due.
    pub fault_timer_ticks: u8,

    /// Buffers the current CR-terminated host command before parser dispatch.
    pub ser_input: String,
}

impl<H: DivHardware> DeviceState<H> {
    /// Creates DIV runtime state at the Pascal power-on range (2.5 V DC), with empty integrations and no pending trigger.
    pub fn new(hw: H) -> Self {
        Self {
            hw,
            eeprom: EepromData::default(),
            slave_ch: 0,
            sub_ch: 0,
            current_ch: 255,
            range: DivRange::Dc2V5,
            lcd_integrate: 1,
            inc_rast: 4,
            measured_value: 0.0,
            measured_aux: 0.0,
            trigger_pending: false,
            auto_trigger_elapsed_ms: 0,
            trigger_outputs: [0; 3],
            trigger_output_count: 0,
            current_range_config: RangeRelayConfig::for_range(DivRange::Dc2V5),
            integrate_24_fast: 0,
            integrate_24_slow: 0,
            overload_negative: false,
            overload_positive: false,
            busy_flag: false,
            user_srq_flag: false,
            ee_unlocked: false,
            check_limit_err: ErrorCode::NoErr,
            button_number: 0,
            err_count: 0,
            fault_timer_ticks: 20,
            ser_input: String::new(),
        }
    }

    /// Copies persisted startup choices into live state so initialization and later commands observe the same configuration.
    pub fn patch_copy_from_ee(&mut self) {
        self.inc_rast = self.eeprom.init_inc_rast;
        self.lcd_integrate = self.eeprom.init_lcd_integrate;
        self.range = self.eeprom.init_range;
        self.hw.set_trigger_edge(self.eeprom.trigger_edge_level);
    }

    /// Reports whether ac range without mutating device state.
    pub fn is_ac_range(&self) -> bool {
        matches!(
            self.range,
            DivRange::Ac250mV
                | DivRange::Ac2V5
                | DivRange::Ac25V
                | DivRange::Ac250V
                | DivRange::Ac250uA
                | DivRange::Ac25mA
                | DivRange::Ac2A5
                | DivRange::Ac10A
        )
    }

    /// Applies the active range offset and scale while producing parameter scale 10.
    pub fn param_scale_10(&self, raw: i16) -> Float {
        // The original path applied offset first, then the per-range full-scale factor,
        // then the stored calibration scale factor for the 10-bit ADC path.
        let offset_raw = raw + self.eeprom.ad10_offsets[self.range as usize];
        let value = offset_raw as Float
            * RANGE_ARRAY_10[self.range as usize]
            * self.eeprom.ad10_scales[self.range as usize];
        if self.is_ac_range() {
            value.abs()
        } else {
            value
        }
    }

    /// Applies the active range offset and scale while producing parameter scale 24.
    pub fn param_scale_24(&self, raw: i32) -> Float {
        // Same scaling order as Pascal, but for the LTC2400 measurement path.
        let offset_raw = raw + self.eeprom.ad24_offsets[self.range as usize];
        let value = offset_raw as Float
            * RANGE_ARRAY_24[self.range as usize]
            * self.eeprom.ad24_scales[self.range as usize];
        if self.is_ac_range() {
            value.abs()
        } else {
            value
        }
    }

    /// Obtains ad10 from the owning state or hardware register for the caller that consumes it.
    pub fn get_ad10(&mut self, channel: u8) {
        let mut raw = self.hw.read_adc10(channel);
        self.overload_positive = raw >= 1022;
        self.overload_negative = false;
        if channel == 5 {
            self.overload_negative = raw == 0;
            raw -= 512;
        }
        self.measured_aux = self.param_scale_10(raw);
    }

    /// Returns the latest LTC2400 sample so callers use the intended display or trigger integration mode.
    pub fn get_ad24(&mut self, int_mode: u8) {
        self.overload_negative = self.hw.adc24_overload_negative();
        self.overload_positive = self.hw.adc24_overload_positive();
        let source = match int_mode {
            1 => self.hw.read_adc24_fast_integrated(),
            2 => self.hw.read_adc24_slow_integrated(),
            _ => self.hw.read_adc24(),
        };
        let mut raw = source - ADC24_MID_SCALE;
        if self.is_ac_range() && raw < 0 {
            raw = 0;
        }
        self.measured_value = self.param_scale_24(raw);
    }

    /// Derives overload flag from the current flags for protocol and protection decisions.
    pub fn overload_flag(&self) -> bool {
        self.overload_negative || self.overload_positive
    }

    /// Derives fault flags from the current flags for protocol and protection decisions.
    pub fn fault_flags(&self) -> u8 {
        u8::from(self.overload_negative) | (u8::from(self.overload_positive) << 1)
    }

    /// The hardware implementation must use volatile/atomic access for a flag
    /// which is shared with an interrupt handler.
    pub fn wait_ad10(&mut self) {
        self.hw.clear_adc10_ready();
        while !self.hw.adc10_ready() {
            core::hint::spin_loop();
        }
    }

    /// The hardware implementation must use volatile/atomic access for a flag
    /// which is shared with an interrupt handler.
    pub fn wait_ad24(&mut self) {
        self.hw.clear_adc24_ready();
        while !self.hw.adc24_ready() {
            core::hint::spin_loop();
        }
    }

    /// Resets integrate reset so samples from an earlier range or operation cannot leak into the next result.
    pub fn integrate_reset(&mut self) {
        // Clear the integration history whenever the range relays move so the next
        // reading is not blended with samples from the previous attenuation path.
        self.integrate_24_fast = i64::from(ADC24_MID_SCALE);
        self.integrate_24_slow = i64::from(ADC24_MID_SCALE);
    }

    /// Applies range as one coherent state and hardware transition.
    pub fn switch_range(&mut self, range: DivRange) {
        // In Pascal this selected relay and gain bit patterns from lookup tables on
        // PortA/PortC, updated display formatting, and reset the running integrators.
        self.range = range;
        self.current_range_config = RangeRelayConfig::for_range(range);
        self.hw.set_range_config(self.current_range_config);
        self.integrate_reset();
    }

    /// Persists the `TRL` polarity and immediately reconfigures the hardware INT2 edge selector.
    pub fn set_trigger_edge_level(&mut self, positive_edge: bool) {
        self.eeprom.trigger_edge_level = positive_edge;
        self.hw.set_trigger_edge(positive_edge);
    }

    /// Latches int2 trigger edge for deferred processing outside the interrupt-sensitive edge handler.
    pub fn int2_trigger_edge(&mut self, positive_edge: bool) {
        if positive_edge == self.eeprom.trigger_edge_level {
            self.trigger_pending = true;
        }
    }

    /// Advances auto trigger using elapsed time supplied by the caller.
    pub fn tick_auto_trigger(&mut self, elapsed_ms: u16) {
        let timer = self.eeprom.trigger_timer_ms;
        if timer == 0 {
            self.auto_trigger_elapsed_ms = 0;
            return;
        }

        self.auto_trigger_elapsed_ms = self.auto_trigger_elapsed_ms.saturating_add(elapsed_ms);
        if self.auto_trigger_elapsed_ms >= timer {
            self.auto_trigger_elapsed_ms %= timer;
            self.trigger_pending = true;
        }
    }

    /// Handles trigger as one bounded polling-loop or interrupt service step.
    pub fn service_trigger(&mut self) -> &[u8] {
        self.trigger_output_count = 0;
        if !self.trigger_pending {
            return &self.trigger_outputs[..0];
        }

        let mask = self.eeprom.trigger_mode;
        if (mask & 0x01) != 0 {
            self.push_trigger_output(0);
        }
        if (mask & 0x02) != 0 {
            self.push_trigger_output(10);
        }
        if (mask & 0x04) != 0 {
            self.push_trigger_output(11);
        }
        self.trigger_pending = false;
        &self.trigger_outputs[..usize::from(self.trigger_output_count)]
    }

    /// Queues trigger output for the next bounded consumer without changing unrelated state.
    pub(super) fn push_trigger_output(&mut self, sub_channel: u8) {
        let index = usize::from(self.trigger_output_count);
        self.trigger_outputs[index] = sub_channel;
        self.trigger_output_count += 1;
    }

    /// Terminates the current serial response with CRLF because existing clients parse line-delimited frames.
    pub fn ser_crlf(&mut self) {
        self.hw.serial_write("\r\n");
    }

    /// Writes the addressed channel prefix before a payload so every response keeps the Pascal wire framing.
    pub fn write_ch_prefix(&mut self) {
        self.hw.serial_write("#");
        self.hw.serial_write(&self.slave_ch.to_string());
        self.hw.serial_write(":");
        self.hw.serial_write(&self.sub_ch.to_string());
        self.hw.serial_write("=");
    }

    /// Writes serial inp to the serial, display, or peripheral destination selected by the implementation.
    pub fn write_ser_inp(&mut self) {
        self.hw.serial_write(&self.ser_input);
        self.ser_crlf();
    }

    /// Encodes the current status and error flags into the Pascal prompt frame returned after commands.
    pub fn ser_prompt(&mut self, err: ErrorCode) {
        self.sub_ch = ERR_SUB_CH;
        self.write_ch_prefix();

        let mut status = 0u8;
        if self.busy_flag {
            status |= 0x80;
        }
        if self.user_srq_flag {
            status |= 0x40;
        }
        if self.overload_flag() {
            status |= 0x20;
        }
        if self.ee_unlocked {
            status |= 0x10;
        }

        let fault_flags = self.fault_flags();
        match err {
            ErrorCode::UserReq => {
                status |= 0x40 | (self.button_number & 0x0F);
            }
            ErrorCode::OvlErr => {
                status |= fault_flags;
            }
            _ => {
                status |= err as u8;
                if err != ErrorCode::NoErr {
                    self.err_count = self.err_count.saturating_add(1);
                }
            }
        }

        self.hw.serial_write(&status.to_string());
        if fault_flags != 0 {
            if (fault_flags & DivFault::NegativeOverload.mask()) != 0 {
                self.hw.serial_write(" ");
                self.hw.serial_write(DivFault::NegativeOverload.as_str());
            }
            if (fault_flags & DivFault::PositiveOverload.mask()) != 0 {
                self.hw.serial_write(" ");
                self.hw.serial_write(DivFault::PositiveOverload.as_str());
            }
        } else {
            self.hw.serial_write(" ");
            self.hw.serial_write(err.as_str());
        }
        self.ser_crlf();
    }

    /// Converts to string into the representation used on the wire or display.
    pub fn param_to_str(&self, to_lcd: bool) -> String {
        let decimals = NACHKOMMA_ARR[self.range as usize] as usize;
        if to_lcd {
            let mut text = format!("{:0>7.*}", decimals, self.measured_value.abs());
            let prefix = if self.measured_value < 0.0 {
                '-'
            } else if self.is_ac_range() {
                '\x03'
            } else {
                '+'
            };
            text.insert(0, prefix);
            text.push_str("00000");
            text.truncate(8);
            text
        } else {
            let decimals = decimals + 2;
            if self.measured_value < 0.0 {
                format!("{:.*}", decimals, self.measured_value)
            } else {
                format!("{:.*}", decimals, self.measured_value.abs())
            }
        }
    }

    /// Emits show range using the exact channel and status framing expected by existing clients.
    pub fn show_range(&mut self) {
        self.hw
            .lcd_write_line(1, RANGE_STR_ARR[self.range as usize]);
    }

    /// Formats the current measurement as the fixed eight-character value row and writes LCD row zero.
    pub fn write_param_lcd(&mut self) {
        self.hw.lcd_write_line(0, &self.param_to_str(true));
    }

    /// Writes the primary measurement, substituting `-9999 [OVERLD]` for overloaded measurement subchannels.
    pub fn write_param_ser(&mut self, ovl: bool) {
        self.write_ch_prefix();
        if ovl && self.sub_ch < 20 {
            self.hw.serial_write("-9999 [OVERLD]");
        } else {
            self.hw.serial_write(&self.param_to_str(false));
            if self.sub_ch < 20 {
                if let Some(suffix) = range_exponent_suffix(self.range) {
                    self.hw.serial_write(suffix);
                }
            }
        }
        self.ser_crlf();
    }

    /// Writes the ADC10 auxiliary measurement with the same overload and range-suffix framing as the primary value.
    pub fn write_param_aux_ser(&mut self, ovl: bool) {
        self.write_ch_prefix();
        if ovl && self.sub_ch < 20 {
            self.hw.serial_write("-9999 [OVERLD]");
        } else {
            self.hw
                .serial_write(&self.param_to_str_value(self.measured_aux, false));
            if self.sub_ch < 20 {
                if let Some(suffix) = range_exponent_suffix(self.range) {
                    self.hw.serial_write(suffix);
                }
            }
        }
        self.ser_crlf();
    }

    /// Converts to string value into the representation used on the wire or display.
    pub(super) fn param_to_str_value(&self, value: Float, to_lcd: bool) -> String {
        let decimals = NACHKOMMA_ARR[self.range as usize] as usize;
        if to_lcd {
            let mut text = format!("{:0>7.*}", decimals, value.abs());
            let prefix = if value < 0.0 {
                '-'
            } else if self.is_ac_range() {
                '\x03'
            } else {
                '+'
            };
            text.insert(0, prefix);
            text.push_str("00000");
            text.truncate(8);
            text
        } else {
            let decimals = decimals + 2;
            if value < 0.0 {
                format!("{:.*}", decimals, value)
            } else {
                format!("{:.*}", decimals, value.abs())
            }
        }
    }

    /// Contains value in converter counts until the owning conversion or output routine consumes it.
    pub fn write_param_long_int_ser(&mut self, value: i64) {
        self.write_ch_prefix();
        self.hw.serial_write(&value.to_string());
        self.ser_crlf();
    }

    /// Validates limits before dependent hardware state is changed.
    pub fn check_limits(&mut self) -> bool {
        self.check_limit_err = ErrorCode::NoErr;
        false
    }

    /// Validates limits raw range before dependent hardware state is changed.
    pub fn check_limits_raw_range(&mut self, raw_range: u8) -> bool {
        let (range, limited) = limit_raw_range(raw_range);
        self.range = range;
        self.check_limit_err = if limited {
            ErrorCode::ParamErr
        } else {
            ErrorCode::NoErr
        };
        limited
    }

    /// Validates serial before dependent hardware state is changed.
    pub fn check_ser(&mut self) {
        while let Some(byte) = self.hw.poll_serial_byte() {
            match byte {
                8 => {
                    self.ser_input.pop();
                }
                13 => {
                    self.parse_serial_frame();
                    self.ser_input.clear();
                }
                32..=127 => {
                    self.ser_input.push(byte as char);
                }
                _ => {}
            }
        }

        self.overload_negative = self.hw.adc24_overload_negative();
        self.overload_positive = self.hw.adc24_overload_positive();
        if self.fault_timer_ticks == 0 {
            if self.fault_flags() != 0 {
                self.ser_prompt(ErrorCode::OvlErr);
            }
            self.fault_timer_ticks = 20;
        } else {
            self.fault_timer_ticks -= 1;
        }
    }

    /// Parses serial frame and updates only the state owned by that protocol phase.
    pub(super) fn parse_serial_frame(&mut self) {
        if self.ser_input.is_empty() {
            self.ser_prompt(ErrorCode::NoErr);
            return;
        }

        if self.ser_input.starts_with('#') {
            self.write_ser_inp();
            return;
        }

        let frame = self.ser_input.clone();
        let (addressed, payload) = match frame.split_once(':') {
            Some((main_ch, rest)) => {
                if main_ch == "*" {
                    self.write_ser_inp();
                    (true, rest)
                } else if main_ch.parse::<u8>().ok() == Some(self.slave_ch) {
                    self.current_ch = self.slave_ch;
                    (true, rest)
                } else {
                    self.write_ser_inp();
                    (false, rest)
                }
            }
            None => (true, frame.as_str()),
        };

        if !addressed {
            return;
        }

        if self.busy_flag {
            self.ser_prompt(ErrorCode::BusyErr);
            return;
        }

        let request = !payload.contains('=');
        let command = payload.trim_end_matches('?').trim_end_matches('!');
        if request {
            self.parse_get_command(command);
        } else if let Some((left, right)) = payload.split_once('=') {
            self.parse_set_command(left.trim(), right.trim());
        } else {
            self.ser_prompt(ErrorCode::ParamErr);
        }
    }

    /// Parses get command and updates only the state owned by that protocol phase.
    pub(super) fn parse_get_command(&mut self, command: &str) {
        let upper = command.trim().to_ascii_uppercase();
        match upper.as_str() {
            "STR" => self.ser_prompt(ErrorCode::NoErr),
            "IDN" => {
                self.sub_ch = 254;
                self.write_ch_prefix();
                self.hw.serial_write(VERS1_STR);
                self.ser_crlf();
            }
            "TRG" => {
                self.trigger_pending = true;
                self.ser_prompt(ErrorCode::NoErr);
            }
            "RNG" => {
                self.sub_ch = 19;
                self.write_param_long_int_ser(self.range as i64);
            }
            "ERC" => {
                self.sub_ch = 251;
                self.write_param_long_int_ser(i64::from(self.err_count));
            }
            raw => {
                let sub_ch = raw
                    .strip_prefix("VAL")
                    .unwrap_or(raw)
                    .trim()
                    .parse::<u8>()
                    .unwrap_or(0);
                self.sub_ch = sub_ch;
                self.parse_get_sub_ch(sub_ch);
            }
        }
    }

    /// Parses get sub channel and updates only the state owned by that protocol phase.
    pub(super) fn parse_get_sub_ch(&mut self, sub_ch: u8) {
        match sub_ch {
            0..=2 => {
                self.get_ad24(sub_ch);
                self.write_param_ser(self.overload_flag());
            }
            3 => {
                self.wait_ad24();
                self.get_ad24(0);
                self.write_param_ser(self.overload_flag());
            }
            10 => {
                self.wait_ad10();
                let channel = if self.is_ac_range() { 3 } else { 5 };
                self.get_ad10(channel);
                self.write_param_aux_ser(self.overload_flag());
            }
            11 => {
                self.wait_ad10();
                let channel = if self.is_ac_range() { 4 } else { 5 };
                self.get_ad10(channel);
                self.write_param_aux_ser(self.overload_flag());
            }
            19 => {
                self.write_param_long_int_ser(self.range as i64);
            }
            255 => {
                self.ser_prompt(ErrorCode::NoErr);
            }
            _ => self.ser_prompt(ErrorCode::ParamErr),
        }
    }

    /// Parses set command and updates only the state owned by that protocol phase.
    pub(super) fn parse_set_command(&mut self, left: &str, right: &str) {
        let value = right.parse::<i32>().unwrap_or(0);
        match left.to_ascii_uppercase().as_str() {
            "RNG" => {
                let limited = self.check_limits_raw_range(value as u8);
                self.sub_ch = 19;
                self.switch_range(self.range);
                if limited {
                    self.ser_prompt(self.check_limit_err);
                }
            }
            "WEN" => {
                self.sub_ch = 250;
                self.ee_unlocked = true;
            }
            _ => self.ser_prompt(ErrorCode::ParamErr),
        }
    }

    /// Initializes zero offsets in the same order as the original startup routine.
    pub fn initialise_zero_offsets(&mut self, sample: i32) {
        let offset = ADC24_MID_SCALE - sample;
        for offset_slot in self.eeprom.ad24_offsets.iter_mut() {
            *offset_slot = offset;
        }
        self.eeprom.offset_initialised = OFFSET_INITIALISED_MAGIC;
    }

    /// Reports whether zero offset initialisation without mutating device state.
    pub fn needs_zero_offset_initialisation(&self) -> bool {
        self.eeprom.offset_initialised != OFFSET_INITIALISED_MAGIC
    }

    /// Validates delay before dependent hardware state is changed.
    pub fn check_delay(&mut self, delay_ticks: u8) {
        for _ in 0..delay_ticks {
            self.check_ser();
        }
    }

    /// Services serial input during a front-panel delay so startup blinking does not make the instrument unresponsive.
    pub fn blink_delay(&mut self, delay_ticks: u8) {
        self.check_delay(delay_ticks);
    }

    /// Initializes all in the same order as the original startup routine.
    pub fn init_all(&mut self) {
        self.patch_copy_from_ee();
        self.switch_range(self.range);
        self.sub_ch = 254;
        self.write_ch_prefix();
        self.hw.serial_write(VERS1_STR);
        if self.eeprom.ee_initialised != EE_INITIALISED_MAGIC {
            self.hw.serial_write("EEPROM EMPTY! ");
        }
        if self.needs_zero_offset_initialisation() {
            self.hw.serial_write("[OFS INIT]");
            self.ser_crlf();
            self.busy_flag = true;
            self.ser_prompt(ErrorCode::BusyErr);
            let zero_sample = self.hw.read_adc24_fast_integrated();
            self.initialise_zero_offsets(zero_sample);
            self.busy_flag = false;
            self.ser_prompt(ErrorCode::NoErr);
        } else {
            self.ser_crlf();
        }
        self.current_ch = 255;
    }

    /// Handles once as one bounded polling-loop or interrupt service step.
    pub fn service_once(&mut self, elapsed_ms: u16) {
        self.check_ser();
        self.tick_auto_trigger(elapsed_ms);
        if self.trigger_pending {
            let outputs = self.service_trigger().to_vec();
            for sub_ch in outputs {
                self.sub_ch = sub_ch;
                self.parse_get_sub_ch(sub_ch);
            }
        }
    }
}
