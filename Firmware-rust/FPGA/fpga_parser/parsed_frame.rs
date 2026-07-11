use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFrame
{
    pub main_channel: Option<u8>,
    pub subchannel: u16,
    pub is_request: bool,
    pub is_result: bool,
    pub is_omni: bool,
    pub verbose: bool,
    pub parameter: Parameter,
}
