use super::*;

pub trait DcgHardware {
    fn set_sdata_out(&mut self, high: bool);
    fn set_sclk(&mut self, high: bool);
    fn set_str_dac(&mut self, high: bool);
    fn set_str_ad16(&mut self, high: bool);
    fn set_mpx_i(&mut self, high: bool);
    fn set_mpx_u(&mut self, high: bool);
    fn set_mpx_1864(&mut self, high: bool);
    fn read_sdata_in1(&self) -> bool;
    fn spin_delay_cycles(&mut self, cycles: u16);
    fn set_admux(&mut self, value: u8);
    fn write_adcsra(&mut self, value: u8);
    fn read_adcsra(&self) -> u8;
    fn read_adcl(&self) -> u8;
    fn read_adch(&self) -> u8;
    fn begin_interrupt_exclusion(&mut self) -> u8;
    fn end_interrupt_exclusion(&mut self, saved_status: u8);

    fn nop(&mut self) {
        self.spin_delay_cycles(1);
    }

    fn wait_post_dac_settle(&mut self) {
        for _ in 0..DAC_POST_WRITE_SETTLE_LOOP_ITERATIONS {
            self.nop();
        }
    }
}
