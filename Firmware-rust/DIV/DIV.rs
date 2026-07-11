//! Best-effort Rust port of `DIV.pas`.
//!
//! This preserves the original digital voltmeter structure: range tables,
//! calibration storage, ADC conversion helpers, display/serial formatting, and
//! a polling-style service loop.

#![allow(dead_code)]

/// Uses 32-bit floating point to match the precision and storage cost of the Pascal firmware.
pub type Float = f32;

/// Declares the 16 MHz AVR clock used to derive UART, TWI, ADC, and systick timing.
pub const PROC_CLOCK: u32 = 16_000_000;

/// Provides the full identification string returned by the `IDN` command.
pub const VERS1_STR: &str = "3.10 [DIV by CM/c't 03/2007]";

/// Provides the compact firmware name shown on the front-panel startup screen.
pub const VERS3_STR: &str = "DIV 3.10";

/// Sets `PORT_A_INIT` pull-ups and idle output levels before peripherals are accessed.
pub const PORT_A_INIT: u8 = 0b0000_0011;

/// Sets `PORT_C_INIT` pull-ups and idle output levels before peripherals are accessed.
pub const PORT_C_INIT: u8 = 0b0000_0011;

/// Defines LTC2400 bipolar zero at code 0x800000, which is subtracted before DC scaling.
pub const ADC24_MID_SCALE: i32 = 0x800000;

/// Marks a DIV EEPROM image as initialized with the 0xAA55 sentinel.
pub const EE_INITIALISED_MAGIC: u16 = 0xAA55;

/// Marks DIV zero-offset calibration as captured with the 0xAA55 sentinel.
pub const OFFSET_INITIALISED_MAGIC: u16 = 0xAA55;

/// Reserves err sub ch as the wire-level subchannel used by existing ctLab clients.
pub const ERR_SUB_CH: u8 = 255;

/// Provides the fixed-width range labels shown on the LCD and returned by range queries.
///
/// Each entry uses the discriminant of the corresponding [`DivRange`], so its
/// spacing and electrical unit are part of the user-visible firmware protocol.
#[rustfmt::skip]
pub const RANGE_STR_ARR: [&str; 16] = [
    "DC 250mV",
    "DC  2.5V",
    "DC   25V",
    "DC  250V",
    "AC 250mV",
    "AC  2.5V",
    "AC   25V",
    "AC  250V",
    "DC 250uA",
    "DC  25mA",
    "DC  2.5A",
    "DC   10A",
    "AC 250uA",
    "AC  25mA",
    "AC  2.5A",
    "AC   10A",
];

/// Gives the number of digits before the decimal separator for every DIV range.
///
/// The formatter uses the range discriminant as the index so values retain a
/// stable width when the selected voltage or current unit changes.
#[rustfmt::skip]
pub const DIGITS_ARR: [u8; 16] = [
    3,
    1,
    2,
    3,
    3,
    1,
    2,
    3,
    3,
    2,
    1,
    1,
    3,
    2,
    1,
    1,
];

/// Gives the number of digits after the decimal separator for every DIV range.
///
/// These values preserve the Pascal firmware's range-dependent resolution in
/// serial replies and on the front-panel display.
#[rustfmt::skip]
pub const NACHKOMMA_ARR: [u8; 16] = [
    3,
    5,
    4,
    3,
    3,
    5,
    4,
    3,
    3,
    4,
    5,
    5,
    3,
    4,
    5,
    5,
];

/// Maps each DIV range index to its Port A relay and gain bit pattern.
#[rustfmt::skip]
pub const RANGE_ARR_PORT_A: [u8; 16] = [
    PORT_A_INIT,
    PORT_A_INIT,
    0b0010_0000 | PORT_A_INIT,
    0b0010_0000 | PORT_A_INIT,
    PORT_A_INIT,
    PORT_A_INIT,
    PORT_A_INIT,
    PORT_A_INIT,
    PORT_A_INIT,
    0b0100_0000 | PORT_A_INIT,
    0b1000_0000 | PORT_A_INIT,
    0b1000_0000 | PORT_A_INIT,
    PORT_A_INIT,
    0b0100_0000 | PORT_A_INIT,
    0b1000_0000 | PORT_A_INIT,
    0b1000_0000 | PORT_A_INIT,
];

