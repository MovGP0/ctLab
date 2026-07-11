//! Defines the DCG hardware seam used by calibrated foreground and protection logic.

use super::*;

/// Hardware boundary for the DC generator state machine; target code performs real I/O while tests can verify ordering and safety blanking.
pub trait DcgHardware {
    /// Drives the shared serial-data pin used by the board's DAC and ADC shift protocols.
    fn set_sdata_out(&mut self, high: bool);

    /// Drives the shared serial clock; callers control each edge explicitly because attached converters sample on different phases.
    fn set_sclk(&mut self, high: bool);

    /// Drives the DAC latch strobe only after a complete serial word is stable.
    fn set_str_dac(&mut self, high: bool);

    /// Drives the ADC conversion/chip-select strobe around one uninterrupted LTC1864 transfer.
    fn set_str_ad16(&mut self, high: bool);

    /// Selects the current measurement path before the ADC settling interval begins.
    fn set_mpx_i(&mut self, high: bool);

    /// Selects the voltage measurement path before the ADC settling interval begins.
    fn set_mpx_u(&mut self, high: bool);

    /// Selects the LTC1864 input phase used by the next pipelined conversion.
    fn set_mpx_1864(&mut self, high: bool);

    /// Samples the converter's serial-data pin on the clock edge chosen by the bit-level receive routine.
    fn read_sdata_in1(&self) -> bool;

    /// Burns the requested CPU cycles between converter edges whose minimum spacing is shorter than a scheduler tick.
    fn spin_delay_cycles(&mut self, cycles: u16);

    /// Writes the AVR analog-multiplexer selection used by the subsequent ADC10 conversion.
    fn set_admux(&mut self, value: u8);

    /// Writes the AVR ADC control/status byte used to start conversion with the configured prescaler.
    fn write_adcsra(&mut self, value: u8);

    /// Reads ADC status so polling waits for ADSC to clear instead of consuming an unfinished conversion.
    fn read_adcsra(&self) -> u8;

    /// Reads the ADC low byte first, which latches the paired high byte on AVR hardware.
    fn read_adcl(&self) -> u8;

    /// Reads the high byte after ADCL to complete one coherent AVR ADC sample.
    fn read_adch(&self) -> u8;

    /// Saves SREG and disables interrupts so a multi-edge converter transaction cannot be split by timer service.
    fn begin_interrupt_exclusion(&mut self) -> u8;

    /// Restores the saved SREG value so the caller's previous interrupt-enable state survives the converter transaction.
    fn end_interrupt_exclusion(&mut self, saved_status: u8);

    /// Emits one AVR no-operation cycle used to satisfy converter setup and hold times without touching registers.
    fn nop(&mut self) {
        self.spin_delay_cycles(1);
    }

    /// Holds the DCG DAC interface idle for the converter's required post-latch settling loop.
    fn wait_post_dac_settle(&mut self) {
        for _ in 0..DAC_POST_WRITE_SETTLE_LOOP_ITERATIONS {
            self.nop();
        }
    }
}
