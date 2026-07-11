//! Implements DDS, DAC, and relay wire transactions without allocating temporary buffers.

use super::*;

/// DDS hardware-side working registers and relay image used to build deterministic DAC and AD9833 transactions.
#[derive(Clone, Debug)]
pub struct DdsHardwareState {
    /// Caches `board_has_two_shift_registers` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub board_has_two_shift_registers: bool,

    /// Caches `dss_cmd` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub dss_cmd: u16,

    /// Retains the validated AD9833 waveform control state between tuning-word and control-register writes.
    pub wave_cmd: Ad9833Control,

    /// Caches `switch_state` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub switch_state: u8,

    /// Scratch word shifted most-significant bit first; destructive shifts leave the retained calibrated DAC words untouched.
    pub dac_temp: i16,

    /// Caches `level_byte_hi` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub level_byte_hi: u8,

    /// Caches `level_byte_lo` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub level_byte_lo: u8,

    /// Caches `dds_frequency_word` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub dds_frequency_word: i32,

    /// Amplitude attenuation range currently reflected in relay state and level calibration.
    pub level_range: bool,

    /// Stores the requested frequency tenths hz; limit checking and calibrated output conversion consume this same value before hardware is updated.
    pub frequency_tenths_hz: i32,

    /// Stores the requested offset mv; limit checking and calibrated output conversion consume this same value before hardware is updated.
    pub offset_mv: i32,

    /// Calibrated amplitude DAC code latched with the current attenuation and waveform routing.
    pub dac_level: f32,

    /// Caches `attn_switch_point` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub attn_switch_point: f32,

    /// Caches `level_scale_low` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub level_scale_low: f32,

    /// Caches `level_scale_high` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub level_scale_high: f32,

    /// Caches `pwr_gain` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub pwr_gain: f32,

    /// Amplitude correction applied when the attenuator relay selects the low-level range.
    pub attn_fac: f32,
}
impl Default for DdsHardwareState {
    /// Creates a DDS hardware shadow with output off, relay bytes clear, and converter scratch registers reset.
    fn default() -> Self {
        Self {
            board_has_two_shift_registers: true,
            dss_cmd: 0,
            wave_cmd: Ad9833Control::Reset,
            switch_state: 0,
            dac_temp: 0,
            level_byte_hi: 0,
            level_byte_lo: 0,
            dds_frequency_word: 0,
            level_range: false,
            frequency_tenths_hz: 10_000,
            offset_mv: 0,
            dac_level: 0.0,
            attn_switch_point: 100.0,
            level_scale_low: 1.0,
            level_scale_high: 1.0,
            pwr_gain: 2.0,
            attn_fac: 40.0,
        }
    }
}
impl DdsHardwareState {
    /// Bit-bangs the auxiliary UART with the edge spacing and idle level expected by the legacy MP3 controller.
    pub fn ser_aux<IO: DdsHardwareIo>(&self, io: &mut IO, mybyte: u8) {
        let mut value = mybyte;

        // Auxiliary 19.2 kBd serial output for the MP3 player.
        // The Pascal code sends a start bit, then shifts bit 0 first.
        io.clear_bit(PortKind::Extension, B_SER_AUX);
        io.delay_units(5);

        for _ in 0..8 {
            if value & 0x01 != 0 {
                io.set_bit(PortKind::Extension, B_SER_AUX);
            } else {
                io.clear_bit(PortKind::Extension, B_SER_AUX);
            }
            io.delay_units(5);
            value >>= 1;
        }

        io.set_bit(PortKind::Extension, B_SER_AUX);
        io.delay_units(10);
    }

    /// Serializes the 12-bit DAC word with the original hold delays required by the converter and board wiring.
    pub fn shift_out_1257<IO: DdsHardwareIo>(&mut self, io: &mut IO, my_val: i16) {
        let clamped = my_val.clamp(-0x07ff, 0x07ff);
        // The offset DAC is wired around midscale, so 0 V lives at FS/2.
        // Negative offsets are therefore corrected by adding 0x800 first.
        let dac_word = (clamped + 0x0800) as u16;
        self.dac_temp = clamped + 0x0800;

        io.clear_bit(PortKind::ControlBit, B_SDATAOUT);
        io.clear_bit(PortKind::ControlBit, B_SCLK);
        io.set_bit(PortKind::ControlBit, B_STRDAC);

        for bit in (0..12).rev() {
            if (dac_word >> bit) & 1 != 0 {
                io.set_bit(PortKind::ControlBit, B_SDATAOUT);
            } else {
                io.clear_bit(PortKind::ControlBit, B_SDATAOUT);
            }

            io.set_bit(PortKind::ControlBit, B_SCLK);
            if bit == 0 {
                io.clear_bit(PortKind::ControlBit, B_STRDAC);
            }
            io.nop();
            io.clear_bit(PortKind::ControlBit, B_SDATAOUT);
            io.clear_bit(PortKind::ControlBit, B_SCLK);
        }

        io.set_bit(PortKind::ControlBit, B_STRDAC);
    }

