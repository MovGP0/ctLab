//! Serial protocol parser for the FPGA module controller.
//!
//! The command table and offsets are transcribed from `FPGA.pas` 2.62. The
//! parser retains the c't-Lab channel prefix, direct numeric subchannels,
//! wildcard forwarding, result frames, verbose markers, and XOR checksum.

/// Syntax failures detected before controller state can change.
#[path = "fpga_parser/parse_error.rs"]
mod parse_error;
pub use parse_error::ParseError;

/// Typed FPGA command mnemonics and their Pascal subchannel offsets.
#[path = "fpga_parser/cmd_which.rs"]
mod cmd_which;
pub use cmd_which::CmdWhich;

/// Typed right-hand-side values retained after parsing.
#[path = "fpga_parser/parameter.rs"]
mod parameter;
pub use parameter::Parameter;

/// Normalized routing and operation metadata consumed by dispatch.
#[path = "fpga_parser/parsed_frame.rs"]
mod parsed_frame;
pub use parsed_frame::ParsedFrame;

/// Computes the bytewise XOR checksum used after the protocol's `$HH` suffix.
pub fn xor_checksum(bytes: &[u8]) -> u8
{
    bytes.iter().fold(0, |checksum, byte| checksum ^ byte)
}

/// Normalizes one serial line into typed routing, operation, and parameter fields.
///
/// Checksum validation happens before marker removal so the XOR covers exactly
/// the bytes transmitted by the sender.
///
/// # Errors
///
/// Returns [`ParseError`] for an empty line, malformed channel or command syntax,
/// a missing setter value, or a supplied checksum that does not match the frame.
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
        CmdWhich::from_mnemonic(token)
            .sub_channel(argument)
            .ok_or(ParseError::Syntax)?
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

/// Separates and decodes an optional terminal `$HH` checksum.
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

/// Separates a mnemonic from its numeric suffix while leaving direct subchannels intact.
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

/// Prefers quoted text, then numeric conversion, and finally unquoted text.
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
#[path = "FPGA-Parser_tests.rs"]
mod tests;
