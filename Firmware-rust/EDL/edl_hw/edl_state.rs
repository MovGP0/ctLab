use super::*;

#[derive(Debug, Clone)]
pub struct EdlState {
    pub dac_temp: u16,
    pub ad16_temp: u16,
    pub ad16_temp_ioff: u16,
    pub ad16_temp_uoff: u16,
    pub ad16_temp_ion: u16,
    pub ad16_temp_uon: u16,
    pub dac_temp_on: u16,
    pub dac_temp_off: u16,
    pub dac_type: DacType,
    pub pw_counter: i32,
    pub pw_off_time: i32,
    pub pw_on_time: i32,
    pub pw_on_off: bool,
    pub trig_in_enable: bool,
    pub overload_flag: bool,
    pub ad16_select: bool,
    pub next_meas: MeasurementPhase,
    pub last_meas: MeasurementPhase,
    pub this_meas: MeasurementPhase,
}

impl Default for EdlState {
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
