//! Defines ACV persisted calibration and startup values mirrored from the Pascal EEPROM layout.

#[allow(unused_imports)]
use super::*;

/// Stores the persisted EEPROM image layout mirrored from the Pascal firmware.
#[derive(Debug, Clone)]
pub(super) struct EepromImage {
    /// Stores the 0xAA55 validity sentinel checked before accepting the remaining EEPROM image.
    pub(super) ee_initialized: u16,

    /// Persists the rotary-encoder count threshold that produces one panel edit step.
    pub(super) init_inc_rast: i32,

    /// Persists the startup preamplifier-gain table index selected by option subchannel 150.
    pub(super) init_gain: u8,

    /// Persists the startup S/PDIF clock-source and sample-rate selection.
    pub(super) init_rate: Spdif,

    /// Persists the auxiliary command byte selected from the front-panel `AuxFunct` screen.
    pub(super) init_aux_cmd: u8,

    /// Stores the AVR UBRR divisor persisted by `SBD`; startup validates it before enabling double-speed UART mode.
    pub(super) ee_ser_baud_reg: u8,

    /// Stores the calibrated counts-to-engineering-unit factor for adc scales l.
    pub(super) adc_scales_l: [u16; 8],

    /// Stores the calibrated counts-to-engineering-unit factor for adc scales r.
    pub(super) adc_scales_r: [u16; 8],

    /// Persists whether startup exchanges the left and right display/measurement channels.
    pub(super) init_lr_swap: bool,
}

impl Default for EepromImage {
    /// Supplies the Pascal factory calibration and startup selections used for an empty ACV EEPROM.
    #[rustfmt::skip]
    fn default() -> Self {
        Self {
            ee_initialized: EE_INITIALIZED_MAGIC,
            init_inc_rast: 4,
            init_gain: 2,
            init_rate: Spdif::C48,
            init_aux_cmd: 7,
            ee_ser_baud_reg: 51,
            adc_scales_l: [
                2100,
                664,
                2100,
                664,
                2100,
                664,
                2100,
                664,
            ],
            adc_scales_r: [
                2100,
                664,
                2100,
                664,
                2100,
                664,
                2100,
                664,
            ],
            init_lr_swap: false,
        }
    }
}
