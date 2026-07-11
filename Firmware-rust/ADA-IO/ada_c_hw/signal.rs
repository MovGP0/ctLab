//! Defines ADA logical peripheral signals and their role in bit-level transactions.

#[allow(unused_imports)]
use super::*;

/// Names ADA serial, strobe, and data pins independently of their AVR port mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Signal {
    /// Addresses the serial-data output pin used by DAC and shift-register writes.
    SDataOut,

    /// Addresses the shared peripheral serial-clock pin.
    SClk,

    /// Addresses the DAC latch strobe.
    StrDac,

    /// Addresses the external 16-bit ADC strobe.
    StrAd16,

    /// Addresses the output shift-register latch strobe.
    StrSr,

    /// Addresses the DAC multiplexer latch strobe.
    StrDaMux,

    /// Addresses the serial-data input pin used by external converters.
    SDataIn1,
}
