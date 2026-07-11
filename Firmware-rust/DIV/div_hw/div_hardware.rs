#[allow(unused_imports)]
use super::*;

pub trait DivHardware {
    fn set_str_ad24(&mut self, high: bool);
    fn set_sclk(&mut self, high: bool);
    fn read_sdata_in1(&self) -> bool;
    fn set_spi_control(&mut self, value: u8);
    fn spi_transfer(&mut self, tx: u8) -> u8;
    fn spin_delay_cycles(&mut self, cycles: u16);
}
