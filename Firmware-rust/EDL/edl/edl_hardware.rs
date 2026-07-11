use super::*;

/// Hardware effects required by the high-level EDL control state machine.
///
/// Required methods have no no-op defaults because losing an ADC read, shunt
/// change, temperature write, or output-disable action would be unsafe.
pub trait EdlHardware {
    /// Reads the 16-bit voltage converter for the requested ripple phase.
    fn read_voltage_adc16(&mut self, on_phase: bool) -> u16;

    /// Reads the 16-bit current converter for the requested ripple phase.
    fn read_current_adc16(&mut self, on_phase: bool) -> u16;

    /// Samples the AVR ADC voltage monitor used for independent protection checks.
    fn read_voltage_adc10(&mut self) -> i16;

    /// Samples the AVR ADC current monitor used for independent protection checks.
    fn read_current_adc10(&mut self) -> i16;

    /// Selects the calibrated shunt before current conversion or DAC programming.
    fn set_shunt(&mut self, shunt_index: u8);

    /// Drives load enable so any latched protection can remove power immediately.
    fn set_output_enabled(&mut self, enabled: bool);

    /// Programs the active current/resistance DAC with its calibrated raw code.
    fn set_dac_raw(&mut self, raw: u16);

    /// Reads the active temperature source; `None` means disabled or unavailable hardware.
    fn read_temp_c(&mut self) -> Option<Float>;

    /// Writes LM75 configuration, threshold, hysteresis, and register-pointer data.
    fn lm75_write(&mut self, address: u8, register: u8, data: &[u8]);

    /// Emits already framed protocol text without changing checksum or line endings.
    fn serial_write(&mut self, text: &str);

    /// Replaces one fixed-width LCD row used by front-panel status pages.
    fn lcd_write_line(&mut self, row: u8, text: &str);

    /// Samples the external trigger that gates ripple and service behavior.
    fn read_trigger_in(&mut self) -> bool;
}