/// Maps each DIV range index to its Port C AC/DC and attenuation relay pattern.
#[rustfmt::skip]
pub const RANGE_ARR_PORT_C: [u8; 16] = [
    PORT_C_INIT,
    PORT_C_INIT,
    0b0001_0000 | PORT_C_INIT,
    0b0010_0000 | PORT_C_INIT,
    0b0100_0100 | PORT_C_INIT,
    0b0100_0000 | PORT_C_INIT,
    0b0100_1100 | PORT_C_INIT,
    0b0100_1000 | PORT_C_INIT,
    0b1000_0000 | PORT_C_INIT,
    0b1000_0000 | PORT_C_INIT,
    0b1000_0000 | PORT_C_INIT,
    0b1000_0000 | PORT_C_INIT,
    0b1100_0100 | PORT_C_INIT,
    0b1100_0100 | PORT_C_INIT,
    0b1100_0100 | PORT_C_INIT,
    0b1100_0100 | PORT_C_INIT,
];

/// Converts signed LTC2400 counts to engineering units for each DIV range before calibration scale is applied.
#[rustfmt::skip]
pub const RANGE_ARRAY_24: [Float; 16] = [
    250.0 / 8_388_608.0,
    2.5 / 8_388_608.0,
    25.0 / 8_388_608.0,
    250.0 / 8_388_608.0,
    250.0 / 8_388_608.0,
    2.5 / 8_388_608.0,
    25.0 / 8_388_608.0,
    250.0 / 8_388_608.0,
    250.0 / 8_388_608.0,
    25.0 / 8_388_608.0,
    2.5 / 8_388_608.0,
    2.5 / 8_388_608.0,
    250.0 / 8_388_608.0,
    25.0 / 8_388_608.0,
    2.5 / 8_388_608.0,
    2.5 / 8_388_608.0,
];

/// Converts internal ADC counts to engineering units for each DIV auxiliary range.
#[rustfmt::skip]
pub const RANGE_ARRAY_10: [Float; 16] = [
    250.0 / 512.0,
    2.5 / 512.0,
    25.0 / 512.0,
    250.0 / 512.0,
    250.0 / 1024.0,
    25.0 / 1024.0,
    2.5 / 1024.0,
    2.5 / 1024.0,
    250.0 / 512.0,
    25.0 / 512.0,
    2.5 / 512.0,
    2.5 / 512.0,
    250.0 / 1024.0,
    25.0 / 1024.0,
    2.5 / 1024.0,
    2.5 / 1024.0,
];

#[path = "div/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;
#[path = "div/error_code.rs"]
mod error_code;
pub use error_code::ErrorCode;
#[path = "div/div_fault.rs"]
mod div_fault;
pub use div_fault::DivFault;
#[path = "div/div_range.rs"]
mod div_range;
pub use div_range::DivRange;
#[path = "div/range_relay_config.rs"]
mod range_relay_config;
pub use range_relay_config::RangeRelayConfig;
#[path = "div/eeprom_data.rs"]
mod eeprom_data;
pub use eeprom_data::EepromData;
#[path = "div/div_hardware.rs"]
mod div_hardware;
pub use div_hardware::DivHardware;
#[path = "div/device_state.rs"]
mod device_state;
pub use device_state::DeviceState;

/// Maps div range from u8 into the typed state used internally, rejecting or defaulting unsupported wire values as the implementation specifies.
fn div_range_from_u8(value: u8) -> DivRange {
    limit_raw_range(value).0
}

/// Clamps an untrusted range byte to the 16-entry table while reporting whether protocol status must flag a parameter error.
fn limit_raw_range(value: u8) -> (DivRange, bool) {
    let limited = value > 15;
    let value = if value > 127 {
        0
    } else if value > 15 {
        15
    } else {
        value
    };

    match value {
        0 => (DivRange::Dc250mV, limited),
        1 => (DivRange::Dc2V5, limited),
        2 => (DivRange::Dc25V, limited),
        3 => (DivRange::Dc250V, limited),
        4 => (DivRange::Ac250mV, limited),
        5 => (DivRange::Ac2V5, limited),
        6 => (DivRange::Ac25V, limited),
        7 => (DivRange::Ac250V, limited),
        8 => (DivRange::Dc250uA, limited),
        9 => (DivRange::Dc25mA, limited),
        10 => (DivRange::Dc2A5, limited),
        11 => (DivRange::Dc10A, limited),
        12 => (DivRange::Ac250uA, limited),
        13 => (DivRange::Ac25mA, limited),
        14 => (DivRange::Ac2A5, limited),
        _ => (DivRange::Ac10A, limited),
    }
}

/// Chooses the engineering exponent appended to serial values so milli- and micro-ranges keep the original wire units.
pub fn range_exponent_suffix(range: DivRange) -> Option<&'static str> {
    match range {
        DivRange::Dc250mV | DivRange::Ac250mV | DivRange::Dc25mA | DivRange::Ac25mA => Some("E-3"),
        DivRange::Dc250uA | DivRange::Ac250uA => Some("E-6"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "DIV_tests.rs"]
mod tests;
