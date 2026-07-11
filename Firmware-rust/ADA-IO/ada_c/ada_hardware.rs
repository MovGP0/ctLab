//! Defines ADA the board-I/O contract that separates protocol logic from register access.

#[allow(unused_imports)]
use super::*;

/// Separates ADA protocol/state logic from converters, expanders, UART, trigger GPIO, and board detection.
pub trait AdaHardware {
    /// Starts and returns one conversion from one-based AVR ADC channel 1..8.
    fn get_adc(&mut self, channel_1_based: u8) -> i16;

    /// Transfers TWI out using the byte order expected by the attached peripheral; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn twi_out(&mut self, slave_addr: u8, command: u16) -> bool;

    /// Transmits all eight local output bytes to the cascaded 4094 registers and latches them together.
    fn shift_out_sr(&mut self, port_array: &[u8; 8]);

    /// Returns io pin from the selected local port or I2C expander cache; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_io_pin(&mut self, port: u8) -> u8;

    /// Writes io dir to the serial, display, or peripheral destination selected by the implementation; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn write_io_dir(&mut self, port: u8, value: u8);

    /// Configures I2C expander before code that relies on that hardware capability runs; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn detect_i2c_expander(&mut self) -> bool;

    /// Configures sense before code that relies on that hardware capability runs; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn detect_sense(&mut self) -> bool;

    /// Reads the address strap pins once so serial routing uses the instrument's physical channel number; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_slave_channel(&mut self) -> u8;

    /// Selects the external INT2 polarity requested by `TRL`: false for falling, true for rising.
    fn set_external_trigger_edge(&mut self, positive: bool);

    /// Configures interrupts before code that relies on that hardware capability runs; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn enable_interrupts(&mut self);

    /// Encodes TWI inp byte in the compact representation consumed by registers or the serial protocol; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn twi_inp_byte(&mut self, slave_addr: u8) -> u8;

    /// Transfers TWI inp word using the byte order expected by the attached peripheral; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn twi_inp_word(&mut self, slave_addr: u8) -> u16;

    /// Polls the serial receiver for one byte while respecting the caller's timeout; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn serial_read_byte_timeout(&mut self, timeout_ms: u16) -> Option<u8>;

    /// Appends text to the active serial frame without changing parser state; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn serial_write(&mut self, text: &str);

    /// Programs the UART divisor and U2X mode restored from EEPROM or changed by `SBD`.
    fn set_serial_baud(&mut self, ubrr: u8, double_speed: bool);

    /// Selects whether ADC10 conversions use the AVR internal 2.56 V reference.
    fn set_internal_reference(&mut self, internal: bool);

    /// Drives the shared DAC/4094 serial-data output pin.
    fn set_sdataout(&mut self, high: bool);

    /// Drives the installed DAC's latch/chip-select strobe.
    fn set_str_dac(&mut self, high: bool);

    /// Drives the LTC1864 ADC's active-low strobe/chip-select.
    fn set_str_ad16(&mut self, high: bool);

    /// Drives the trigger LED for the 30-systick indication window after a scan starts.
    fn set_trigger_led(&mut self, active: bool);
}
