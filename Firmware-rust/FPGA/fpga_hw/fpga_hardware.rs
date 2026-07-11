/// Electrical operations required by the FPGA controller bus.

///
/// None of these operations has a default because omitting a select pulse,
/// configuration edge, or delay would silently corrupt hardware behavior.
pub trait FpgaHardware
{
    /// Sends one byte through the ATmega UART when serial traffic is externally routed.
    fn external_serial_write(&mut self, byte: u8);

    /// Clocks the register-select transaction that precedes every FPGA data exchange.
    fn select_fpga_register(&mut self, register: u8);

    /// Exchanges equally sized SPI payloads while the selected register is active.
    fn exchange_fpga_data(&mut self, tx: &[u8], rx: &mut [u8]);

    /// Shifts one configuration bitstream byte in original file order.
    fn shift_configuration_byte(&mut self, byte: u8);

    /// Drives the active-low `PROG` line used to reset and start FPGA configuration.
    fn set_configuration_program(&mut self, high: bool);

    /// Samples `DONE` so streaming stops only after the FPGA accepts its image.
    fn configuration_done(&self) -> bool;

    /// Holds `PROG` transitions for FPGA startup and limits internal serial output to 10 kcharacters/s.
    fn delay_us(&mut self, microseconds: u16);
}
