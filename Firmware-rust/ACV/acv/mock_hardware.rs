//! Defines ACV the board-I/O contract that separates protocol logic from register access.

#[allow(unused_imports)]
use super::*;

/// Models ACV GPIO, converter, display, UART, and TWI state for host-side translation tests.
#[derive(Debug, Clone)]
pub(super) struct MockHardware {
    /// Holds the complete Port B output image, including the auxiliary serial line on bit 4.
    pub(super) port_b: u8,

    /// Holds the Port C idle and output levels established by ACV initialization.
    pub(super) port_c: u8,

    /// Holds the Port D idle and output levels established by ACV initialization.
    pub(super) port_d: u8,

    /// Provides scripted Port D strap and button bits to the ACV host model.
    pub(super) pin_d: u8,

    /// Mirrors the ADC board configuration byte written through I2C register 0x08.
    pub(super) adc_config: u8,

    /// Contains adc values in converter counts until scaling or hardware output consumes it.
    pub(super) adc_values: [u16; 8],

    /// Models the byte-addressed CS8406/ADC-board register space reached through TWI address 0x10.
    pub(super) i2c_registers: [u8; 256],

    /// Holds the two fixed-width LCD rows after padding or truncation.
    pub(super) lcd_lines: [String; 2],

    /// Gates panel rendering when startup detection reports no LCD.
    pub(super) lcd_present: bool,

    /// Queues host UART characters consumed by `check_ser`.
    pub(super) serial_input: VecDeque<char>,

    /// Accumulates complete host UART responses in emission order.
    pub(super) serial_output: String,

    /// Records auxiliary command bytes before bit-level transmission.
    pub(super) aux_serial_log: Vec<u8>,

    /// Records the start, eight data, and stop levels driven on Port B bit 4.
    pub(super) aux_serial_bits: Vec<bool>,

    /// Stores the AVR UBRR value applied during ACV initialization.
    pub(super) uart_baud_reg: u8,

    /// Records whether initialization enabled the UART U2X divisor.
    pub(super) uart_double_speed: bool,

    /// Mirrors the active-low command/activity LED output level.
    pub(super) led_activity: bool,

    /// Supplies the absolute rotary-encoder counter sampled by the polling loop.
    pub(super) rotary_value: i32,

    /// Stores button temp until calibration, limit checking, and response formatting have consumed it.
    pub(super) button_temp: u8,

    /// Keeps the first button sample until a matching sample confirms the press.
    pub(super) button_debounce_sample: u8,

    /// Prevents a held front-panel key from generating repeated press events before release.
    pub(super) button_waiting_for_release: bool,
}

impl MockHardware {
    /// Returns the scripted converter count for an ACV ADC channel, or zero outside the eight-channel table.
    pub(super) fn get_adc(&self, channel: usize) -> u16 {
        self.adc_values.get(channel).copied().unwrap_or_default()
    }

    /// Transfers TWI out 10 using the byte order expected by the attached peripheral.
    pub(super) fn twi_out_10(&mut self, register: u8, data: u8) {
        self.i2c_registers[register as usize] = data;
    }

    /// Transfers TWI in 10 using the byte order expected by the attached peripheral.
    pub(super) fn twi_in_10(&self, register: u8) -> u8 {
        self.i2c_registers[register as usize]
    }

    /// Drives Port B bit 4 and records every start, data, and stop level emitted by the auxiliary UART routine.
    pub(super) fn set_aux_serial_line(&mut self, high: bool) {
        if high {
            self.port_b |= 1 << B_SER_AUX;
        } else {
            self.port_b &= !(1 << B_SER_AUX);
        }
        self.aux_serial_bits.push(high);
    }

    /// Renders LCD write line into the fixed LCD cells used by the front panel.
    pub(super) fn lcd_write_line(&mut self, row: usize, text: String) {
        if row < self.lcd_lines.len() {
            self.lcd_lines[row] = Self::lcd_fixed_text(&text);
        }
    }

    /// Renders LCD fixed text into the fixed LCD cells used by the front panel.
    pub(super) fn lcd_fixed_text(text: &str) -> String {
        let mut line: String = text.chars().take(LCD_COLUMNS).collect();
        while line.chars().count() < LCD_COLUMNS {
            line.push(' ');
        }
        line
    }

    /// Polls the serial receiver for one byte while respecting the caller's timeout.
    pub(super) fn serial_read_timeout(&mut self, _timeout_ticks: u8) -> Option<char> {
        self.serial_input.pop_front()
    }

    /// Reports whether a serial byte is waiting so panel work does not race command processing.
    pub(super) fn serial_pending(&self) -> bool {
        !self.serial_input.is_empty()
    }

    /// Renders LCD bar out into the fixed LCD cells used by the front panel.
    pub(super) fn lcd_bar_out(&mut self, row: usize, value: u8) {
        let mut line = vec![' '; LCD_COLUMNS];
        let segments = usize::from(value / 32).min(7);
        for column in 1..=segments {
            line[column] = '#';
        }
        self.lcd_write_line(row, line.into_iter().collect());
    }

    /// Renders LCD write bargraph line into the fixed LCD cells used by the front panel.
    pub(super) fn lcd_write_bargraph_line(&mut self, row: usize, channel: char, value: u8) {
        let mut line = vec![' '; LCD_COLUMNS];
        line[0] = channel;
        let segments = usize::from(value / 32).min(7);
        for column in 1..=segments {
            line[column] = '#';
        }
        if value < 96 {
            line[6] = LCD_ZERO_DB_MARK_CHAR;
        }
        if value > 180 {
            line[7] = LCD_OVERLOAD_BLOCK_CHAR;
        }
        self.lcd_write_line(row, line.into_iter().collect());
    }
}

impl Default for MockHardware {
    /// Builds a disconnected host model with released buttons, empty queues, and cleared outputs.
    fn default() -> Self {
        Self {
            port_b: 0,
            port_c: 0,
            port_d: 0,
            pin_d: 0,
            adc_config: 0,
            adc_values: [0; 8],
            i2c_registers: [0; 256],
            lcd_lines: [String::new(), String::new()],
            lcd_present: false,
            serial_input: VecDeque::new(),
            serial_output: String::new(),
            aux_serial_log: Vec::new(),
            aux_serial_bits: Vec::new(),
            uart_baud_reg: 0,
            uart_double_speed: false,
            led_activity: false,
            rotary_value: 0,
            button_temp: 0xff,
            button_debounce_sample: BUTTON_RELEASED,
            button_waiting_for_release: false,
        }
    }
}
