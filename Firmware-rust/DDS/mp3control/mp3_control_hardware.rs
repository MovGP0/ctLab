pub trait Mp3ControlHardware {
    fn set_ser_aux(&mut self, high: bool);
    fn micro_delay(&mut self, ticks: u8);
    fn milli_delay(&mut self, ticks: u16);
    fn send_shift_register(&mut self);
}
