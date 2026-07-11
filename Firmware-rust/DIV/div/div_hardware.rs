//! Defines DIV the board-I/O contract that separates protocol logic from register access.

#[allow(unused_imports)]
use super::*;

/// Defines the div hardware boundary so translated timing and protocol logic can run against AVR registers or deterministic host doubles.
pub trait DivHardware {
    /// Starts and returns one conversion from one-based AVR ADC channel 1..8.
    fn read_adc10(&mut self, channel_1_based: u8) -> i16;

    /// Returns the latest LTC2400 sample so callers use the intended display or trigger integration mode; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_adc24(&mut self) -> i32;

    /// Returns the fast integrated LTC2400 sample so callers use the intended display or trigger integration mode; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_adc24_fast_integrated(&mut self) -> i32;

    /// Returns the slow integrated LTC2400 sample so callers use the intended display or trigger integration mode; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_adc24_slow_integrated(&mut self) -> i32;

    /// Exposes the latched LTC2400 polarity or clipping state captured with the conversion sample; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn adc24_overload_negative(&self) -> bool;

    /// Exposes the latched LTC2400 polarity or clipping state captured with the conversion sample; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn adc24_overload_positive(&self) -> bool;

    /// Clears adc10 ready before the next operation is allowed to complete; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn clear_adc10_ready(&mut self);

    /// Returns adc10 ready so the caller can gate the next protocol or conversion step; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn adc10_ready(&mut self) -> bool;

    /// Clears adc24 ready before the next operation is allowed to complete; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn clear_adc24_ready(&mut self);

    /// Returns adc24 ready so the caller can gate the next protocol or conversion step; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn adc24_ready(&mut self) -> bool;

    /// Applies the Port A and Port C relay/gain bit patterns for one DIV measurement range.
    fn set_range_config(&mut self, config: RangeRelayConfig);

    /// Selects the INT2 trigger polarity: false for falling edge, true for rising edge.
    fn set_trigger_edge(&mut self, positive_edge: bool);

    /// Encodes poll serial byte in the compact representation consumed by registers or the serial protocol; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn poll_serial_byte(&mut self) -> Option<u8>;

    /// Appends text to the active serial frame without changing parser state; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn serial_write(&mut self, text: &str);

    /// Renders LCD write line into the fixed LCD cells used by the front panel; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn lcd_write_line(&mut self, row: u8, text: &str);
}
