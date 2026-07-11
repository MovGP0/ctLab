//! Defines the consumer/professional S/PDIF formats and sample rates supported by the ACV converter setup.

#[allow(unused_imports)]
use super::*;

/// Selects both the S/PDIF channel-status format and converter sample rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Spdif {
    /// Uses consumer-format S/PDIF at 48 kHz.
    C48Khz,

    /// Uses consumer-format S/PDIF at 96 kHz.
    C96Khz,

    /// Uses consumer-format S/PDIF at 192 kHz.
    C192Khz,

    /// Uses professional-format S/PDIF at 48 kHz.
    P48Khz,

    /// Uses professional-format S/PDIF at 96 kHz.
    P96Khz,

    /// Uses professional-format S/PDIF at 192 kHz.
    P192Khz,
}
