use super::*;

#[derive(Debug, Clone)]
/// Timing-critical values shared by SysTick acquisition and foreground control.
pub struct EdlState {
    /// Raw DAC word selected for the next serial transfer.
    pub dac_temp: u16,

    /// Most recent 16-bit ADC word shifted in at the start of SysTick.
    pub ad16_temp: u16,

    /// Smoothed current sample captured while the ripple output is off.
    pub ad16_temp_ioff: u16,

    /// Smoothed voltage sample captured while the ripple output is off.
    pub ad16_temp_uoff: u16,

    /// Smoothed current sample captured while the ripple output is on.
    pub ad16_temp_ion: u16,

    /// Smoothed voltage sample captured while the ripple output is on.
    pub ad16_temp_uon: u16,

    /// Calibrated DAC code applied during the active pulse.
    pub dac_temp_on: u16,

    /// DAC code applied during the ripple gap, normally zero unless offset is required.
    pub dac_temp_off: u16,

    /// Wire protocol selected from the EEPROM hardware option bits.
    pub dac_type: DacType,

    /// Remaining SysTick intervals before the current PWM phase changes.
    pub pw_counter: i32,

    /// Duration of the unloaded ripple phase.
    pub pw_off_time: i32,

    /// Duration of the loaded ripple phase.
    pub pw_on_time: i32,

    /// Current ripple phase controlling trigger output and DAC selection.
    pub pw_on_off: bool,

    /// Enables external-trigger gating of the active phase.
    pub trig_in_enable: bool,

    /// Forces both phase DAC values to zero while protection is active.
    pub overload_flag: bool,

    /// ADC mux selection prepared for the next pipelined conversion.
    pub ad16_select: bool,

    /// Measurement identity that the next conversion will produce.
    pub next_meas: MeasurementPhase,

    /// Measurement identity belonging to the ADC word read this tick.
    pub last_meas: MeasurementPhase,

    /// Pipeline stage connecting mux selection to the following ADC result.
    pub this_meas: MeasurementPhase,
}

impl Default for EdlState {
    /// Starts with output inactive and the ADC pipeline pointed at off-phase voltage.
    fn default() -> Self {
        Self {
            dac_temp: 0,
            ad16_temp: 0,
            ad16_temp_ioff: 0,
            ad16_temp_uoff: 0,
            ad16_temp_ion: 0,
            ad16_temp_uon: 0,
            dac_temp_on: 0,
            dac_temp_off: 0,
            dac_type: DacType::Ltc8043,
            pw_counter: 0,
            pw_off_time: 0,
            pw_on_time: 0,
            pw_on_off: false,
            trig_in_enable: false,
            overload_flag: false,
            ad16_select: false,
            next_meas: MeasurementPhase::Uoff,
            last_meas: MeasurementPhase::Uoff,
            this_meas: MeasurementPhase::Uoff,
        }
    }
}
