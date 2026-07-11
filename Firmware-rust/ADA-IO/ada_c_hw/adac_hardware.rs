#[allow(unused_imports)]
use super::*;

pub trait AdacHardware {
    fn set_signal(&mut self, signal: Signal, high: bool);
    fn read_signal(&self, signal: Signal) -> bool;
    fn set_port_c(&mut self, value: Byte);
    fn set_admux(&mut self, value: Byte);
    fn write_adcsra(&mut self, value: Byte);
    fn read_adcsra(&self) -> Byte;
    fn read_adcl(&self) -> Byte;
    fn read_adch(&self) -> Byte;
    fn begin_interrupt_exclusion(&mut self) -> Byte;
    fn end_interrupt_exclusion(&mut self, saved_status: Byte);

    fn nop(&mut self) {
        self.wait_cycles(1);
    }

    fn wait_cycles(&mut self, cycles: u16) {
        for _ in 0..cycles {
            core::hint::spin_loop();
        }
    }

    fn wait_for_adc10_complete(&mut self);
}
