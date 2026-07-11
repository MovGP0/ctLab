//! Defines DIV command parsing and dispatch for the instrument serial protocol.

#[allow(unused_imports)]
use super::*;

/// Defines the div parser hooks boundary so translated timing and protocol logic can run against AVR registers or deterministic host doubles.
pub trait DivParserHooks {
    /// Reports whether busy without mutating device state; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn is_busy(&self) -> bool;

    /// Returns the latest LTC2400 sample so callers use the intended display or trigger integration mode; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_ad24(&mut self, sub_ch: u8, state: &mut ParserState);

    /// Waits for ad24 so callers cannot consume a stale hardware result; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn wait_ad24(&mut self, state: &mut ParserState);

    /// Waits for ad10 so callers cannot consume a stale hardware result; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn wait_ad10(&mut self, state: &mut ParserState);

    /// Obtains ad10 from the owning state or hardware register for the caller that consumes it; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_ad10(&mut self, channel: u8, state: &mut ParserState);

    /// Starts and returns one conversion from one-based AVR ADC channel 1..8.
    fn get_adc(&mut self, channel: u8) -> i32;

    /// Applies the active range offset and scale while producing parameter scale24; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn param_scale24(&mut self, state: &mut ParserState);

    /// Applies the active range offset and scale while producing parameter scale10; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn param_scale10(&mut self, state: &mut ParserState);

    /// Reports whether ac range without mutating device state; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn is_ac_range(&self, state: &ParserState) -> bool;

    /// Returns range from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_range(&self) -> u8;

    /// Returns offset24 from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_offset24(&self, index: usize) -> i32;

    /// Stores the signed LTC2400 count correction for one of the 16 range slots.
    fn set_offset24(&mut self, index: usize, value: i32);

    /// Returns offset10 from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_offset10(&self, index: usize) -> i32;

    /// Stores the signed ADC10 count correction for one of the 16 range slots.
    fn set_offset10(&mut self, index: usize, value: i32);

    /// Returns scale24 from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_scale24(&self, index: usize) -> f32;

    /// Stores the LTC2400 calibration multiplier for one of the 16 range slots.
    fn set_scale24(&mut self, index: usize, value: f32);

    /// Returns scale10 from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_scale10(&self, index: usize) -> f32;

    /// Stores the ADC10 calibration multiplier for one of the 16 range slots.
    fn set_scale10(&mut self, index: usize, value: f32);

    /// Returns trigger mask from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_trigger_mask(&self) -> u8;

    /// Stores `TRM` bits 0..2, enabling AD24, ADC10 RMS, and ADC10 peak responses respectively.
    fn set_trigger_mask(&mut self, value: u8);

    /// Returns trigger timer value from live device or EEPROM-backed state so the standalone parser does not maintain a divergent copy; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn get_trigger_timer_value(&self) -> u16;

    /// Stores the `TRT` automatic-trigger interval in milliseconds; zero disables it.
    fn set_trigger_timer_value(&mut self, value: u16);

    /// Latches trigger now for deferred processing outside the interrupt-sensitive edge handler; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn trigger_now(&mut self);

    /// Validates limits before dependent hardware state is changed; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn check_limits(&mut self, state: &mut ParserState);

    /// Applies range as one coherent state and hardware transition; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn switch_range(&mut self, state: &mut ParserState);

    /// Emits show range using the exact channel and status framing expected by existing clients; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn show_range(&mut self, state: &mut ParserState);

    /// Stores the parser's working frame, command, calibration, and response values.
    fn write_param_long_int_ser(&mut self, state: &ParserState);

    /// Stores the parser's working frame, command, calibration, and response values.
    fn write_param_ser(&mut self, state: &ParserState, overload: bool);

    /// Writes the addressed channel prefix before a payload so every response keeps the Pascal wire framing; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn write_ch_prefix(&mut self, state: &ParserState);

    /// Writes serial inp to the serial, display, or peripheral destination selected by the implementation; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn write_ser_inp(&mut self, input: &str);

    /// Writes string to the serial, display, or peripheral destination selected by the implementation; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn write_str(&mut self, text: &str);

    /// Terminates the current serial response with CRLF because existing clients parse line-delimited frames; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn ser_crlf(&mut self);

    /// Emits serprompt using the exact channel and status framing expected by existing clients; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn serprompt(&mut self, state: &mut ParserState, err: ParserError);

    /// Loads the remaining activity-indicator hold time after a valid command.
    fn set_activity_timer(&mut self, ticks: u8);

    /// Drives the active-low command-activity LED to its asserted level.
    fn set_activity_led_low(&mut self);
}
