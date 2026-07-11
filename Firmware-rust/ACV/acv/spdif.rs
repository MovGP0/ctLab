//! Defines the consumer/professional S/PDIF formats and sample rates supported by the ACV converter setup.

#[allow(unused_imports)]
use super::*;

/// Selects both the S/PDIF channel-status format and converter sample rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Spdif {
    /// Uses consumer-format S/PDIF at 48 kHz.
    C48,

    /// Uses consumer-format S/PDIF at 96 kHz.
    C96,

    /// Uses consumer-format S/PDIF at 192 kHz.
    C192,

    /// Uses professional-format S/PDIF at 48 kHz.
    P48,

    /// Uses professional-format S/PDIF at 96 kHz.
    P96,

    /// Uses professional-format S/PDIF at 192 kHz.
    P192,
}
