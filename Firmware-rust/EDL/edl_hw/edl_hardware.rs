use super::*;

/// Pin and register operations needed by cycle-ordered EDL converter routines.
///
/// Required hooks deliberately have no defaults: silently losing a latch edge,
/// trigger update, ADC byte, or interrupt restore would corrupt measurements or output.
pub trait EdlHardware {
    /// Drives a named shared serial/latch control line.
    fn set_control_bit(&mut self, bit: ControlBit, high: bool);

    /// Samples the serial ADC data line without changing its direction or pull-up.
    fn read_control_bit(&self, bit: ControlBit) -> bool;

    /// Mirrors ripple phase on the external trigger output.
    fn set_trigger_out(&mut self, high: bool);

    /// Samples the external gate used to suppress active load pulses.
    fn read_trigger_in(&self) -> bool;

    /// Preselects voltage or current for the pipelined 16-bit conversion.
    fn set_ad16_mpx(&mut self, high: bool);

    /// Selects the AVR ADC input after the Pascal one-based channel conversion.
    fn set_admux(&mut self, value: u8);

    /// Starts the AVR ADC with the required prescaler bits.
    fn write_adcsra(&mut self, value: u8);

    /// Polls the conversion-in-progress bit without caching stale state.
    fn read_adcsra(&self) -> u8;

    /// Reads the low byte first to latch a coherent AVR ADC result.
    fn read_adcl(&self) -> u8;

    /// Reads the high byte completing the latched ADC result.
    fn read_adch(&self) -> u8;

    /// Saves interrupt state and excludes preemption from the LTC1864 clock train.
    fn begin_interrupt_exclusion(&mut self) -> u8;

    /// Restores the exact saved interrupt state rather than unconditionally enabling interrupts.
    fn end_interrupt_exclusion(&mut self, saved_status: u8);

    /// Burns one predictable processor cycle where external devices require hold time.
    fn nop(&mut self);

    /// Performs the required ADC mux-settling delay. Implementations may
    /// override this provided behavior when the target has a cycle-exact form.
    fn settle_adc10_mux(&mut self) {
        for _ in 0..ADC10_SETTLE_CYCLES {
            self.nop();
        }
    }
}