    /// Updates the cascaded level/relay shift registers and pulses the latch only after the complete payload is stable.
    pub fn shift_out_level_sr<IO: DdsHardwareIo>(&mut self, io: &mut IO, my_level_sr: i16) {
        // The 4094 chain carries relay state first and the attenuator level bytes after it.
        self.level_byte_hi = ((my_level_sr as u16 >> 8) & 0x00ff) as u8;
        self.level_byte_lo = (my_level_sr as u16 & 0x00ff) as u8;

        if self.board_has_two_shift_registers {
            // On the 2-SR board the relay bits are merged into the upper level byte.
            self.level_byte_hi |= self.switch_state;
        }

        io.clear_bit(PortKind::DdsOut, B_SCLK);
        io.clear_bit(PortKind::DdsOut, B_SDATAOUT);

        self.shift_byte_msb_first(io, PortKind::DdsOut, PortKind::DdsOut, self.switch_state);
        self.shift_byte_msb_first(io, PortKind::DdsOut, PortKind::DdsOut, self.level_byte_hi);
        self.shift_byte_msb_first(io, PortKind::DdsOut, PortKind::DdsOut, self.level_byte_lo);

        io.set_bit(PortKind::DdsOut, B_STROBE);
        io.nop();
        io.nop();
        io.clear_bit(PortKind::DdsOut, B_STROBE);
        io.set_bit(PortKind::DdsOut, B_SCLK);
    }

    /// Sends one AD9833 control word inside a critical section so its frame cannot be split by an interrupt.
    pub fn send_dds<IO: DdsHardwareIo>(&self, io: &mut IO) {
        io.set_bit(PortKind::DdsOut, B_SCLK);
        io.clear_bit(PortKind::ControlBit, B_SDATAOUT);
        io.clear_bit(PortKind::DdsOut, B_FSYNC);

        // AD9833 writes are 16-bit frames with the high byte shifted first.
        for bit in (0..16).rev() {
            if (self.dss_cmd >> bit) & 1 != 0 {
                io.set_bit(PortKind::ControlBit, B_SDATAOUT);
            }
            io.clear_bit(PortKind::ControlBit, B_SCLK);
            io.clear_bit(PortKind::ControlBit, B_SDATAOUT);
            io.set_bit(PortKind::ControlBit, B_SCLK);
        }

        io.set_bit(PortKind::DdsOut, B_FSYNC);
    }

    /// Programs the SQG output path while preserving the board-specific shared shift-register payload.
    pub fn set_level_dds_sqg<IO: DdsHardwareIo>(&mut self, io: &mut IO, wave: Waveform) {
        self.switch_state = 0;

        if self.dac_level < self.attn_switch_point {
            self.set_attn_sw(true);
        } else {
            self.set_attn_sw(false);
        }

        self.wave_cmd = match wave {
            Waveform::Sine => Ad9833Control::Sine,
            Waveform::Triangle => Ad9833Control::Triangle,
            Waveform::Square | Waveform::Logic => Ad9833Control::Square,
            Waveform::Off | Waveform::External(_) => Ad9833Control::Reset,
        };

        // The Pascal SQG variant does not recompute or clear the level payload before
        // committing the relay selection. Preserve the existing shift-register payload
        // bytes and only merge the current relay bits in ShiftOutLevelSR.
        let retained_level_payload =
            ((u16::from(self.level_byte_hi) << 8) | u16::from(self.level_byte_lo)) as i16;
        self.shift_out_level_sr(io, retained_level_payload);

        // The SQG variant builds the tuning word from decimal digits using floating-point factors.
        self.dds_frequency_word = Self::dds_tuning_word_sqg(self.frequency_tenths_hz);

        io.begin_critical_section();
        self.send_tuning_word(io, self.dds_frequency_word as u32);
        self.dss_cmd = self.wave_cmd.as_word();
        self.send_dds(io);
        io.end_critical_section();
    }

