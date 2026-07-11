//! Defines ADA the board-I/O contract that separates protocol logic from register access.

#[allow(unused_imports)]
use super::*;

/// Supplies the GPIO, ADC registers, and interrupt exclusion needed by ADA bit-level converter transactions.
pub trait AdacHardware {
    /// Drives the AVR pin mapped to the named clock, data, or latch signal.
    fn set_signal(&mut self, signal: Signal, high: bool);

    /// Samples signal directly from its mapped input pin during the bit-level peripheral transaction; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_signal(&self, signal: Signal) -> bool;

    /// Writes the complete Port C image used for mux channel bits 2..4 and fixed control levels.
    fn set_port_c(&mut self, value: Byte);

    /// Writes ADMUX with the selected ADC channel and reference bits before an ADC10 conversion.
    fn set_admux(&mut self, value: Byte);

    /// Writes adcsra to the serial, display, or peripheral destination selected by the implementation; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn write_adcsra(&mut self, value: Byte);

    /// Reads the AVR adcsra register used to detect completion and assemble the 10-bit conversion; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_adcsra(&self) -> Byte;

    /// Reads the AVR adcl register used to detect completion and assemble the 10-bit conversion; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_adcl(&self) -> Byte;

    /// Reads the AVR adch register used to detect completion and assemble the 10-bit conversion; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn read_adch(&self) -> Byte;

    /// Marks the begin interrupt exclusion boundary so interrupt state is saved and restored around the complete transaction; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn begin_interrupt_exclusion(&mut self) -> Byte;

    /// Marks the end interrupt exclusion boundary so interrupt state is saved and restored around the complete transaction; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn end_interrupt_exclusion(&mut self, saved_status: Byte);

    /// Provides the nop timing gap required between peripheral signal edges; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn nop(&mut self) {
        self.wait_cycles(1);
    }

    /// Waits for cycles so callers cannot consume a stale hardware result; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn wait_cycles(&mut self, cycles: u16) {
        for _ in 0..cycles {
            core::hint::spin_loop();
        }
    }

    /// Waits for for adc10 complete so callers cannot consume a stale hardware result; the abstraction keeps board-specific access out of protocol logic and makes ordering testable.
    fn wait_for_adc10_complete(&mut self);
}
