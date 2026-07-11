//! Measurement ranges supported by the digital voltmeter.

#[allow(unused_imports)]
use super::*;

/// Selects the measured quantity, current type, and full-scale value.
///
/// The discriminants intentionally match the positional calibration, display,
/// decimal-place, and relay tables used when [`DeviceState::switch_range`]
/// configures the analog front end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DivRange {
    /// Measures direct voltage with a full-scale range of 250 millivolts.
    Dc250mV = 0,

    /// Measures direct voltage with a full-scale range of 2.5 volts.
    Dc2V5 = 1,

    /// Measures direct voltage with a full-scale range of 25 volts.
    Dc25V = 2,

    /// Measures direct voltage with a full-scale range of 250 volts.
    Dc250V = 3,

    /// Measures alternating voltage with a full-scale range of 250 millivolts.
    Ac250mV = 4,

    /// Measures alternating voltage with a full-scale range of 2.5 volts.
    Ac2V5 = 5,

    /// Measures alternating voltage with a full-scale range of 25 volts.
    Ac25V = 6,

    /// Measures alternating voltage with a full-scale range of 250 volts.
    Ac250V = 7,

    /// Measures direct current with a full-scale range of 250 microamperes.
    Dc250uA = 8,

    /// Measures direct current with a full-scale range of 25 milliamperes.
    Dc25mA = 9,

    /// Measures direct current with a full-scale range of 2.5 amperes.
    Dc2A5 = 10,

    /// Measures direct current with a full-scale range of 10 amperes.
    Dc10A = 11,

    /// Measures alternating current with a full-scale range of 250 microamperes.
    Ac250uA = 12,

    /// Measures alternating current with a full-scale range of 25 milliamperes.
    Ac25mA = 13,

    /// Measures alternating current with a full-scale range of 2.5 amperes.
    Ac2A5 = 14,

    /// Measures alternating current with a full-scale range of 10 amperes.
    Ac10A = 15,
}
