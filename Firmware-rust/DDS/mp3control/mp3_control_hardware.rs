//! Defines the pin and timing operations required by the YAMPP auxiliary link.

/// Minimal timing and pin boundary needed by the legacy MP3 auxiliary serial link.
pub trait Mp3ControlHardware {
    /// Drives the YAMPP auxiliary transmit line, whose idle state is high and whose data bits are sent LSB first.
    fn set_ser_aux(&mut self, high: bool);

    /// Supplies the sub-millisecond edge spacing required by the bit-banged MP3 auxiliary UART.
    fn micro_delay(&mut self, ticks: u8);

    /// Supplies command-to-command settling time required by the MP3 decoder after power or track changes.
    fn milli_delay(&mut self, ticks: u16);

    /// Latches the MP3 board's power/control shadow byte after it changes, keeping decoder power state synchronized with the firmware latch.
    fn send_shift_register(&mut self);
}
