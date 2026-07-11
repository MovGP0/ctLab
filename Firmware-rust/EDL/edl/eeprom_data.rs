use super::*;

#[derive(Debug, Clone)]
/// Persistent calibration and startup layout consumed by the EDL foreground state machine.
pub struct EepromData {
    /// Low/high voltage ADC zero corrections.
    pub adc_u_offsets: [i16; 2],

    /// Low/high voltage ADC gain corrections.
    pub adc_u_scales: [Float; 2],

    /// Per-shunt current ADC zero corrections.
    pub adc_i_offsets: [i16; 4],

    /// Per-shunt current ADC gain corrections.
    pub adc_i_scales: [Float; 4],

    /// Per-shunt current DAC zero corrections.
    pub dac_i_offsets: [i16; 4],

    /// Per-shunt current DAC gain corrections used for both current and legacy resistance scaling.
    pub dac_i_scales: [Float; 4],

    /// Stored resistance DAC factors retained for EEPROM layout compatibility despite Pascal not using them.
    pub dac_r_scales: [Float; 4],

    /// Indexed hardware and boot options in the original EEPROM order.
    pub option_array: [Float; OptionSlot::COUNT],

    /// UART divisor restored before serial command processing.
    pub ee_ser_baud_reg: u8,

    /// Encoder raster/detent default restored at startup.
    pub inc_rast_def: i16,

    /// Trigger and LM75 enable bits restored into live control state.
    pub trig_mask: u8,

    /// Magic value distinguishing initialized EEPROM from erased storage.
    pub ee_initialised: u16,

    /// Requests one-time startup initialization behavior.
    pub first_run: bool,
}

impl Default for EepromData {
    /// Reproduces the Pascal factory calibration placeholders and boot options.
    #[rustfmt::skip]
    fn default() -> Self {
        Self {
            adc_u_offsets: [
                -260,
                -260,
            ],
            adc_u_scales: [
                1.01,
                1.01,
            ],
            adc_i_offsets: [
                -260,
                -260,
                -260,
                -260,
            ],
            adc_i_scales: [
                1.01,
                1.01,
                1.01,
                1.01,
            ],
            dac_i_offsets: [
                0,
                0,
                0,
                0,
            ],
            dac_i_scales: [
                1.0,
                1.0,
                1.0,
                1.0,
            ],
            dac_r_scales: [
                1.0,
                1.0,
                1.0,
                1.0,
            ],
            option_array: [
                0.0,
                0.02,
                2.5,
                10.0,
                10.0,
                2.5,
                50.0,
                100.0,
                10.0,
                1.0,
                0.1,
                0.002,
                0.020,
                0.200,
                2.0,
                25.0,
                6.1,
                4.0,
                0.0,
                10.0,
                0.0,
                50.0,
            ],
            ee_ser_baud_reg: 51,
            inc_rast_def: 4,
            trig_mask: 0,
            ee_initialised: 0xAA55,
            first_run: false,
        }
    }
}

impl EepromData {
    /// Returns the startup low-voltage cutoff.
    pub fn init_volt(&self) -> Float {
        self.option_array[OptionSlot::InitialVoltage.index()]
    }

    /// Returns the startup constant-current setpoint.
    pub fn init_amp(&self) -> Float {
        self.option_array[OptionSlot::InitialCurrent.index()]
    }

    /// Returns the divider ratio for low-voltage modes.
    pub fn init_low_divider_u(&self) -> Float {
        self.option_array[OptionSlot::LowVoltageDivider.index()]
    }

    /// Returns the divider ratio for high-voltage modes.
    pub fn init_hi_divider_u(&self) -> Float {
        self.option_array[OptionSlot::HighVoltageDivider.index()]
    }

    /// Returns analog current-stage gain used in ADC and DAC conversion factors.
    pub fn init_gain_i(&self) -> Float {
        self.option_array[OptionSlot::CurrentMeasurementGain.index()]
    }

    /// Returns the converter reference voltage shared by scale calculations.
    pub fn uref(&self) -> Float {
        self.option_array[OptionSlot::ReferenceVoltage.index()]
    }

    /// Returns the maximum permitted load power.
    pub fn pmax(&self) -> Float {
        self.option_array[OptionSlot::MaximumPower.index()]
    }

    /// Returns the calibrated resistance of one shunt.
    ///
    /// # Panics
    ///
    /// Panics when `index` is outside the four physical shunt positions.
    pub fn rsense(&self, index: usize) -> Float {
        self.option_array[OptionSlot::SenseResistanceA.index() + index]
    }

    /// Returns the maximum safe current for one shunt.
    ///
    /// # Panics
    ///
    /// Panics when `index` is outside the four physical shunt positions.
    pub fn imax(&self, index: usize) -> Float {
        self.option_array[OptionSlot::MaximumCurrentA.index() + index]
    }

    /// Returns the high-range over-voltage clamp.
    pub fn voltage_limit_hi(&self) -> Float {
        self.option_array[OptionSlot::HighVoltageLimit.index()]
    }

    /// Returns the low-range over-voltage clamp.
    pub fn voltage_limit_lo(&self) -> Float {
        self.option_array[OptionSlot::LowVoltageLimit.index()]
    }

    /// Converts packed hardware option storage to the live option byte.
    pub fn init_options(&self) -> u8 {
        self.option_array[OptionSlot::InstalledHardware.index()] as u8
    }

    /// Returns the startup ripple-current percentage.
    pub fn init_i_percent(&self) -> i32 {
        self.option_array[OptionSlot::InitialCurrentPercent.index()] as i32
    }

    /// Returns the startup active ripple duration.
    pub fn init_ton(&self) -> i32 {
        self.option_array[OptionSlot::InitialRippleOnTime.index()] as i32
    }

    /// Returns the startup inactive ripple duration.
    pub fn init_toff(&self) -> i32 {
        self.option_array[OptionSlot::InitialRippleOffTime.index()] as i32
    }

    /// Returns the LM75 fan/over-temperature threshold programmed at initialization.
    pub fn init_fan_on_temp(&self) -> Float {
        self.option_array[OptionSlot::FanOnTemperature.index()]
    }
}
