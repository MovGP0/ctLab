//! Best-effort Rust port of `EDL.pas`.
//!
//! The Pascal source is a large foreground loop wrapped around timer-driven ADC
//! and DAC state. This module restores the audited main-file semantics with an
//! explicit state machine: per-fault latching, calibration derived from the
//! documented EEPROM layout, ripple/off-phase sampling, averaged power, range
//! selection, periodic telemetry, and Ah/Wh integration.

#![allow(dead_code)]

/// Firmware arithmetic precision, matching the AVR Pascal `Single` representation.
pub type Float = f32;

/// CPU clock used to derive timer and serial timing.
pub const PROC_CLOCK: u32 = 16_000_000;

/// Full serial identification returned by `IDN`.
pub const VERS1_STR: &str = "1.784 [EDL by CM/c't 09/2008]";

/// Short front-panel version shown during startup.
pub const VERS3_STR: &str = "EDL 1.78";

/// Protocol subchannel carrying errors and packed status bits.
pub const ERR_SUBCH: u8 = 255;

/// Sentinel selecting automatic shunt range calculation.
pub const AUTO_SHUNT_RANGE: u8 = 4;

/// Number of the lowest-current/highest-resistance shunt.
pub const SHUNT_D: u8 = 3;

/// Foreground measurement/control cadence inherited from the Pascal service loop.
pub const SERVICE_INTERVAL_MS: u32 = 40;

/// Ah/Wh integration cadence, kept separate from faster regulation updates.
pub const INTEGRATION_INTERVAL_MS: u32 = 200;

/// Service cycles between unsolicited measurement frames.
pub const PERIODIC_TELEMETRY_CYCLES: u8 = 10;

/// Service cycles between relatively slow temperature reads.
pub const TEMPERATURE_POLL_CYCLES: u8 = 20;

/// Absolute software over-temperature trip independent of LM75 output polarity.
pub const TEMPERATURE_MAX_C: Float = 70.0;

const DAC_TYPE_MASK: u8 = 0x03;
const LM75_INTERNAL_BIT: u8 = 1 << 2;
const LM75_EXTERNAL_BIT: u8 = 1 << 3;
const LM75_INTERNAL_ADDRESS: u8 = 0x49;
const LM75_EXTERNAL_ADDRESS: u8 = 0x48;
const LM75_TEMPERATURE_REGISTER: u8 = 0;
const LM75_CONFIGURATION_REGISTER: u8 = 1;
const LM75_HYSTERESIS_REGISTER: u8 = 2;
const LM75_OVERTEMPERATURE_REGISTER: u8 = 3;
const LM75_INVERTED_OUTPUT_CONFIGURATION: u8 = 4;
const LM75_HYSTERESIS_C: Float = 3.0;

const OPT_INIT_VOLT: usize = 0;
const OPT_INIT_AMP: usize = 1;
const OPT_INIT_LOW_DIVIDER_U: usize = 2;
const OPT_INIT_HI_DIVIDER_U: usize = 3;
const OPT_INIT_GAIN_I: usize = 4;
const OPT_UREF: usize = 5;
const OPT_PMAX: usize = 6;
const OPT_RSENSE_BASE: usize = 7;
const OPT_IMAX_BASE: usize = 11;
const OPT_UMAX_HI: usize = 15;
const OPT_UMAX_LO: usize = 16;
const OPT_INIT_OPTIONS: usize = 17;
const OPT_INIT_IPERCENT: usize = 18;
const OPT_INIT_TON: usize = 19;
const OPT_INIT_TOFF: usize = 20;
const OPT_INIT_FAN_TEMP: usize = 21;

/// Command identities kept aligned with the Pascal mnemonic table.
#[path = "edl/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;

/// Regulation law and voltage-range modes.
#[path = "edl/mode.rs"]
mod mode;
pub use mode::Mode;

/// Front-panel encoder/menu targets.
#[path = "edl/modify.rs"]
mod modify;
pub use modify::Modify;

