//! Defines ADA persisted calibration and startup values mirrored from the Pascal EEPROM layout.

#[allow(unused_imports)]
use super::*;

/// Stores the persisted EEPROM data layout mirrored from the Pascal firmware.
#[derive(Debug, Clone)]
pub struct EepromData {
    /// Stores converter-count offsets in the protocol's ADC/DAC calibration slot order.
    pub offset_array: [i16; 28],

    /// Stores multiplicative calibration factors in the protocol's ADC/DAC calibration slot order.
    pub scale_array: [Float; 30],

    /// Stores eight startup direction bytes indexed by logical I/O port 0..7.
    pub dir_init_array: [u8; 8],

    /// Stores four trigger masks indexed by AD10, AD16, DAC, and digital-port scan group.
    pub trig_mask_array: [u8; 4],

    /// Uses `0` for the falling-edge trigger and `1` for the rising-edge trigger configured by `TRL`.
    pub trig_level: u8,

    /// Stores the ADA automatic-trigger interval in milliseconds; zero disables it.
    pub trig_timer_value: u16,

    /// Persists whether startup averages four LTC1864 samples for each published AD16 value.
    pub init_integrate_ad16: bool,

    /// Uses `0` for the external ADC reference and `1` for the AVR internal reference selected in ADMUX.
    pub ext_ref: u8,

    /// Persists the rotary-encoder count threshold used for one parameter edit step.
    pub inc_rast_def: i16,

    /// Stores the AVR UBRR divisor persisted by `SBD`; startup validates it before enabling double-speed UART mode.
    pub ee_ser_baud_reg: u8,

    /// Stores parameter text array in the wire or LCD representation expected by the original firmware.
    pub param_text_array: [String; 38],

    /// Stores the 0xAA55 validity sentinel checked before startup restores the remaining EEPROM fields.
    pub ee_initialised: u16,

    /// Persists the eight output bytes restored to the 4094 chain or I2C expanders at startup.
    pub port_init_array: [u8; 8],
}

impl Default for EepromData {
    /// Supplies the Pascal factory calibration, trigger, UART, and port values for an empty ADA EEPROM.
    #[rustfmt::skip]
    fn default() -> Self {
        Self {
            offset_array: [
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                -40,
                -40,
                -40,
                -40,
                -40,
                -40,
                -40,
                -40,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            scale_array: [
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                0.0,
                100.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                0.0,
                3185.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                200.0,
                3200.0,
            ],
            dir_init_array: [0; 8],
            trig_mask_array: [0; 4],
            trig_level: 0,
            trig_timer_value: 0,
            init_integrate_ad16: false,
            ext_ref: 1,
            inc_rast_def: 4,
            ee_ser_baud_reg: 51,
            param_text_array: array::from_fn(|_| String::new()),
            ee_initialised: 0xAA55,
            port_init_array: [0; 8],
        }
    }
}
