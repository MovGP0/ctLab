use super::*;

/// Supplies controlled conversions and captures output while parser tests exercise the live DIV runtime adapter.
#[derive(Debug, Clone, Default)]
struct MockHardware {
    /// Queues serial in UART order for the host-side hardware model.
    serial: String,

    /// Records LCD lines in order so tests can verify every externally visible operation.
    lcd_lines: Vec<(u8, String)>,

    /// Tracks last range so conversion, relay, and formatting decisions agree.
    last_range: Option<DivRange>,

    /// Contains ad24 in converter counts until the owning conversion or output routine consumes it.
    ad24: i32,

    /// Contains ad10 in converter counts until the owning conversion or output routine consumes it.
    ad10: [i16; 8],

    /// Indicates whether ad10 ready clears; the producer updates it before consumers choose their next action.
    ad10_ready_clears: usize,

    /// Indicates whether ad24 ready clears; the producer updates it before consumers choose their next action.
    ad24_ready_clears: usize,
}

impl DivRuntimeHardware for MockHardware {
    fn read_adc10(&mut self, channel_1_based: u8) -> i16 {
        self.ad10[channel_1_based as usize]
    }

    /// Returns the latest LTC2400 sample so callers use the intended display or trigger integration mode.
    fn read_adc24(&mut self) -> i32 {
        self.ad24
    }

    /// Returns the fast integrated LTC2400 sample so callers use the intended display or trigger integration mode.
    fn read_adc24_fast_integrated(&mut self) -> i32 {
        self.ad24
    }

    /// Returns the slow integrated LTC2400 sample so callers use the intended display or trigger integration mode.
    fn read_adc24_slow_integrated(&mut self) -> i32 {
        self.ad24
    }

    /// Exposes the latched LTC2400 polarity or clipping state captured with the conversion sample.
    fn adc24_overload_negative(&self) -> bool {
        false
    }

    /// Exposes the latched LTC2400 polarity or clipping state captured with the conversion sample.
    fn adc24_overload_positive(&self) -> bool {
        false
    }

    /// Clears adc10 ready before the next operation is allowed to complete.
    fn clear_adc10_ready(&mut self) {
        self.ad10_ready_clears += 1;
    }

    /// Returns adc10 ready so the caller can gate the next protocol or conversion step.
    fn adc10_ready(&mut self) -> bool {
        true
    }

    /// Clears adc24 ready before the next operation is allowed to complete.
    fn clear_adc24_ready(&mut self) {
        self.ad24_ready_clears += 1;
    }

    /// Returns adc24 ready so the caller can gate the next protocol or conversion step.
    fn adc24_ready(&mut self) -> bool {
        true
    }

    fn set_range_config(&mut self, config: crate::div::RangeRelayConfig) {
        self.last_range = Some(config.range);
    }

    fn set_trigger_edge(&mut self, _positive_edge: bool) {
        // This parser test double does not model an external interrupt pin.
    }

    /// Encodes poll serial byte in the compact representation consumed by registers or the serial protocol.
    fn poll_serial_byte(&mut self) -> Option<u8> {
        None
    }

    /// Appends text to the active serial frame without changing parser state.
    fn serial_write(&mut self, text: &str) {
        self.serial.push_str(text);
    }

    /// Renders LCD write line into the fixed LCD cells used by the front panel.
    fn lcd_write_line(&mut self, row: u8, text: &str) {
        self.lcd_lines.push((row, text.to_string()));
    }
}

/// Loads one complete serial frame into the parser and executes it, keeping parser tests concise and consistent.
fn run_frame(parser: &mut DivParser<DivRuntimeAdapter<'_, MockHardware>>, frame: &str) {
    parser.state.ser_inp_str = frame.to_string();
    parser.parse_sub_ch();
}

fn new_parser() -> DivParser<DivRuntimeAdapter<'static, MockHardware>> {
    let device = Box::new(DivDeviceState::new(MockHardware::default()));
    let leaked = Box::leak(device);
    let hooks = DivRuntimeAdapter::new(leaked);
    let mut parser = DivParser::new(hooks);
    parser.state.slave_ch = 1;
    parser.state.current_ch = 1;
    parser
}

/// Verifies every command owns its mnemonic instead of depending on enum position.
#[test]
fn command_mnemonics_round_trip_through_enum_methods() {
    #[rustfmt::skip]
    let commands = [
        CmdWhich::Str,
        CmdWhich::Idn,
        CmdWhich::Trg,
        CmdWhich::Val,
        CmdWhich::Rng,
        CmdWhich::Dsp,
        CmdWhich::Ofs,
        CmdWhich::Scl,
        CmdWhich::All,
        CmdWhich::Trm,
        CmdWhich::Trt,
        CmdWhich::Trl,
        CmdWhich::Erc,
        CmdWhich::Sbd,
        CmdWhich::Wen,
        CmdWhich::Nop,
    ];

    for command in commands {
        let mnemonic = command.as_str().expect("wire command has a mnemonic");
        assert_eq!(CmdWhich::from_mnemonic(mnemonic), command);
        assert_eq!(
            CmdWhich::from_mnemonic(&mnemonic.to_ascii_lowercase()),
            command
        );
    }
    assert_eq!(CmdWhich::Err.as_str(), None);
    assert_eq!(CmdWhich::from_mnemonic("UNKNOWN"), CmdWhich::Err);
}