/// Protocol error discriminants and string-table indices.
#[path = "edl/error_code.rs"]
mod error_code;
pub use error_code::ErrorCode;

/// Signal/phase tags for converter samples.
#[path = "edl/measure_kind.rs"]
mod measure_kind;
pub use measure_kind::MeasureKind;

/// Installed DAC wire protocol selection.
#[path = "edl/dac_kind.rs"]
mod dac_kind;
pub use dac_kind::DacKind;

/// Independently latched output protection causes.
#[path = "edl/protection_flags.rs"]
mod protection_flags;
pub use protection_flags::ProtectionFlags;

/// Fault identities keeping status bits and exact wire labels in one exhaustive match.
#[path = "edl/protection_fault.rs"]
mod protection_fault;
pub use protection_fault::ProtectionFault;

/// Persistent calibration and startup layout.
#[path = "edl/eeprom_data.rs"]
mod eeprom_data;
pub use eeprom_data::EepromData;

/// Runtime conversion factors derived from EEPROM.
#[path = "edl/scale_state.rs"]
mod scale_state;
pub use scale_state::ScaleState;

/// Volatile high-nibble protocol status flags.
#[path = "edl/runtime_status.rs"]
mod runtime_status;
pub use runtime_status::RuntimeStatus;

/// Hardware effects required by the foreground control state.
#[path = "edl/edl_hardware.rs"]
mod edl_hardware;
pub use edl_hardware::EdlHardware;

/// Coherent on/off-phase measurement results.
#[path = "edl/measurement_snapshot.rs"]
mod measurement_snapshot;
pub use measurement_snapshot::MeasurementSnapshot;

