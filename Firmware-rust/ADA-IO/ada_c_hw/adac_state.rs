#[allow(unused_imports)]
use super::*;

#[derive(Clone, Debug, Default)]
pub struct AdacState {
    pub dac_temp: Word,
    pub ad_raw: Word,
    pub port_sr0: Byte,
    pub port_sr1: Byte,
    pub port_sr2: Byte,
    pub port_sr3: Byte,
    pub mux_ch: usize,
    pub adc16_present: bool,
    pub dac16_present: bool,
    pub dac714_present: bool,
    pub dac12_present: bool,
    pub integrate_ad16: bool,
    pub ad16_long: LongInt,
    pub adc_raw_array: [Integer; MUX_CHANNEL_COUNT],
    pub dac_raw_array: [Word; MUX_CHANNEL_COUNT],
}
