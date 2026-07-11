pub trait FpgaHardware
{
    fn external_serial_write(&mut self, byte: u8);

    fn select_fpga_register(&mut self, register: u8);

    fn exchange_fpga_data(&mut self, tx: &[u8], rx: &mut [u8]);

    fn shift_configuration_byte(&mut self, byte: u8);

    fn set_configuration_program(&mut self, high: bool);

    fn configuration_done(&self) -> bool;

    fn delay_us(&mut self, microseconds: u16);
}
