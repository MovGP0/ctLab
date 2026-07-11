#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone)]
pub struct EepromData {
    pub ad24_offsets: [i32; 16],
    pub ad24_scales: [Float; 16],
    pub ad10_offsets: [i16; 16],
    pub ad10_scales: [Float; 16],
    pub init_inc_rast: i16,
    pub init_lcd_integrate: u8,
    pub init_range: DivRange,
    // Original TRM mask: bit0=AD24, bit1=AD10 RMS, bit2=AD10 peak.
    pub trigger_mode: u8,
    // Auto-trigger interval in milliseconds; 0 disables timed retriggering.
    pub trigger_timer_ms: u16,
    // INT2 trigger edge: false=negative edge, true=positive edge.
    pub trigger_edge_level: bool,
    pub ee_ser_baud_reg: u8,
    pub ee_initialised: u16,
    pub offset_initialised: u16,
}

impl Default for EepromData {
    fn default() -> Self {
        Self {
            ad24_offsets: [0; 16],
            ad24_scales: [1.0; 16],
            ad10_offsets: [0; 16],
            ad10_scales: [1.0; 16],
            init_inc_rast: 4,
            init_lcd_integrate: 1,
            init_range: DivRange::Dc25V,
            trigger_mode: 0,
            trigger_timer_ms: 0,
            trigger_edge_level: false,
            ee_ser_baud_reg: 51,
            ee_initialised: EE_INITIALISED_MAGIC,
            offset_initialised: 0,
        }
    }
}
