use crate::test_failures::TestFailures;
use super::*;
use std::collections::VecDeque;

#[derive(Default)]
struct DummyHardware {
    serial: String,
    serial_in: VecDeque<char>,
    dds_words: Vec<u16>,
    shift_ops: Vec<(i32, u8)>,
    offset_ops: Vec<i16>,
    delays: Vec<u16>,
    baud_calls: Vec<(u8, bool)>,
    slave_channel: u8,
    activity_leds: Vec<bool>,
    lcd_setup_result: bool,
    lcd_chars: Vec<(u8, [u8; 8])>,
    lcd_lines: Vec<(u8, String)>,
    serial_pending: bool,
    systicks: u8,
    panel_events: Vec<PanelEvent>,
}

impl HardwareInterface for DummyHardware {
    fn serout_byte(&mut self, byte: u8) {
        self.serial.push(byte as char);
    }

    fn write_serial(&mut self, text: &str) {
        self.serial.push_str(text);
    }

    fn send_dds_word(&mut self, word: u16) {
        self.dds_words.push(word);
    }

    fn shift_out_level_sr(&mut self, level: i32, switch_state: u8) {
        self.shift_ops.push((level, switch_state));
    }

    fn shift_out_offset_dac(&mut self, dac_counts: i16) {
        self.offset_ops.push(dac_counts);
    }

    fn serial_timeout_char(&mut self, _timeout_ticks: u8) -> Option<char> {
        self.serial_in.pop_front()
    }

    fn set_serial_baud_register(&mut self, register: u8, double_speed: bool) {
        self.baud_calls.push((register, double_speed));
    }

    fn read_slave_channel(&mut self) -> u8 {
        self.slave_channel
    }

    fn serial_read_immediate(&mut self) -> Option<char> {
        self.serial_in.pop_front()
    }

    fn lcd_setup(&mut self) -> bool {
        self.lcd_setup_result
    }

    fn lcd_define_custom_char(&mut self, slot: u8, bitmap: [u8; 8]) {
        self.lcd_chars.push((slot, bitmap));
    }

    fn lcd_write_line(&mut self, row: u8, text: &str) {
        self.lcd_lines.push((row, text.to_string()));
    }

    fn serial_pending(&self) -> bool {
        self.serial_pending
    }

    fn take_systick(&mut self) -> bool {
        if self.systicks == 0 {
            false
        } else {
            self.systicks -= 1;
            true
        }
    }

    fn next_panel_event(&mut self) -> PanelEvent {
        if self.panel_events.is_empty() {
            PanelEvent::None
        } else {
            self.panel_events.remove(0)
        }
    }

    fn delay_ms(&mut self, ms: u16) {
        self.delays.push(ms);
    }

    fn set_activity_led(&mut self, active_low: bool) {
        self.activity_leds.push(active_low);
    }
}

#[test]
fn set_level_dds_emits_three_dds_words() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState::default();
    let mut hw = DummyHardware::default();

    state.frequenz = 10_000;
    state.wave = Waveform::Square;
    state.set_level_dds(&mut hw);

    assert.eq(hw.dds_words.len(), 3);
    assert.eq(hw.dds_words[2], Ad9833Control::Square.as_word());
    assert.eq(state.dds_frequ, 13_421);
    assert.finish();
}

#[test]
fn set_level_dds_restores_level_offset_and_relay_setup() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState::default();
    let mut hw = DummyHardware::default();

    state.dac_level = 1000.0;
    state.offset_mv = 250;
    state.wave = Waveform::Square;
    state.set_level_dds(&mut hw);

    assert.eq(hw.offset_ops, vec![50]);
    assert.eq(
        hw.shift_ops,
        vec![(
            40_000,
            SwitchOutput::Attenuator.mask() | SwitchOutput::Square.mask(),
        )],
    );
    assert.is_false(state.level_range);
    assert.finish();
}

#[test]
fn set_level_dds_mutes_before_switching_from_high_to_low_range() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState {
        dac_level: 1000.0,
        level_range: true,
        wave: Waveform::Square,
        ..Default::default()
    };
    let mut hw = DummyHardware::default();

    state.set_level_dds(&mut hw);

    assert.eq(hw.dds_words[0], Ad9833Control::Reset.as_word());
    assert.eq(
        hw.shift_ops[0],
        (
            0,
            SwitchOutput::Offset.mask() | SwitchOutput::Attenuator.mask(),
        ),
    );
    assert.eq(hw.delays, vec![5]);
    assert.is_false(state.level_range);
    assert.finish();
}

#[test]
fn run_main_loop_iteration_drives_burst_from_systick() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState {
        burst_mode: 2,
        burst_count: 1,
        wave_cmd: Ad9833Control::Square,
        ..Default::default()
    };
    let mut hw = DummyHardware {
        systicks: 2,
        ..Default::default()
    };

    state.run_main_loop_iteration(&mut hw);

    assert.eq(
        hw.dds_words,
        vec![
            Ad9833Control::Square.as_word(),
            Ad9833Control::Reset.as_word(),
        ],
    );
    assert.eq(state.burst_count, 2);
    assert.finish();
}

