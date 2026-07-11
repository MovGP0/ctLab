//! Defines DIV persisted calibration and startup values mirrored from the Pascal EEPROM layout.

#[allow(unused_imports)]
use super::*;

/// Stores the persisted EEPROM data layout mirrored from the Pascal firmware.
#[derive(Debug, Clone)]
pub struct EepromData {
    /// Stores signed converter-count corrections for ad24 offsets, indexed by the declared channel or range.
    pub ad24_offsets: [i32; 16],

    /// Stores the calibrated counts-to-engineering-unit factor for ad24 scales.
    pub ad24_scales: [Float; 16],

    /// Stores signed converter-count corrections for ad10 offsets, indexed by the declared channel or range.
    pub ad10_offsets: [i16; 16],

    /// Stores the calibrated counts-to-engineering-unit factor for ad10 scales.
    pub ad10_scales: [Float; 16],

    /// Persists the rotary-encoder count threshold used for one front-panel edit step.
    pub init_inc_rast: i16,

    /// Mirrors the EEPROM display-integration mode restored by initialization and `ALL`.
    pub init_lcd_integrate: u8,

    /// Tracks init range so conversion, relay, and formatting decisions agree.
    pub init_range: DivRange,
    // Original TRM mask: bit0=AD24, bit1=AD10 RMS, bit2=AD10 peak.

    /// Uses bits 0..2 to select DIV AD24, AD10 RMS, and AD10 peak trigger responses.
    pub trigger_mode: u8,
    // Auto-trigger interval in milliseconds; 0 disables timed retriggering.

    /// Stores the DIV automatic-trigger interval in milliseconds; zero disables it.
    pub trigger_timer_ms: u16,
    // INT2 trigger edge: false=negative edge, true=positive edge.

    /// Stores the DIV external-trigger polarity selected by `TRL`: false for falling, true for rising.
    pub trigger_edge_level: bool,

    /// Stores the AVR UBRR divisor persisted by `SBD`; startup validates it before enabling double-speed UART mode.
    pub ee_ser_baud_reg: u8,

    /// Stores the 0xAA55 validity sentinel checked before restoring the DIV EEPROM image.
    pub ee_initialised: u16,

    /// Stores signed converter-count corrections for offset initialised, indexed by the declared channel or range.
    pub offset_initialised: u16,
}

impl Default for EepromData {
    /// Supplies neutral calibration and the Pascal 25 V DC startup selection for an empty DIV EEPROM.
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
