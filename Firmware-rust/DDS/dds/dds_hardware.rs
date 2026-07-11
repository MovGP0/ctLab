//! Defines the DDS foreground hardware seam for serial, panel, measurement, and output I/O.

use super::*;

/// Functional hardware boundary for the DDS foreground state machine, including serial, LCD, measurement, and output control.
pub trait DdsHardware {
    /// Writes the complete DDS tuning word derived from the tenths-hertz protocol setpoint.
    fn send_dds_frequency_word(&mut self, word: u32);

    /// Writes the calibrated output-level word after attenuation/range decisions have been made.
    fn send_amplitude_word(&mut self, word: u16);

    /// Writes the AD9833 mode and relay routing chosen by the validated waveform state.
    fn set_waveform(&mut self, waveform: Waveform);

    /// Switches the analog input gain network after the state machine has selected its matching calibration factor.
    fn set_input_range(&mut self, range: InputRange);

    /// Transmits the auxiliary control byte used by the external board after parser validation.
    fn send_aux_config(&mut self, value: u8);

    /// Returns the calibrated AC input level selected by the active gain range for overload and display reporting.
    fn read_input_level(&mut self) -> Float;

    /// Samples the hardware overload indication separately from level magnitude because a clipped converter cannot report its true input.
    fn read_input_overload(&mut self) -> bool;

    /// Emits one byte on the instrument UART; framing and reply ordering remain the caller's responsibility.
    fn serial_write(&mut self, text: &str);

    /// Consumes one already-available UART byte used by the DDS parser's bounded foreground receive loop.
    fn serial_read(&mut self) -> Option<char>;

    /// Programs the AVR baud divisor and double-speed mode together because either value alone would produce the wrong wire rate.
    fn set_serial_baud_register(&mut self, register: u8, double_speed: bool);

    /// Reads the address straps used to decide whether an incoming frame belongs to this instrument.
    fn read_slave_channel(&mut self) -> u8;

    /// Drives the active-low panel LED used to expose local/serial activity without coupling protocol code to a port bit.
    fn set_activity_led(&mut self, enabled: bool);

    /// Provides board-timed settling while keeping the delay mechanism replaceable in host tests.
    fn delay_ms(&mut self, milliseconds: u16);

    /// Probes and initializes the optional LCD; its result gates all later panel writes on headless builds.
    fn lcd_setup(&mut self) -> bool;

    /// Loads one CGRAM glyph used by the original panel cursor and waveform indicators.
    fn lcd_define_custom_char(&mut self, slot: u8, bitmap: [u8; 8]);

    /// Replaces a fixed LCD row in one operation, avoiding partially refreshed values during foreground updates.
    fn lcd_write_line(&mut self, row: u8, text: &str);
}
