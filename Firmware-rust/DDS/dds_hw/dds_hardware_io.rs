use super::*;

pub trait DdsHardwareIo {
    fn set_bit(&mut self, port: PortKind, bit: u8);
    fn clear_bit(&mut self, port: PortKind, bit: u8);
    fn nop(&mut self);
    fn delay_units(&mut self, units: u8);
    fn delay_ms(&mut self, milliseconds: u16);
    fn begin_critical_section(&mut self);
    fn end_critical_section(&mut self);
}