    /// Programs attenuation, offset, tuning words, and waveform in a safe order so relay transitions cannot expose an unintended level.
    pub fn set_level_dds<IO: DdsHardwareIo>(&mut self, io: &mut IO, wave: Waveform) {
        self.switch_state = 0;
        // Zero the level bytes first to suppress switching clicks while the relays move.
        self.level_byte_hi = 0;
        self.level_byte_lo = 0;

        let mut my_offset = self.offset_mv;

        if my_offset == 0 {
            // Zero offset opens the DC offset relay path and turns the indicator LED off.
            self.set_offs_sw(true);
            self.set_led_switch(io, true);
        } else {
            self.set_led_switch(io, false);
        }

        let my_level = if self.dac_level < self.attn_switch_point {
            // Below the threshold the AC path goes back to full-scale and uses the 1/40 attenuator relay.
            let scaled = (self.dac_level * self.attn_fac * self.level_scale_low).round() as i16;
            self.set_attn_sw(true);

            if self.level_range {
                self.dss_cmd = Ad9833Control::Reset.as_word();
                io.begin_critical_section();
                self.send_dds(io); // Briefly mute the DDS before the range relay flips.
                io.end_critical_section();
                self.shift_out_level_sr(io, 0); // Commit relay state, then wait for contacts to settle.
                io.delay_ms(5);
                self.level_range = false;
            }

            scaled
        } else {
            self.set_attn_sw(false);
            self.level_range = true;
            (self.dac_level * self.level_scale_high).round() as i16
        };

        self.wave_cmd = match wave {
            Waveform::Sine => Ad9833Control::Sine,
            Waveform::Triangle => Ad9833Control::Triangle,
            Waveform::Square => {
                self.set_square_sw(true);
                Ad9833Control::Square
            }
            Waveform::Logic => {
                self.set_square_sw(true);
                if self.board_has_two_shift_registers {
                    // The 2-SR logic variant reuses the offset DAC as a power/output level helper.
                    my_offset =
                        (self.dac_level * self.pwr_gain * core::f32::consts::SQRT_2).round() as i32;
                    self.set_offs_sw(false);
                } else {
                    self.set_logic_sw(true);
                }
                Ad9833Control::Square
            }
            Waveform::External(_) => {
                // External/audio modes disable DDS generation and only gate the external path.
                self.set_ext_on(true);
                Ad9833Control::Reset
            }
            Waveform::Off => Ad9833Control::Reset,
        };

        self.shift_out_1257(io, (my_offset / 5) as i16); // FS = 10 V, so one DAC count is 5 mV.
        self.shift_out_level_sr(io, my_level);

        // The original firmware derives the AD9833 tuning word by summing decimal-digit factors.
        self.dds_frequency_word = Self::dds_tuning_word_integer(self.frequency_tenths_hz);

        io.begin_critical_section();
        self.send_tuning_word(io, self.dds_frequency_word as u32);
        self.dss_cmd = self.wave_cmd.as_word();
        self.send_dds(io);
        io.end_critical_section();
    }

    /// Reproduces the integer Pascal decade sum used by the original DDS firmware.
    pub fn dds_tuning_word_integer(frequency_tenths_hz: i32) -> i32 {
        let mut acc = 0_i32;
        for (digit, factor) in Self::decimal_digits::<8>(frequency_tenths_hz)
            .into_iter()
            .zip(FHZ_INT)
        {
            acc += factor * i32::from(digit);
        }
        acc
    }

    /// Reproduces the SQG floating-point decade sum for its different reference clock.
    pub fn dds_tuning_word_sqg(frequency_tenths_hz: i32) -> i32 {
        let mut acc = 0.0_f32;
        for (digit, factor) in Self::decimal_digits::<9>(frequency_tenths_hz)
            .into_iter()
            .zip(FHZ_SQG)
        {
            acc += factor * f32::from(digit);
        }
        acc as i32
    }

