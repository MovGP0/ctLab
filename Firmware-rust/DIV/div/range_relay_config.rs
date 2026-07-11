//! Defines DIV range-dependent relay, gain, and display configuration.

#[allow(unused_imports)]
use super::*;

/// Collects the relay, ADC-gain, and display settings for one voltmeter range.
///
/// Keeping these values together ensures that a range change cannot combine the
/// analogue signal path for one range with the conversion or formatting rules
/// for another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeRelayConfig {
    /// Identifies the direct- or alternating-voltage/current range being configured.
    pub range: DivRange,

    /// Provides the complete Port A output image that selects the range relays.
    pub port_a: u8,

    /// Provides the complete Port C output image that selects the AC/DC signal path.
    pub port_c: u8,

    /// Enables the ADC's tenfold gain for low-level direct measurements.
    pub dc_gain_10: bool,

    /// Selects the number of digits before the decimal separator for the range's display unit.
    pub digits: u8,

    /// Selects the number of digits printed after the decimal separator.
    pub decimals: u8,
}

impl RangeRelayConfig {
    /// Builds one coherent configuration by indexing every range-dependent table
    /// with the discriminant of `range`.
    ///
    /// The Pascal firmware uses the same shared index so relay switching, ADC
    /// gain, and numeric formatting all change together.
    pub fn for_range(range: DivRange) -> Self {
        let index = range as usize;
        Self {
            range,
            port_a: RANGE_ARR_PORT_A[index],
            port_c: RANGE_ARR_PORT_C[index],
            dc_gain_10: matches!(
                range,
                DivRange::Dc250mV
                    | DivRange::Dc250uA
                    | DivRange::Dc25mA
                    | DivRange::Dc2A5
                    | DivRange::Dc10A
            ),
            digits: DIGITS_ARR[index],
            decimals: NACHKOMMA_ARR[index],
        }
    }
}