#[test]
fn run_main_loop_iteration_skips_panel_while_serial_is_pending() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState {
        lcd_present: true,
        current_ch: 0,
        ..Default::default()
    };
    let mut hw = DummyHardware {
        serial_pending: true,
        panel_events: vec![PanelEvent::Button(PanelButton::Left)],
        ..Default::default()
    };

    state.run_main_loop_iteration(&mut hw);

    assert.eq(state.modify, Modify::Frequency);
    assert.is_true(
        hw
        .panel_events
        .contains(&PanelEvent::Button(PanelButton::Left)),
    );
    assert.finish();
}

#[test]
fn panel_button_uses_busy_status_and_updates_display_selection_when_uart_idle() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState {
        lcd_present: true,
        current_ch: 0,
        ..Default::default()
    };
    let mut hw = DummyHardware {
        panel_events: vec![PanelEvent::Button(PanelButton::Left)],
        ..Default::default()
    };

    state.run_main_loop_iteration(&mut hw);

    assert.eq(state.modify, Modify::Amplitude);
    assert.is_true(state.status.busy);
    assert.is_true(hw.serial.contains("#0:1=5000"));
    assert.finish();
}

#[test]
fn parse_get_param_returns_version() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState::default();
    let mut hw = DummyHardware::default();

    state.sub_ch = 254;
    state.parse_get_param(&mut hw);

    assert.is_true(hw.serial.contains(VERS1_STR));
    assert.finish();
}

#[test]
fn patch_copy_from_ee_restores_sqg_eeprom_backed_reset_values() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState {
        wave: Waveform::Off,
        frequenz: 42,
        dac_level: 1000.0,
        terz_num: 1,
        burst_mode: 9,
        inc_rast: 1,
        attn_fac: 1.0,
        pwr_gain: 1.0,
        level_scale_low: 2.0,
        level_scale_hi: 3.0,
        ..Default::default()
    };
    state.defaults.init_wave = Waveform::Square.as_byte();
    state.defaults.init_frequenz = 20_000;
    state.defaults.init_level = 5000.0;
    state.defaults.init_terz_num = 12;
    state.defaults.init_burst = 7;
    state.defaults.init_inc_rast = 6;
    state.defaults.init_attn_fac = 33.0;
    state.defaults.init_pwr_gain = 1.5;
    state.defaults.level_scale_low = 0.95;
    state.defaults.level_scale_hi = 1.05;

    state.patch_copy_from_ee();

    assert.eq(state.wave, Waveform::Square);
    assert.eq(state.frequenz, 20_000);
    assert.eq(state.dac_level, 5000.0);
    assert.eq(state.terz_num, 12);
    assert.eq(state.burst_mode, 7);
    assert.eq(state.inc_rast, 6);
    assert.eq(state.attn_fac, 33.0);
    assert.eq(state.pwr_gain, 1.5);
    assert.eq(state.level_scale_low, 0.95);
    assert.eq(state.level_scale_hi, 1.05);
    assert.finish();
}

#[test]
fn init_all_restores_pascal_startup_state_and_banner() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState::default();
    let mut hw = DummyHardware {
        serial_in: "stale".chars().collect(),
        slave_channel: 2,
        lcd_setup_result: true,
        ..Default::default()
    };

    state.defaults.ee_initialized = 0;
    state.defaults.ee_ser_baud_reg = 5;
    state.frequenz = 42;
    state.status.busy = true;
    state.err_count = 9;
    state.err_flag = true;
    state.incr_fine = true;
    state.incr_diff = 3;
    state.ser_inp_str = "partial".to_string();

    state.init_all(&mut hw);

    assert.eq(state.defaults.ee_ser_baud_reg, 51);
    assert.eq(hw.baud_calls, vec![(51, true)]);
    assert.eq(state.slave_ch, 2);
    assert.is_true(state.lcd_present);
    assert.eq(state.status, StatusFlags::default());
    assert.eq(state.burst_count, 1);
    assert.eq(state.modify, Modify::Frequency);
    assert.eq(state.current_ch, 255);
    assert.eq(state.err_count, 0);
    assert.is_false(state.err_flag);
    assert.is_true(state.changed_flag);
    assert.is_true(state.first_turn);
    assert.is_false(state.incr_fine);
    assert.eq(state.incr_diff, 0);
    assert.is_true(state.ser_inp_str.is_empty());
    assert.is_true(hw.serial_in.is_empty());
    assert.eq(
        hw.lcd_lines,
        vec![
            (0, VERS3_STR.to_string()),
            (1, EEPROM_EMPTY_STR.to_string()),
        ],
    );
    assert.eq(hw.delays, vec![1000, 150, 150, 150, 150, 500]);
    assert.eq(hw.activity_leds, vec![true, false, true, false, true, false]);
    assert.eq(hw.serial, format!("#2:254={VERS1_STR}{EEPROM_EMPTY_STR}\r\n"));
    assert.eq(hw.dds_words.len(), 3);
    assert.finish();
}

