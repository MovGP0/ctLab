//! Defines ADA state retained across parser, polling-loop, or interrupt operations.

#[allow(unused_imports)]
use super::*;

/// Collects parse context that must survive across polling-loop or interrupt updates.
#[derive(Debug, Clone)]
pub struct ParseContext {
    /// Contains the current CR-terminated command frame without its trailing carriage return.
    pub ser_inp_str: String,

    /// Indexes the next unconsumed byte in the current command frame during token extraction.
    pub ser_inp_ptr: usize,

    /// Stores parameter string in the wire or LCD representation expected by the original firmware.
    pub param_str: String,

    /// Contains the parsed floating-point operand until range checking and command execution complete.
    pub param: f32,

    /// Contains the parsed integer operand until range checking and command execution complete.
    pub param_int: i32,

    /// Contains the parsed byte operand until range checking and command execution complete.
    pub param_byte: u8,

    /// Holds the protocol subchannel selected by the current frame; 255 is the status channel.
    pub sub_ch: u8,

    /// Tracks the most recently addressed channel so short-form commands can omit the address.
    pub current_ch: u8,

    /// Stores the address read from board straps and used to accept or prefix serial frames.
    pub slave_ch: u8,

    /// Stores the enum produced by mnemonic lookup and consumed by command dispatch.
    pub cmd_which: CmdWhich,

    /// Records whether `?` or `!` requested a verbose status response for the active frame.
    pub verbose: bool,

    /// Requests a display refresh after a setter or front-panel edit changes visible state.
    pub changed_flag: bool,

    /// Mirrors status bit 4 while `WEN` permits EEPROM-changing parser commands.
    pub ee_unlocked: bool,

    /// Selects the front-panel value or visualization currently being edited.
    pub modify: u8,

    /// Sets the number of raw encoder increments required for one accepted detent.
    pub inc_rast: i32,

    /// Mirrors the EEPROM startup encoder threshold changed by option subchannel 99.
    pub inc_rast_def: i32,

    /// Selects whether four external AD16 samples are accumulated before publication.
    pub integrate_ad16: bool,

    /// Holds the EEPROM-backed startup choice to average four LTC1864 samples per AD16 result.
    pub init_integrate_ad16: bool,

    /// Uses `0` for the external ADC reference and `1` for the AVR internal reference selected in ADMUX.
    pub ext_ref: u8,

    /// Stores the ADA automatic-trigger interval in milliseconds; zero disables it.
    pub trig_timer_value: u16,

    /// Uses `0` for the falling-edge trigger and `1` for the rising-edge trigger configured by `TRL`.
    pub trig_level: u8,

    /// Stores the AVR UBRR divisor persisted by `SBD`; startup validates it before enabling double-speed UART mode.
    pub ee_ser_baud_reg: u8,

    /// Counts protocol errors returned by `ERC` until that command clears the counter.
    pub err_count: i32,

    /// Caches the packed protocol status byte: error in the low nibble, then unlock, overload, user-request, and busy bits.
    pub status: u8,

    /// Stores the active 7-bit I2C slave address used by ICB, ICW, ICS, ICT, and ICA operations.
    pub i2c_slave_adr: u8,

    /// Latches an external, automatic, or command trigger until the polling loop services it.
    pub trigger: bool,

    /// Records the active-low command LED level asserted after valid parser activity.
    pub led_activity_low: bool,

    /// Counts the 125-systick activity-LED hold time loaded after a valid command.
    pub activity_timer_ticks: u8,

    /// Stores vers1 string in the wire or LCD representation expected by the original firmware.
    pub vers1_str: String,

    /// Stores egg string in the wire or LCD representation expected by the original firmware.
    pub egg_str: String,

    /// Stores parameter text array in the wire or LCD representation expected by the original firmware.
    pub param_text_array: Vec<String>,

    /// Stores eight unclamped DAC codes indexed by output channel 0..7.
    pub dac_raw_array: [u16; 8],

    /// Stores eight calibrated DAC setpoints indexed by output channel 0..7.
    pub dac_value_array: [f32; 8],

    /// Stores converter-count offsets in the protocol's ADC/DAC calibration slot order.
    pub offset_array: [i32; 28],

    /// Stores multiplicative calibration factors in the protocol's ADC/DAC calibration slot order.
    pub scale_array: [f32; 30],

    /// Holds the eight EEPROM startup bytes changed by subchannels 180..187.
    pub port_init_array: [u8; 8],

    /// Stores eight startup direction bytes indexed by logical I/O port 0..7.
    pub dir_init_array: [u8; 8],

    /// Stores four trigger masks indexed by AD10, AD16, DAC, and digital-port scan group.
    pub trig_mask_array: [u8; 4],