/// Foreground regulation, protection, UI, and telemetry state machine.
#[path = "edl/device_state.rs"]
mod device_state;
pub use device_state::DeviceState;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, Clone)]
    struct MockHardware {
        voltage_on: u16,
        voltage_off: u16,
        current_on: u16,
        current_off: u16,
        voltage10: i16,
        current10: i16,
        temp_c: Option<Float>,
        trigger_in: bool,
        shunts: Vec<u8>,
        outputs: Vec<bool>,
        dacs: Vec<u16>,
        lm75_writes: Vec<(u8, u8, Vec<u8>)>,
        serial: Vec<String>,
        lcd: Vec<(u8, String)>,
    }

    impl MockHardware {
        fn serial_joined(&self) -> String {
            self.serial.join("")
        }
    }

    impl EdlHardware for MockHardware {
        fn read_voltage_adc16(&mut self, on_phase: bool) -> u16 {
            if on_phase {
                self.voltage_on
            } else {
                self.voltage_off
            }
        }

        fn read_current_adc16(&mut self, on_phase: bool) -> u16 {
            if on_phase {
                self.current_on
            } else {
                self.current_off
            }
        }

        fn read_voltage_adc10(&mut self) -> i16 {
            self.voltage10
        }

        fn read_current_adc10(&mut self) -> i16 {
            self.current10
        }

        fn set_shunt(&mut self, shunt_index: u8) {
            self.shunts.push(shunt_index);
        }

        fn set_output_enabled(&mut self, enabled: bool) {
            self.outputs.push(enabled);
        }

        fn set_dac_raw(&mut self, raw: u16) {
            self.dacs.push(raw);
        }

        fn read_temp_c(&mut self) -> Option<Float> {
            self.temp_c
        }

        fn lm75_write(&mut self, address: u8, register: u8, data: &[u8]) {
            self.lm75_writes.push((address, register, data.to_vec()));
        }

        fn serial_write(&mut self, text: &str) {
            self.serial.push(text.to_string());
        }

        fn lcd_write_line(&mut self, row: u8, text: &str) {
            self.lcd.push((row, text.to_string()));
        }

        fn read_trigger_in(&mut self) -> bool {
            self.trigger_in
        }
    }

    fn test_eeprom() -> EepromData {
        let mut eeprom = EepromData::default();
        eeprom.adc_u_offsets = [0, 0];
        eeprom.adc_u_scales = [1.0, 1.0];
        eeprom.adc_i_offsets = [0, 0, 0, 0];
        eeprom.adc_i_scales = [1.0, 1.0, 1.0, 1.0];
        eeprom.dac_i_offsets = [0, 0, 0, 0];
        eeprom.dac_i_scales = [1.0, 1.0, 1.0, 1.0];
        eeprom.option_array[OPT_INIT_LOW_DIVIDER_U] = 1.0;
        eeprom.option_array[OPT_INIT_HI_DIVIDER_U] = 1.0;
        eeprom.option_array[OPT_INIT_GAIN_I] = 1.0;
        eeprom.option_array[OPT_UREF] = 1.0;
        eeprom.option_array[OPT_PMAX] = 2.0;
        eeprom.option_array[OPT_RSENSE_BASE..OPT_RSENSE_BASE + 4]
            .copy_from_slice(&[10.0, 1.0, 0.5, 0.1]);
        eeprom.option_array[OPT_IMAX_BASE..OPT_IMAX_BASE + 4]
            .copy_from_slice(&[0.01, 0.1, 0.5, 2.0]);
        eeprom.option_array[OPT_UMAX_HI] = 2.0;
        eeprom.option_array[OPT_UMAX_LO] = 1.0;
        eeprom.option_array[OPT_INIT_IPERCENT] = 100.0;
        eeprom.option_array[OPT_INIT_TON] = 10.0;
        eeprom.option_array[OPT_INIT_TOFF] = 0.0;
        eeprom.option_array[OPT_INIT_OPTIONS] = 4.0;
        eeprom
    }

    #[test]
    fn default_eeprom_matches_pascal_layout() {
        let eeprom = EepromData::default();
        assert_eq!(eeprom.dac_i_offsets, [0, 0, 0, 0]);
        assert_eq!(eeprom.adc_u_offsets, [-260, -260]);
        assert_eq!(eeprom.option_array[OPT_INIT_AMP], 0.02);
        assert_eq!(eeprom.option_array[OPT_RSENSE_BASE], 100.0);
        assert_eq!(eeprom.option_array[OPT_RSENSE_BASE + 3], 0.1);
        assert_eq!(eeprom.option_array[OPT_IMAX_BASE], 0.002);
        assert_eq!(eeprom.option_array[OPT_IMAX_BASE + 3], 2.0);
        assert_eq!(eeprom.option_array[OPT_UMAX_HI], 25.0);
        assert_eq!(eeprom.option_array[OPT_UMAX_LO], 6.1);
        assert_eq!(eeprom.option_array[OPT_INIT_OPTIONS], 4.0);
        assert_eq!(eeprom.option_array[OPT_INIT_TON], 10.0);
        assert_eq!(eeprom.option_array[OPT_INIT_TOFF], 0.0);
        assert_eq!(eeprom.option_array[OPT_INIT_FAN_TEMP], 50.0);
    }

    #[test]
    fn set_lm75_temp_programs_pascal_threshold_hysteresis_and_pointer_sequence() {
        let hw = MockHardware::default();
        let mut eeprom = test_eeprom();
        eeprom.option_array[OPT_INIT_OPTIONS] = Float::from(LM75_INTERNAL_BIT | LM75_EXTERNAL_BIT);
        eeprom.option_array[OPT_INIT_FAN_TEMP] = 50.0;
        let mut state = DeviceState::with_eeprom(hw, eeprom);
        state.hw.lm75_writes.clear();

        state.set_lm75_temp();

        let one_sensor = |address| {
            vec![
                (
                    address,
                    LM75_CONFIGURATION_REGISTER,
                    vec![LM75_INVERTED_OUTPUT_CONFIGURATION],
                ),
                (address, LM75_OVERTEMPERATURE_REGISTER, vec![0x32, 0x00]),
                (address, LM75_HYSTERESIS_REGISTER, vec![0x2f, 0x00]),
                (address, LM75_TEMPERATURE_REGISTER, vec![]),
            ]
        };
        let mut expected = one_sensor(LM75_INTERNAL_ADDRESS);
        expected.extend(one_sensor(LM75_EXTERNAL_ADDRESS));
        assert_eq!(state.hw.lm75_writes, expected);
    }

    #[test]
    fn encoder_rounding_and_acceleration_match_pascal_current_adjustment() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.mode = Mode::IHiVolt;
        state.modify = Modify::LowerMainMenu;
        state.current_set = 0.12346;
        state.first_turn = true;

        state.inc_fac_i();
        state.round_inc_param();

        assert_eq!(state.inc_coarse_div, 100.0);
        assert_eq!(state.inc_fine_div, 10_000.0);
        assert!((state.current_set - 0.12).abs() < 0.000_001);
        assert!(!state.first_turn);

        state.incr_acc_float = 5.0;
        state.set_acc_param();
        assert!((state.incr_acc_float - 0.05).abs() < 0.000_001);
        assert!((state.current_set - 0.17).abs() < 0.000_001);
    }

    #[test]
    fn encoder_fine_adjustment_skips_coarse_rounding_and_uses_fine_divisor() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.mode = Mode::PHiVolt;
        state.modify = Modify::LowerMainMenu;
        state.power_set = 12.345;
        state.incr_fine = true;
        state.first_turn = true;

        state.inc_fac_p();
        state.round_inc_param();
        state.incr_acc_float = -2.0;
        state.set_acc_param();

        assert_eq!(state.inc_coarse_div, 10.0);
        assert_eq!(state.inc_fine_div, 100.0);
        assert!(state.first_turn);
        assert!((state.incr_acc_float + 0.02).abs() < 0.000_001);
        assert!((state.power_set - 12.325).abs() < 0.000_001);
    }

    #[test]
    fn check_limits_restores_clamps_and_ripple_normalization() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.mode = Mode::RLoVolt;
        state.init_scales();
        state.resistance_set = 0.01;
        state.current_set = 5.0;
        state.i_percent = 150;
        state.pw_on_time_ms = 0;
        state.pw_off_time_ms = -5;
        state.power_set = 5.0;
        state.voltage_cutoff = 5.0;
        state.track_channel = 9;

        let err = state.check_limits();

        assert_eq!(err, ErrorCode::ParamErr);
        assert_eq!(state.resistance_set, state.scale.dc_ohm_min);
        assert_eq!(state.current_set, 2.0);
        assert_eq!(state.i_percent, 100);
        assert_eq!(state.pw_on_time_ms, 1);
        assert_eq!(state.pw_off_time_ms, 0);
        assert_eq!(state.power_set, 2.0);
        assert_eq!(state.voltage_cutoff, 1.0);
        assert_eq!(state.track_channel, 7);
        assert!(state.no_toggle);
    }

    #[test]
    fn status_prompt_reports_each_fault_instead_of_single_overload_bit() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.faults = ProtectionFlags {
            over_power: true,
            fuse_blown: false,
            over_voltage: true,
            over_temp: true,
            low_volt: true,
        };
        state.status.overload_flag = true;
        let frame = state.status_frame(ErrorCode::NoErr);

        assert!(frame.contains("[OVRPOWR]"));
        assert!(frame.contains("[OVRVOLT]"));
        assert!(frame.contains("[OVRTEMP]"));
        assert!(frame.contains("[LOWVOLT]"));
        assert!(frame.starts_with("#0:255=61 "));
    }

    #[test]
    fn current_and_resistance_modes_restore_auto_range_behavior() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());

        state.mode = Mode::IHiVolt;
        state.current_set = 0.2;
        state.i_percent = 50;
        state.pw_off_time_ms = 10;
        state.check_limits();
        state.set_level_dac_i();
        assert_eq!(state.shunt_select, 2);
        assert!(state.dac_raw_on > state.dac_raw_off);

        state.mode = Mode::RLoVolt;
        state.init_scales();
        state.resistance_set = 0.05;
        state.check_limits();
        state.set_level_dac_r();
        assert_eq!(state.shunt_select, 3);
        assert_eq!(state.dac_raw_on, state.dac_raw_off);
        assert_eq!(state.hw.shunts.last().copied(), Some(3));
    }

    #[test]
    fn init_all_restores_startup_constants_and_banner() {
        let hw = MockHardware::default();
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());

        state.init_all();

        assert_eq!(state.mode, Mode::IHiVolt);
        assert!(state.output_enabled);
        assert_eq!(state.current_set, state.eeprom.init_amp());
        assert_eq!(state.shunt_range, AUTO_SHUNT_RANGE);
        assert_eq!(state.capacity_ah, 0.0);
        assert_eq!(state.capacity_wh, 0.0);
        assert!(state.hw.serial_joined().contains(VERS1_STR));
        assert!(state
            .hw
            .lcd
            .iter()
            .any(|(row, text)| *row == 0 && text == VERS3_STR));
    }

    #[test]
    fn service_cycle_restores_off_phase_sampling_average_power_and_telemetry() {
        let hw = MockHardware {
            voltage_on: 32_768,
            voltage_off: 65_535,
            current_on: 65_535,
            current_off: 32_768,
            ..MockHardware::default()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_all();
        state.mode = Mode::IHiVolt;
        state.output_enabled = true;
        state.i_percent = 50;
        state.pw_on_time_ms = 3;
        state.pw_off_time_ms = 1;
        state.set_trigger_mask(0x02);
        state.check_limits();
        state.service_cycle();

        assert!((state.last_measurement.voltage_on - 32_768.0 / 65_535.0).abs() < 0.0001);
        assert!((state.last_measurement.voltage_off - 1.0).abs() < 0.0001);
        assert!((state.measured_power - 0.5).abs() < 0.0001);
        assert!((state.internal_resistance - 1.0).abs() < 0.0002);
        let serial = state.hw.serial_joined();
        assert!(serial.contains("#0:10="));
        assert!(serial.contains("#0:11="));
        assert!(serial.contains("#0:15="));
        assert!(serial.contains("#0:16="));
    }

    #[test]
    fn output_off_zeroes_current_and_normalizes_off_phase_when_ripple_is_disabled() {
        let hw = MockHardware {
            voltage_on: 40_000,
            voltage_off: 50_000,
            current_on: 30_000,
            current_off: 10_000,
            ..MockHardware::default()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_all();
        state.output_enabled = false;
        state.i_percent = 100;
        state.pw_off_time_ms = 0;
        state.check_limits();

        let on_voltage = state.read_voltage_phase(true);
        let off_voltage = state.read_voltage_phase(false);
        let off_current = state.read_current_phase(false);
        state.service_cycle();

        assert!((on_voltage - off_voltage).abs() < 0.0001);
        assert_eq!(off_current, 0.0);
        assert_eq!(state.measured_current, 0.0);
        assert_eq!(state.measured_power, 0.0);
    }

    #[test]
    fn energy_integration_accumulates_and_resets_like_pascal() {
        let hw = MockHardware {
            voltage_on: 65_535,
            voltage_off: 65_535,
            current_on: 65_535,
            current_off: 65_535,
            ..MockHardware::default()
        };
        let mut state = DeviceState::with_eeprom(hw, test_eeprom());
        state.init_all();
        state.output_enabled = true;
        state.i_percent = 100;
        state.pw_off_time_ms = 0;
        state.check_limits();
        state.service_step(200);

        assert!(state.capacity_ah > 0.0);
        assert!(state.capacity_wh > 0.0);

        state.i_percent = 50;
        state.pw_off_time_ms = 10;
        state.check_limits();
        state.service_step(200);
        assert_eq!(state.capacity_ah, 0.0);
        assert_eq!(state.capacity_wh, 0.0);

        state.capacity_ah = 1.0;
        state.capacity_wh = 2.0;
        state.reset_energy_counters();
        assert_eq!(state.capacity_ah, 0.0);
        assert_eq!(state.capacity_wh, 0.0);
    }
}
