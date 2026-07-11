//! Defines ADA state retained across parser, polling-loop, or interrupt operations.

#[allow(unused_imports)]
use super::*;

/// Collects adac state that must survive across polling-loop or interrupt updates.
#[derive(Clone, Debug, Default)]
pub struct AdacState {
    /// Stores DAC temp until calibration, limit checking, and response formatting have consumed it.
    pub dac_temp: Word,

    /// Contains ad raw in converter counts until scaling or hardware output consumes it.
    pub ad_raw: Word,

    /// Holds the byte shifted last into the cascaded 4094 chain, for logical output port 0.
    pub port_sr0: Byte,

    /// Holds the 4094 output image for logical port 1.
    pub port_sr1: Byte,

    /// Holds the 4094 output image for logical port 2.
    pub port_sr2: Byte,

    /// Holds the byte shifted first into the cascaded 4094 chain, for logical output port 3.
    pub port_sr3: Byte,

    /// Indexes the next of eight ADA channels serviced by the systick round-robin scan.
    pub mux_ch: usize,

    /// Enables LTC1864 sampling after startup sense-line detection succeeds.
    pub adc16_present: bool,

    /// Selects the LTC1655 16-bit transfer during each mux-channel update.
    pub dac16_present: bool,

    /// Selects the DAC714 clock/latch sequence during each mux-channel update.
    pub dac714_present: bool,

    /// Selects the LTC1257 12-bit transfer during each mux-channel update.
    pub dac12_present: bool,

    /// Selects whether four external AD16 samples are accumulated before publication.
    pub integrate_ad16: bool,

    /// Accumulates external AD16 samples without overflowing the 16-bit published result.
    pub ad16_long: LongInt,

    /// Stores external ADC samples indexed by the corresponding protocol channel.
    pub adc_raw_array: [Integer; MUX_CHANNEL_COUNT],

    /// Stores eight unclamped DAC codes indexed by output channel 0..7.
    pub dac_raw_array: [Word; MUX_CHANNEL_COUNT],
}
