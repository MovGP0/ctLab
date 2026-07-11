//! Best-effort Rust port of `DDS-HW.pas`.
//!
//! The original Pascal unit bit-bangs an AD9833 DDS, a LTC1257 offset DAC,
//! a 4094 shift register chain, and an auxiliary serial output. This port
//! keeps the hardware-facing constants and routines readable while replacing
//! direct AVR register access with a small I/O trait.

use core::marker::PhantomData;

use crate::avrd_support::{Atmega32, AvrdPortIo, Mcu, RegisterPort};
pub use crate::{Ad9833Control, Waveform};

/// AVR data-space address of SREG, used to save and restore the caller's interrupt-enable state around converter frames.
#[cfg(target_arch = "avr")]
const AVR_SREG_ADDRESS: *mut u8 = 0x5f as *mut u8;

/// Masks AVR SREG bit 7 so critical-section code can restore the caller's interrupt-enable state exactly.
#[cfg(target_arch = "avr")]
const AVR_SREG_INTERRUPT_ENABLE_MASK: u8 = 0x80;

/// Converts each Pascal auxiliary-delay unit into 160 CPU cycles for the YAMPP bit-banged baud rate.
pub const SER_AUX_DELAY_CYCLES_PER_UNIT: u16 = 160;

#[path = "dds_hw/port_kind.rs"]
mod port_kind;
pub use port_kind::PortKind;

#[path = "dds_hw/dds_hardware_io.rs"]
mod dds_hardware_io;
pub use dds_hardware_io::DdsHardwareIo;

#[path = "dds_hw/dds_avrd.rs"]
mod dds_avrd;
pub use dds_avrd::DdsAvrd;

#[path = "dds_hw/dds_hardware_state.rs"]
mod dds_hardware_state;
pub use dds_hardware_state::DdsHardwareState;

/// Concrete DDS hardware adapter binding the generic AVR implementation to the ATmega32 register map.
pub type DdsAtmega32 = DdsAvrd<Atmega32>;

/// DDS output-port bit driving the shared serial clock.
pub const B_SCLK: u8 = 0;

/// DDS output-port bit carrying AD9833, DAC, and relay serial data.
pub const B_SDATAOUT: u8 = 1;

/// DDS output-port bit framing each AD9833 sixteen-bit register write.
pub const B_FSYNC: u8 = 2;

/// Control-port bit latching the cascaded relay shift registers.
pub const B_STROBE: u8 = 3;

/// Control-port bit latching a completed amplitude-DAC word.
pub const B_STRDAC: u8 = 4;

/// Control-port bit used as the bit-banged auxiliary YAMPP transmit line.
pub const B_SER_AUX: u8 = 5;

/// AD9833 frame prefix selecting frequency register zero for each fourteen-bit tuning-word half.
pub const DDS_FREQ_REGISTER_WRITE: u16 = 0b0100_0000_0000_0000;

/// Integer decimal-decade factors used to assemble the 28-bit AD9833 tuning word without floating-point work.
#[rustfmt::skip]
pub const FHZ_INT: [i32; 8] = [
    64_000_000,
    6_400_000,
    640_000,
    64_000,
    6_400,
    640,
    64,
    6,
];

/// SQG decimal-decade factors for its reference clock and floating-point frequency representation.
#[rustfmt::skip]
pub const FHZ_SQG: [f32; 9] = [
    134_217_728.0,
    13_421_772.8,
    1_342_177.3,
    134_217.73,
    13_421.772,
    1_342.177_2,
    134.217_73,
    13.421_773,
    1.342_177_3,
];

/// Square-wave relay position in the two-register board's second payload byte.
const TWO_SR_SQUARE_SW_BIT: u8 = 4;

/// Attenuator relay position in the two-register board's second payload byte.
const TWO_SR_ATTN_SW_BIT: u8 = 5;

/// External-output enable position in the two-register board's second payload byte.
const TWO_SR_EXT_ON_BIT: u8 = 6;

/// Offset-path relay position in the two-register board's second payload byte.
const TWO_SR_OFFS_SW_BIT: u8 = 7;

/// Square-wave relay position in the three-register board's control byte.
const THREE_SR_SQUARE_SW_BIT: u8 = 0;

/// Attenuator relay position in the three-register board's control byte.
const THREE_SR_ATTN_SW_BIT: u8 = 1;

/// External-output enable position in the three-register board's control byte.
const THREE_SR_EXT_ON_BIT: u8 = 2;

/// Offset-path relay position in the three-register board's control byte.
const THREE_SR_OFFS_SW_BIT: u8 = 3;

/// Logic-level output relay position in the three-register board's control byte.
const THREE_SR_LOGIC_SW_BIT: u8 = 4;

/// Front-panel switch LED position in the LED shift-register image.
const LED_SWITCH_BIT: u8 = 3;

#[cfg(test)]
#[path = "DDS-HW_tests.rs"]
mod tests;
