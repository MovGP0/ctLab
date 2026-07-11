//! Defines every SQG timer, serial, panel, LCD, converter, and relay operation required by the foreground state machine.

use super::*;

/// SQG hardware contract. Required methods make missing DAC, serial, timer, or panel behavior a compile-time error.
pub(super) trait HardwareInterface {
    /// Writes one framing byte directly so CR/LF emission does not require allocating a temporary string.
    fn serout_byte(&mut self, byte: u8);

    /// Writes a complete text fragment to the UART without adding addressing or line termination.
    fn write_serial(&mut self, text: &str);

    /// Shifts one 16-bit AD9833 frame most-significant byte first while FSYNC is asserted; uninterrupted framing is required for the chip to accept the register write.
    fn send_dds_word(&mut self, word: u16);

    /// Updates the cascaded level/relay shift registers and pulses the latch only after the complete payload is stable.
    fn shift_out_level_sr(&mut self, level: i32, switch_state: u8);

    /// Maps signed offset counts around DAC midscale, shifts the 12-bit value most-significant bit first, and holds the final latch edge for the LTC1257 timing requirement.
    fn shift_out_offset_dac(&mut self, dac_counts: i16);

    /// Waits only for the bounded Pascal receive interval, allowing foreground protection work to resume when a command stalls.
    fn serial_timeout_char(&mut self, timeout_ticks: u8) -> Option<char>;

    /// Reports queued UART work so panel scanning does not compete with command reception on the FIFO-less ATmega UART.
    fn serial_pending(&self) -> bool;

    /// Consumes one pending timer tick, preventing the foreground loop from processing the same burst interval twice.
    fn take_systick(&mut self) -> bool;

    /// Returns one debounced panel transition at a time so button and encoder state changes remain ordered.
    fn next_panel_event(&mut self) -> PanelEvent;

    /// Programs the AVR baud divisor and double-speed mode together because either value alone would produce the wrong wire rate.
    fn set_serial_baud_register(&mut self, register: u8, double_speed: bool);

    /// Reads the address straps used to decide whether an incoming frame belongs to this instrument.
    fn read_slave_channel(&mut self) -> u8;

    /// Drains already-buffered UART bytes during startup without introducing another receive timeout.
    fn serial_read_immediate(&mut self) -> Option<char>;

    /// Probes and initializes the optional LCD; its result gates all later panel writes on headless builds.
    fn lcd_setup(&mut self) -> bool;

    /// Loads one CGRAM glyph used by the original panel cursor and waveform indicators.
    fn lcd_define_custom_char(&mut self, slot: u8, bitmap: [u8; 8]);

    /// Replaces a fixed LCD row in one operation, avoiding partially refreshed values during foreground updates.
    fn lcd_write_line(&mut self, row: u8, text: &str);

    /// Drives the active-low panel LED used to expose local/serial activity without coupling protocol code to a port bit.
    fn set_activity_led(&mut self, active_low: bool);

    /// Provides board-timed settling while keeping the delay mechanism replaceable in host tests.
    fn delay_ms(&mut self, ms: u16);
}
