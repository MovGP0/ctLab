//! Best-effort Rust port of `EDL-HW.pas`.
//!
//! The original Pascal file is a low-level AVR hardware unit made up mostly of
//! inline assembly. This port keeps the same DAC/ADC state machine and the same
//! bit-banged wire protocol sequencing, but expresses the pin/register access
//! through a small trait so the routines stay readable and portable.

use core::marker::PhantomData;

use crate::avrd_support::{Atmega32, Atmega644, AvrdPortIo, Mcu, RegisterPort};

#[cfg(target_arch = "avr")]
const AVR_SREG_ADDRESS: *mut u8 = 0x5f as *mut u8;
#[cfg(target_arch = "avr")]
const AVR_SREG_INTERRUPT_ENABLE_MASK: u8 = 0x80;

/// Masks the Pascal one-based ADC request to the ATmega's three MUX bits.
pub const ADC10_CHANNEL_MASK: u8 = 0x07;

/// Starts a conversion with ADC enabled and the 128:1 prescaler required at 16 MHz.
pub const ADCSRA_START_DIV128: u8 = 0xC7;

/// `ADSC` bit polled until hardware completes a conversion.
pub const ADCSRA_BUSY_BIT: u8 = 1 << 6;

/// Host/fallback delay iterations matching the AVR mux-settling loop.
pub const ADC10_SETTLE_CYCLES: usize = 15;
#[cfg(target_arch = "avr")]
const EDL_CONTROL_BIT_PORT_IO_ADDRESS: u8 = 0x18;
#[cfg(target_arch = "avr")]
const EDL_CONTROL_BIT_PIN_IO_ADDRESS: u8 = 0x16;

/// Logical identities for the shared serial and latch pins.
#[path = "edl_hw/control_bit.rs"]
mod control_bit;
pub use control_bit::ControlBit;

/// Supported external DAC wire protocols.
#[path = "edl_hw/dac_type.rs"]
mod dac_type;
pub use dac_type::DacType;

/// Tags connecting pipelined ADC results to PWM phase and signal type.
#[path = "edl_hw/measurement_phase.rs"]
mod measurement_phase;
pub use measurement_phase::MeasurementPhase;

/// Electrical contract used by cycle-ordered converter routines.
#[path = "edl_hw/edl_hardware.rs"]
mod edl_hardware;
pub use edl_hardware::EdlHardware;

/// Real ATmega register backend with cycle-exact AVR instruction paths.
#[path = "edl_hw/edl_avrd.rs"]
mod edl_avrd;
pub use edl_avrd::EdlAvrd;

/// EDL backend using the ATmega32 register map.
pub type EdlAtmega32 = EdlAvrd<Atmega32>;

/// EDL backend using the ATmega644 register map.
pub type EdlAtmega644 = EdlAvrd<Atmega644>;

/// Raw converter and PWM pipeline state shared with SysTick.
#[path = "edl_hw/edl_state.rs"]
mod edl_state;
pub use edl_state::EdlState;

/// Bit-banged ADC/DAC transactions and timer-driven acquisition pipeline.
#[path = "edl_hw/edl_hw.rs"]
mod implementation;
pub use implementation::EdlHw;

#[cfg(test)]
#[path = "EDL-HW_tests.rs"]
mod tests;
