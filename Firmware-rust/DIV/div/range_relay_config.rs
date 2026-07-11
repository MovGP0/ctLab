#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeRelayConfig {
    pub range: DivRange,
    pub port_a: u8,
    pub port_c: u8,
    pub dc_gain_10: bool,
    pub digits: u8,
    pub decimals: u8,
}

impl RangeRelayConfig {
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
