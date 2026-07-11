#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Default)]
pub struct DivHardwareState {
    pub ad24_temp: u32,
    pub ad24_temp_fast_integrated: u32,
    pub ad24_temp_slow_integrated: u32,
    pub ad24_integrate0: u32,
    pub ad24_integrate1: u32,
    pub ad24_integrate2: u32,
    pub ad24_integrate3: u32,
    pub negative_flag: bool,
    pub over_voltage_flag: bool,
    pub abort_flag: bool,
    pub trigger: bool,
    pub ad24_ready: bool,
    pub ad10_ready: bool,
}
