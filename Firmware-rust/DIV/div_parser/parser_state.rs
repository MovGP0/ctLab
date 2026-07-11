//! Defines the working state shared by DIV tokenization, command dispatch, calibration access, and response generation.

#[allow(unused_imports)]
use super::*;

/// Carries one DIV frame through address parsing, mnemonic lookup, parameter conversion, and command execution.
#[derive(Debug, Clone)]
pub struct ParserState {
    /// Selects the protocol parameter currently being read or written; `255` is the status channel.
    pub sub_ch: u8,

    /// Tracks the most recently addressed channel so short-form commands can omit the address.
    pub current_ch: u8,

    /// Stores the command enum produced by mnemonic lookup and consumed by get/set dispatch.
    pub cmd_which: CmdWhich,

    /// Stores a parsed floating-point setter operand in engineering units.
    pub param: f32,

    /// Stores raw ADC counts and integer protocol operands that must not pass through floating-point formatting.
    pub param_long_int: i32,

    /// Contains the current token while the parser distinguishes mnemonics, numbers, and checksum text.
    pub param_str: String,

    /// Contains the complete CR-terminated frame being parsed, without the terminating carriage return.
    pub ser_inp_str: String,

    /// Indexes the next byte in `ser_inp_str`; extraction advances it past delimiters and consumed tokens.
    pub ser_inp_ptr: usize,

    /// Stores the address read from board straps and used to accept or prefix serial frames.
    pub slave_ch: u8,

    /// Tracks range so conversion, relay, and formatting decisions agree.
    pub range: u8,

    /// Contains ad24temp in converter counts until the owning conversion or output routine consumes it.
    pub ad24temp: i32,

    /// Selects live display smoothing: direct, fast-integrated, or slow-integrated AD24 data.
    pub lcd_integrate: u8,

    /// Mirrors the EEPROM startup smoothing mode restored by the `ALL` command.
    pub init_lcd_integrate: u8,

    /// Sets the live encoder detent threshold, in raw encoder increments per accepted step.
    pub inc_rast: i32,

    /// Mirrors the EEPROM startup encoder threshold restored during initialization.
    pub init_inc_rast: i32,

    /// Counts protocol errors reported through the `ERC` command until explicitly cleared.
    pub errcount: i32,

    /// Mirrors the `WEN` latch that permits calibration and EEPROM-changing setters.
    pub ee_unlocked: bool,

    /// Records whether `?` or `!` requested a verbose status response for the active frame.
    pub verbose: bool,

    /// Latches overload flag from the same converter status bits as the associated sample.
    pub overload_flag: bool,

    /// Carries the precise validation failure from range/parameter checks into the next status prompt.
    pub check_limit_err: ParserError,

    /// Uses bits 0, 1, and 2 to select AD24, AD10 RMS, and AD10 peak trigger responses.
    pub trig_mask: u8,

    /// Stores the automatic-trigger interval in milliseconds; zero disables timed triggering.
    pub trig_timer_value: u16,

    /// Latches an external, automatic, or command trigger until the polling loop services it.
    pub trigger: bool,

    /// Stores one signed AD24 count offset for each of the 16 ranges, indexed by the range byte.
    pub offset_array24: [i32; 16],

    /// Stores one signed AD10 count offset for each of the 16 ranges, indexed by the range byte.
    pub offset_array10: [i32; 16],

    /// Stores one AD24 multiplicative calibration factor for each of the 16 ranges.
    pub scale_array24: [f32; 16],

    /// Stores one AD10 multiplicative calibration factor for each of the 16 ranges.
    pub scale_array10: [f32; 16],
}

impl Default for ParserState {
    /// Builds one parser frame state with Pascal sentinel channel 255 and neutral calibration tables.
    fn default() -> Self {
        Self {
            sub_ch: 0,
            current_ch: 255,
            cmd_which: CmdWhich::Val,
            param: 0.0,
            param_long_int: 0,
            param_str: String::new(),
            ser_inp_str: String::new(),
            ser_inp_ptr: 0,
            slave_ch: 0,
            range: 0,
            ad24temp: 0,
            lcd_integrate: 1,
            init_lcd_integrate: 1,
            inc_rast: 4,
            init_inc_rast: 4,
            errcount: 0,
            ee_unlocked: false,
            verbose: false,
            overload_flag: false,
            check_limit_err: ParserError::NoErr,
            trig_mask: 0,
            trig_timer_value: 0,
            trigger: false,
            offset_array24: [0; 16],
            offset_array10: [0; 16],
            scale_array24: [1.0; 16],
            scale_array10: [1.0; 16],
        }
    }
}
