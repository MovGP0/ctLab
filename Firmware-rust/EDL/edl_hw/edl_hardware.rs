use super::*;

/// Hardware hooks needed by the original EDL firmware routines.
pub trait EdlHardware {
    fn set_control_bit(&mut self, bit: ControlBit, high: bool);
    fn read_control_bit(&self, bit: ControlBit) -> bool;

    fn set_trigger_out(&mut self, high: bool);
    fn read_trigger_in(&self) -> bool;
    fn set_ad16_mpx(&mut self, high: bool);

    fn set_admux(&mut self, value: u8);
    fn write_adcsra(&mut self, value: u8);
    fn read_adcsra(&self) -> u8;
    fn read_adcl(&self) -> u8;
    fn read_adch(&self) -> u8;

    fn begin_interrupt_exclusion(&mut self) -> u8;
    fn end_interrupt_exclusion(&mut self, saved_status: u8);
    fn nop(&mut self);

    /// Performs the required ADC mux-settling delay. Implementations may
    /// override this provided behavior when the target has a cycle-exact form.
    fn settle_adc10_mux(&mut self) {
        for _ in 0..ADC10_SETTLE_CYCLES {
            self.nop();
        }
    }
}