    /// Stores eight internal 10-bit ADC samples indexed by channel 0..7.
    pub adc10_raw_array: [i32; 8],

    /// Stores external ADC samples indexed by the corresponding protocol channel.
    pub adc_raw_array: [i32; 8],

    /// Holds the eight live output bytes; local 4094 writes transmit the complete image in one transaction.
    pub port_array: [u8; 8],

    /// Caches eight logical input-port bytes indexed by ADA port number 0..7.
    pub io_pin_array: [u8; 8],

    /// Caches eight expander direction bytes indexed by ADA port number 0..7.
    pub dir_output_array: [u8; 8],

    /// Selects I2C-expander writes instead of the local 4094 output image.
    pub io_present: bool,

    /// Selects LTC1257 12-bit output conversion for DAC subchannels 20..27.
    pub dac12_present: bool,

    /// Selects LTC1655 16-bit offset-binary output conversion.
    pub dac16_present: bool,

    /// Selects DAC714 signed 16-bit output conversion.
    pub dac714_present: bool,

    /// Enables AD16 subchannels 10..17 in reads and trigger scans.
    pub adc16_present: bool,

    /// Records display detection for startup capability reporting.
    pub lcd_present: bool,

    /// Caches the AD10 full-scale divisor used before per-channel calibration.
    pub base_scale_ad10: f32,

    /// Caches the AD16 full-scale divisor used before per-channel calibration.
    pub base_scale_ad16: f32,

    /// Caches the DA12 counts-per-unit factor used by output conversion.
    pub base_scale_da12: f32,

    /// Caches the DA16 counts-per-unit factor used by output conversion.
    pub base_scale_da16: f32,

    /// Records whether ADA selected the AVR internal ADC reference.
    pub internal_reference: bool,

    /// Latches trigger positive edge with the corresponding ADC sample for status and protection reporting.
    pub trigger_positive_edge: bool,

    /// Queues byte values returned by successive `ICB` host-model reads.
    pub i2c_byte_reads: VecDeque<u8>,

    /// Queues word values returned by successive `ICW` or `ICS` host-model reads.
    pub i2c_word_reads: VecDeque<u16>,

    /// Records `(slave, payload)` pairs issued by byte, word, and port-expander commands.
    pub i2c_writes: Vec<(u8, u16)>,

    /// Records each complete eight-port image sent to the local 4094 chain.
    pub shift_register_writes: Vec<[u8; 8]>,
}

impl Default for ParseContext {
    /// Builds parser state with the Pascal calibration-slot defaults and no detected daughterboards.
    #[rustfmt::skip]
    fn default() -> Self {
        let mut offset_array = [0; 28];
        offset_array[10..=17].fill(-40);

        Self {
            ser_inp_str: String::new(),
            ser_inp_ptr: 0,
            param_str: String::new(),
            param: 0.0,
            param_int: 0,
            param_byte: 0,
            sub_ch: 0,
            current_ch: 0,
            slave_ch: 0,
            cmd_which: CmdWhich::Err,
            verbose: false,
            changed_flag: false,
            ee_unlocked: false,
            modify: 0,
            inc_rast: 4,
            inc_rast_def: 4,
            integrate_ad16: false,
            init_integrate_ad16: false,
            ext_ref: 1,
            trig_timer_value: 0,
            trig_level: 0,
            ee_ser_baud_reg: 51,
            err_count: 0,
            status: 0,
            i2c_slave_adr: 0,
            trigger: false,
            led_activity_low: false,
            activity_timer_ticks: 0,
            vers1_str: String::new(),
            egg_str: String::new(),
            param_text_array: vec![String::new(); 38],
            dac_raw_array: [0; 8],
            dac_value_array: [0.0; 8],
            offset_array,
            scale_array: [
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                0.0,
                100.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                0.0,
                3185.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                1.0,
                200.0,
                3200.0,
            ],
            port_init_array: [0; 8],
            dir_init_array: [0; 8],
            trig_mask_array: [0; 4],
            adc10_raw_array: [0; 8],
            adc_raw_array: [0; 8],
            port_array: [0; 8],
            io_pin_array: [0; 8],
            dir_output_array: [0; 8],
            io_present: false,
            dac12_present: false,
            dac16_present: false,
            dac714_present: false,
            adc16_present: false,
            lcd_present: false,
            base_scale_ad10: 100.0,
            base_scale_ad16: 3185.0,
            base_scale_da12: 200.0,
            base_scale_da16: 3200.0,
            internal_reference: false,
            trigger_positive_edge: false,
            i2c_byte_reads: VecDeque::new(),
            i2c_word_reads: VecDeque::new(),
            i2c_writes: Vec::new(),
            shift_register_writes: Vec::new(),
        }
    }
}
