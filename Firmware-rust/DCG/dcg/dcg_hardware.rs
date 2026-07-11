//! Defines the DCG hardware seam used by calibrated foreground and protection logic.

use super::*;

/// Hardware boundary for the DC generator state machine; target code performs real I/O while tests can verify ordering and safety blanking.
pub trait DcgHardware {
    /// Selects the one-based AVR ADC channel, waits for mux settling, starts conversion, polls completion, then combines ADCL before ADCH as required by the AVR latch rule.
    fn read_adc10(&mut self, channel_1_based: u8) -> i16;

    /// Returns the LTC1864 voltage sample for the requested ripple phase so foreground scaling uses the matching on/off accumulator.
    fn read_adc16_voltage(&mut self) -> u16;

    /// Returns the LTC1864 current sample for the requested ripple phase so power and protection use time-aligned measurements.
    fn read_adc16_current(&mut self) -> u16;

    /// Waits for one UART byte only up to the legacy receive deadline so an incomplete DCG frame cannot starve protection work.
    fn serial_read_timeout(&mut self, timeout_ms: u16) -> Option<char>;

    /// Writes the energized-phase voltage DAC code used by the ripple timer.
    fn set_voltage_dac_raw(&mut self, raw: u16);

    /// Writes the calibrated current-limit DAC code after range selection and safe blanking.
    fn set_current_dac_raw(&mut self, raw: u16);

    /// Writes the off-phase voltage DAC code so ripple modulation can switch without recalculating in the interrupt.
    fn set_voltage_dac_off_raw(&mut self, raw: u16);

    /// Provides board-timed settling while keeping the delay mechanism replaceable in host tests.
    fn delay_ms(&mut self, milliseconds: u16);

    /// Selects the current shunt before applying its range-specific ADC and DAC calibration.
    fn set_current_range(&mut self, range: CurrentRange);

    /// Selects the voltage divider/relay path before measurements are interpreted with that range's scale.
    fn set_voltage_range(&mut self, range: VoltageRange);

    /// Controls the input supply relay; protection code drops it immediately on overvoltage, overtemperature, or fuse loss.
    fn set_input_relay_high(&mut self, high: bool);

    /// Reads the analog regulator's limit indication so overload reporting reflects real hardware saturation rather than only calculated limits.
    fn current_limit_sense(&mut self) -> bool;

    /// Connects or disconnects the output stage only after fault and mode latches have been evaluated.
    fn set_output_enabled(&mut self, enabled: bool);

    /// Reads the optional LM75 result as `Option`, allowing protection code to distinguish missing hardware from a real zero-degree reading.
    fn read_temp_c(&mut self) -> Option<Float>;

    /// Emits one byte on the instrument UART; framing and reply ordering remain the caller's responsibility.
    fn serial_write(&mut self, text: &str);

    /// Replaces a fixed LCD row in one operation, avoiding partially refreshed values during foreground updates.
    fn lcd_write_line(&mut self, row: u8, text: &str);
}