#[test]
fn parse_extract_accepts_pascal_numeric_token_range() {
    let mut assert = TestFailures::default();

    let state = FirmwareState::default();

    let (token, end, is_param) = state.parse_extract("  -1,5/2?", 0);

    assert.is_true(is_param);
    assert.eq(token, "-1,5/2");
    assert.eq(end, 8);
    assert.finish();
}

#[test]
fn command_table_includes_pascal_nop_mapping() {
    let mut assert = TestFailures::default();

    assert.eq(FirmwareState::cmd_to_index("NOP"), CmdWhich::Nop);
    assert.eq(FirmwareState::cmd_to_index("nop"), CmdWhich::Nop);
    assert.eq(CmdWhich::Nop.as_str(), "NOP");
    assert.eq(CmdWhich::Nop.default_subchannel(), 0);
    assert.eq(CmdWhich::Ofs.as_str(), "DCO");
    assert.eq(CmdWhich::from_mnemonic("dbu"), CmdWhich::Dbu);
    assert.eq(CmdWhich::Dbu.default_subchannel(), 3);
    assert.finish();
}

#[test]
fn shared_waveform_owns_wire_codes_labels_and_sqg_limits() {
    let mut assert = TestFailures::default();

    assert.eq(Waveform::Off.as_byte(), 0);
    assert.eq(Waveform::Sine.as_str(), "Sin");
    assert.eq(Waveform::Triangle.as_str(), "Tri");
    assert.eq(Waveform::Square.as_str(), "Squ");
    assert.eq(Waveform::Logic.as_str(), "Lgc");
    assert.eq(Waveform::External(0).as_str(), "Ext");
    assert.eq(Waveform::External(2).as_byte(), 7);

    assert.eq(Waveform::from_sqg_byte(3), (Waveform::Square, false));
    assert.eq(Waveform::from_sqg_byte(4), (Waveform::Square, true));
    assert.eq(Waveform::from_sqg_byte(127), (Waveform::Square, true));
    assert.eq(Waveform::from_sqg_byte(128), (Waveform::Off, true));
    assert.eq(Waveform::from_sqg_byte(255), (Waveform::Off, true));
    assert.finish();
}

#[test]
fn typed_hardware_and_panel_values_preserve_pascal_encodings() {
    let mut assert = TestFailures::default();

    assert.eq(Ad9833Control::Reset.as_word(), 0b0010_0001_0000_0000);
    assert.eq(Ad9833Control::Sine.as_word(), 0b0010_0000_0000_0000);
    assert.eq(Ad9833Control::Triangle.as_word(), 0b0010_0000_0000_0010);
    assert.eq(Ad9833Control::Square.as_word(), 0b0010_0000_0010_1000);

    assert.eq(SwitchOutput::Square.mask(), 1 << 4);
    assert.eq(SwitchOutput::Attenuator.mask(), 1 << 5);
    assert.eq(SwitchOutput::External.mask(), 1 << 6);
    assert.eq(SwitchOutput::Offset.mask(), 1 << 7);

    assert.eq(PanelRequestCode::Released.as_byte(), 64);
    assert.eq(PanelRequestCode::Left.as_byte(), 65);
    assert.eq(PanelRequestCode::Right.as_byte(), 66);
    assert.eq(PanelRequestCode::PanelActive.as_byte(), 67);
    assert.finish();
}

#[test]
fn parse_sub_ch_accepts_unprefixed_command_after_current_channel_matches() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState::default();
    let mut hw = DummyHardware::default();

    state.current_ch = state.slave_ch;
    state.ser_inp_str = "FRQ?".to_string();
    state.parse_sub_ch(&mut hw);

    assert.eq(state.sub_ch, 0);
    assert.is_true(hw.serial.contains("#0:0=1000"));
    assert.finish();
}

#[test]
fn parse_sub_ch_accepts_unprefixed_direct_subchannel_after_current_channel_matches() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState::default();
    let mut hw = DummyHardware::default();

    state.current_ch = state.slave_ch;
    state.ser_inp_str = "4?".to_string();
    state.parse_sub_ch(&mut hw);

    assert.eq(state.sub_ch, 4);
    assert.is_true(hw.serial.contains("#0:4=3"));
    assert.finish();
}

#[test]
fn parse_set_param_persists_encoder_detent_default() {
    let mut assert = TestFailures::default();

    let mut state = FirmwareState::default();
    let mut hw = DummyHardware::default();

    state.ser_inp_str = "0:250=1".to_string();
    state.parse_sub_ch(&mut hw);
    state.ser_inp_str = "0:89=7".to_string();
    state.parse_sub_ch(&mut hw);
    state.inc_rast = 1;

    state.patch_copy_from_ee();

    assert.eq(state.inc_rast, 7);
    assert.eq(state.defaults.init_inc_rast, 7);
    assert.is_false(state.status.ee_unlocked);
    assert.finish();
}