/// Verifies status and fault labels are attached to their owning enums.
#[test]
fn status_labels_come_from_typed_variants() {
    assert_eq!(ParserError::NoErr.as_str(), "[OK]");
    assert_eq!(ParserError::ChecksumErr.as_str(), "[CHKSUM]");
    assert_eq!(DivFault::NegativeOverload.as_str(), "[OVRNEG]");
    assert_eq!(DivFault::PositiveOverload.as_str(), "[OVRPOS]");
}

/// Verifies that busy commands fail before execution remains faithful to the Pascal behavior.
#[test]
fn busy_commands_fail_before_execution() {
    let mut parser = new_parser();
    parser.hooks.busy = true;

    run_frame(&mut parser, "1:RNG?");

    assert_eq!(parser.hooks.device.hw.serial, "#1:255=130 [BUSY]\r\n");
    assert_eq!(parser.hooks.activity_timer_ticks, None);
}

/// Verifies that runtime adapter waits use device interrupt handshakes remains faithful to the Pascal behavior.
#[test]
fn runtime_adapter_waits_use_device_irq_handshakes() {
    let mut parser = new_parser();

    parser.hooks.wait_ad10(&mut parser.state);
    parser.hooks.wait_ad24(&mut parser.state);

    assert_eq!(parser.hooks.device.hw.ad10_ready_clears, 1);
    assert_eq!(parser.hooks.device.hw.ad24_ready_clears, 1);
}

/// Verifies that calibration and range writes hit live device state remains faithful to the Pascal behavior.
#[test]
fn calibration_and_range_writes_hit_live_device_state() {
    let mut parser = new_parser();

    run_frame(&mut parser, "1:WEN=1");
    run_frame(&mut parser, "1:RNG=5");
    run_frame(&mut parser, "1:WEN=1");
    run_frame(&mut parser, "1:OFS 0=42");
    run_frame(&mut parser, "1:WEN=1");
    run_frame(&mut parser, "1:OFS 20=7");
    run_frame(&mut parser, "1:WEN=1");
    run_frame(&mut parser, "1:SCL 0=1.5");
    run_frame(&mut parser, "1:WEN=1");
    run_frame(&mut parser, "1:SCL 20=2.5");

    assert_eq!(parser.hooks.device.range, DivRange::Ac2V5);
    assert_eq!(parser.hooks.device.hw.last_range, Some(DivRange::Ac2V5));
    assert_eq!(parser.hooks.device.eeprom.ad24_offsets[0], 42);
    assert_eq!(parser.hooks.device.eeprom.ad10_offsets[0], 7);
    assert_eq!(parser.hooks.device.eeprom.ad24_scales[0], 1.5);
    assert_eq!(parser.hooks.device.eeprom.ad10_scales[0], 2.5);
    assert!(!parser.state.ee_unlocked);
}

/// Verifies that trigger commands update runtime state remains faithful to the Pascal behavior.
#[test]
fn trigger_commands_update_runtime_state() {
    let mut parser = new_parser();

    run_frame(&mut parser, "1:WEN=1");
    run_frame(&mut parser, "1:TRM=3");
    run_frame(&mut parser, "1:WEN=1");
    run_frame(&mut parser, "1:TRT=25");
    run_frame(&mut parser, "1:TRG?");

    assert_eq!(parser.hooks.device.eeprom.trigger_mode, 3);
    assert_eq!(parser.hooks.device.eeprom.trigger_timer_ms, 25);
    assert!(parser.hooks.device.trigger_pending);
    assert!(parser.hooks.device.hw.serial.ends_with("#1:255=0 [OK]\r\n"));
}

/// Verifies that forwarded frames preserve pascal wire format remains faithful to the Pascal behavior.
#[test]
fn forwarded_frames_preserve_pascal_wire_format() {
    let mut parser = new_parser();

    run_frame(&mut parser, "#2:19=5");
    run_frame(&mut parser, "2:IDN?");

    assert_eq!(parser.hooks.device.hw.serial, "#2:19=5\r\n2:IDN?\r\n");
}

/// Verifies that replies use prefixed pascal framing remains faithful to the Pascal behavior.
#[test]
fn replies_use_prefixed_pascal_framing() {
    let mut parser = new_parser();

    run_frame(&mut parser, "1:IDN?");
    run_frame(&mut parser, "1:RNG?");

    assert_eq!(
        parser.hooks.device.hw.serial,
        "#1:254=3.10 [DIV by CM/c't 03/2007] \r\n#1:19=1\r\n"
    );
}