    /// Splits a 28-bit tuning word into the two 14-bit AD9833 register writes required on the wire.
    pub fn dds_frequency_frames(tuning_word: u32) -> [u16; 2] {
        [
            ((tuning_word & 0x3fff) as u16) | DDS_FREQ_REGISTER_WRITE,
            (((tuning_word >> 14) & 0x3fff) as u16) | DDS_FREQ_REGISTER_WRITE,
        ]
    }

    /// Sends the AD9833 low 14-bit frequency frame before the high frame, preserving the device's required two-write register sequence.
    pub(super) fn send_tuning_word<IO: DdsHardwareIo>(&mut self, io: &mut IO, tuning_word: u32) {
        let [low_frame, high_frame] = Self::dds_frequency_frames(tuning_word);
        self.dss_cmd = low_frame;
        self.send_dds(io);
        self.dss_cmd = high_frame;
        self.send_dds(io);
    }

    /// Clocks bit 7 through bit 0 while returning data and clock low after each edge, providing the byte primitive used by DAC and relay chains.
    pub(super) fn shift_byte_msb_first<IO: DdsHardwareIo>(
        &self,
        io: &mut IO,
        data_port: PortKind,
        clock_port: PortKind,
        value: u8,
    ) {
        let mut shift = value;
        for _ in 0..8 {
            if shift & 0x80 != 0 {
                io.set_bit(data_port, B_SDATAOUT);
            }
            io.set_bit(clock_port, B_SCLK);
            shift <<= 1;
            io.clear_bit(data_port, B_SDATAOUT);
            io.clear_bit(clock_port, B_SCLK);
        }
    }

    /// Extracts fixed decimal digits arithmetically, avoiding heap formatting while retaining Pascal's decade-based tuning calculation.
    pub(super) fn decimal_digits<const WIDTH: usize>(value: i32) -> [u8; WIDTH] {
        let mut digits = [0; WIDTH];
        let mut remaining = value.max(0) as u32;
        let mut index = WIDTH;
        while index != 0 {
            index -= 1;
            digits[index] = (remaining % 10) as u8;
            remaining /= 10;
        }
        digits
    }

    /// Updates the front-panel switch LED bit in the shift-register shadow before the next latch.
    pub(super) fn set_led_switch<IO: DdsHardwareIo>(&self, io: &mut IO, high: bool) {
        if high {
            io.set_bit(PortKind::LedOut, LED_SWITCH_BIT);
        } else {
            io.clear_bit(PortKind::LedOut, LED_SWITCH_BIT);
        }
    }

    /// Updates the relay-shadow bit that routes square-wave output.
    pub(super) fn set_square_sw(&mut self, high: bool) {
        let bit = if self.board_has_two_shift_registers {
            TWO_SR_SQUARE_SW_BIT
        } else {
            THREE_SR_SQUARE_SW_BIT
        };
        self.set_switch_bit(bit, high);
    }

    /// Updates the relay-shadow bit selecting the attenuator range.
    pub(super) fn set_attn_sw(&mut self, high: bool) {
        let bit = if self.board_has_two_shift_registers {
            TWO_SR_ATTN_SW_BIT
        } else {
            THREE_SR_ATTN_SW_BIT
        };
        self.set_switch_bit(bit, high);
    }

    /// Updates the relay-shadow bit enabling the external output stage.
    pub(super) fn set_ext_on(&mut self, high: bool) {
        let bit = if self.board_has_two_shift_registers {
            TWO_SR_EXT_ON_BIT
        } else {
            THREE_SR_EXT_ON_BIT
        };
        self.set_switch_bit(bit, high);
    }

    /// Updates the relay-shadow bit connecting the offset-DAC path.
    pub(super) fn set_offs_sw(&mut self, high: bool) {
        let bit = if self.board_has_two_shift_registers {
            TWO_SR_OFFS_SW_BIT
        } else {
            THREE_SR_OFFS_SW_BIT
        };
        self.set_switch_bit(bit, high);
    }

    /// Updates the relay-shadow bit routing logic-level output.
    pub(super) fn set_logic_sw(&mut self, high: bool) {
        if !self.board_has_two_shift_registers {
            self.set_switch_bit(THREE_SR_LOGIC_SW_BIT, high);
        }
    }

    /// Updates one relay bit in the shadow byte; the complete byte is latched later to avoid partial output states.
    pub(super) fn set_switch_bit(&mut self, bit: u8, high: bool) {
        if high {
            self.switch_state |= 1 << bit;
        } else {
            self.switch_state &= !(1 << bit);
        }
    }
}
