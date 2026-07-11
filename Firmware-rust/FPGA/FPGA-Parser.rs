//! Serial protocol parser for the FPGA module controller.
//!
//! The command table and offsets are transcribed from `FPGA.pas` 2.62. The
//! parser retains the c't-Lab channel prefix, direct numeric subchannels,
//! wildcard forwarding, result frames, verbose markers, and XOR checksum.

pub const COMMANDS: [&str; 66] = [
    "STR", "IDN", "VAL", "REG", "ACC", "MOV", "DEC", "INC", "CPZ", "XCH", "GET", "PUT",
    "MUL", "DIV", "ADD", "SUB", "SQR", "SQU", "NEG", "LBL", "GTO", "BRA", "BRG", "BGE",
    "BEQ", "BLE", "BRL", "INP", "OUT", "TTF", "TTY", "TSF", "XMR", "TSR", "TSS", "COM",
    "AIR", "AIS", "AIW", "BLD", "BSV", "AIM", "AIE", "CLK", "OPT", "MCH", "SCH", "WTH",
    "WTM", "WTS", "DLY", "FWR", "FWV", "CFG", "LST", "DIR", "FNM", "FNA", "FDL", "FQU",
    "HEX", "WEN", "ERC", "SBD", "REM", "NOP",
];

pub const COMMAND_OFFSETS: [u16; 66] = [
    255, 254, 0, 300, 300, 310, 320, 330, 340, 350, 400, 500, 600, 610, 620, 630, 640,
    650, 660, 1000, 1100, 1100, 1200, 1300, 1400, 1500, 1600, 2000, 2000, 800, 880,
    881, 890, 900, 980, 990, 280, 281, 282, 283, 284, 285, 286, 90, 150, 270, 271, 290,
    291, 292, 299, 220, 230, 240, 241, 241, 242, 243, 244, 249, 88, 250, 251, 252, 253,
    253,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError
{
    Empty,
    Syntax,
    InvalidChannel,
    InvalidSubchannel,
    InvalidChecksum,
    MissingValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Parameter
{
    None,
    Number(f64),
    Text(String),
}

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

pub fn command_index(command: &str) -> Option<usize>
{
    let command = command.trim().to_ascii_uppercase();
    COMMANDS.iter().position(|candidate| *candidate == command)
}

pub fn command_subchannel(command: &str, argument: u16) -> Option<u16>
{
    let index = command_index(command)?;
    COMMAND_OFFSETS[index].checked_add(argument)
}

pub fn xor_checksum(bytes: &[u8]) -> u8
{
    bytes.iter().fold(0, |checksum, byte| checksum ^ byte)
}

pub fn parse_frame(input: &str) -> Result<ParsedFrame, ParseError>
{
    let input = input.trim_end_matches(['\r', '\n']);
    if input.is_empty()
    {
        return Err(ParseError::Empty);
    }

    let (without_checksum, checksum) = split_checksum(input)?;
    if let Some(expected) = checksum
    {
        if xor_checksum(without_checksum.as_bytes()) != expected
        {
            return Err(ParseError::InvalidChecksum);
        }
    }

    let verbose = without_checksum.contains(['!', '?']);
    let cleaned = without_checksum.replace(['!', '?'], "");
    let is_result = cleaned.starts_with('#');
    let is_omni = cleaned.starts_with('*');
    let is_request = !cleaned.contains('=');
    let mut body = cleaned.as_str();
    if is_result
    {
        body = &body[1..];
    }

    let (main_channel, command_and_value) = if let Some(colon) = body.find(':')
    {
        let channel_text = body[..colon].trim();
        let channel = if channel_text == "*"
        {
            None
        }
        else
        {
            Some(channel_text.parse().map_err(|_| ParseError::InvalidChannel)?)
        };
        (channel, &body[colon + 1..])
    }
    else
    {
        (None, body.trim_start_matches('*'))
    };

    let (command_text, value_text) = if let Some(equals) = command_and_value.find('=')
    {
        (&command_and_value[..equals], Some(command_and_value[equals + 1..].trim()))
    }
    else
    {
        (command_and_value, None)
    };

    let (token, argument) = split_command_argument(command_text);
    let subchannel = if let Ok(direct) = token.parse::<u16>()
    {
        direct
    }
    else
    {
        command_subchannel(token, argument).ok_or(ParseError::Syntax)?
    };

    let parameter = match value_text
    {
        None => Parameter::None,
        Some("") => return Err(ParseError::MissingValue),
        Some(value) => parse_parameter(value),
    };

    Ok(ParsedFrame
    {
        main_channel,
        subchannel,
        is_request,
        is_result,
        is_omni,
        verbose,
        parameter,
    })
}

fn split_checksum(input: &str) -> Result<(&str, Option<u8>), ParseError>
{
    let Some(position) = input.rfind('$') else
    {
        return Ok((input, None));
    };
    let checksum_text = input.get(position + 1..position + 3).ok_or(ParseError::InvalidChecksum)?;
    if checksum_text.len() != 2
    {
        return Err(ParseError::InvalidChecksum);
    }
    let checksum = u8::from_str_radix(checksum_text, 16).map_err(|_| ParseError::InvalidChecksum)?;
    Ok((&input[..position], Some(checksum)))
}

fn split_command_argument(command: &str) -> (&str, u16)
{
    let command = command.trim();
    if command.chars().all(|character| character.is_ascii_digit())
    {
        return (command, 0);
    }
    let split = command
        .char_indices()
        .find(|(_, character)| character.is_ascii_digit() || *character == ' ')
        .map(|(index, _)| index)
        .unwrap_or(command.len());
    let token = command[..split].trim();
    let argument = command[split..].trim().parse().unwrap_or(0);
    (token, argument)
}

fn parse_parameter(value: &str) -> Parameter
{
    let unquoted = value.strip_prefix('"').and_then(|value| value.strip_suffix('"'));
    if let Some(text) = unquoted
    {
        return Parameter::Text(text.to_string());
    }
    value
        .parse::<f64>()
        .map(Parameter::Number)
        .unwrap_or_else(|_| Parameter::Text(value.to_string()))
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn all_pascal_commands_keep_their_offsets()
    {
        assert_eq!(COMMANDS.len(), 66);
        assert_eq!(COMMAND_OFFSETS.len(), 66);
        assert_eq!(command_subchannel("REG", 9), Some(309));
        assert_eq!(command_subchannel("BRL", 0), Some(1600));
        assert_eq!(command_subchannel("OUT", 63), Some(2063));
        assert_eq!(command_subchannel("NOP", 0), Some(253));
    }

    #[test]
    fn parses_channel_command_argument_and_numeric_value()
    {
        let parsed = parse_frame("9:REG 3=-12.5!").unwrap();

        assert_eq!(parsed.main_channel, Some(9));
        assert_eq!(parsed.subchannel, 303);
        assert_eq!(parsed.parameter, Parameter::Number(-12.5));
        assert!(parsed.verbose);
        assert!(!parsed.is_request);
    }

    #[test]
    fn parses_quoted_file_names_and_direct_subchannels()
    {
        let parsed = parse_frame("9:243=\"DATAFILE.XLS\"").unwrap();

        assert_eq!(parsed.subchannel, 243);
        assert_eq!(parsed.parameter, Parameter::Text("DATAFILE.XLS".to_string()));
    }

    #[test]
    fn validates_pascal_xor_checksum()
    {
        let command = "9:VAL?";
        let frame = format!("{command}${:02X}", xor_checksum(command.as_bytes()));
        assert!(parse_frame(&frame).is_ok());
        assert_eq!(parse_frame("9:VAL?$00"), Err(ParseError::InvalidChecksum));
    }

    #[test]
    fn identifies_results_and_omni_frames()
    {
        assert!(parse_frame("#9:10=123").unwrap().is_result);
        assert!(parse_frame("*:NOP").unwrap().is_omni);
    }
}
