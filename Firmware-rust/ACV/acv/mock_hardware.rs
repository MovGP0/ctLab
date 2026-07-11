#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone)]
pub(super) struct MockHardware {
    pub(super) port_b: u8,
    pub(super) port_c: u8,
    pub(super) port_d: u8,
    pub(super) pin_d: u8,
    pub(super) adc_config: u8,
    pub(super) adc_values: [u16; 8],
    pub(super) i2c_registers: [u8; 256],
    pub(super) lcd_lines: [String; 2],
    pub(super) lcd_present: bool,
    pub(super) serial_input: VecDeque<char>,
    pub(super) serial_output: String,
    pub(super) aux_serial_log: Vec<u8>,
    pub(super) aux_serial_bits: Vec<bool>,
    pub(super) uart_baud_reg: u8,
    pub(super) uart_double_speed: bool,
    pub(super) led_activity: bool,
    pub(super) rotary_value: i32,
    pub(super) button_temp: u8,
    pub(super) button_debounce_sample: u8,
    pub(super) button_waiting_for_release: bool,
}

impl MockHardware {
    pub(super) fn get_adc(&self, channel: usize) -> u16 {
        self.adc_values.get(channel).copied().unwrap_or_default()
    }

    pub(super) fn twi_out_10(&mut self, register: u8, data: u8) {
        self.i2c_registers[register as usize] = data;
    }

    pub(super) fn twi_in_10(&self, register: u8) -> u8 {
        self.i2c_registers[register as usize]
    }

    pub(super) fn set_aux_serial_line(&mut self, high: bool) {
        if high {
            self.port_b |= 1 << B_SER_AUX;
        } else {
            self.port_b &= !(1 << B_SER_AUX);
        }
        self.aux_serial_bits.push(high);
    }

    pub(super) fn lcd_write_line(&mut self, row: usize, text: String) {
        if row < self.lcd_lines.len() {
            self.lcd_lines[row] = Self::lcd_fixed_text(&text);
        }
    }

    pub(super) fn lcd_fixed_text(text: &str) -> String {
        let mut line: String = text.chars().take(LCD_COLUMNS).collect();
        while line.chars().count() < LCD_COLUMNS {
            line.push(' ');
        }
        line
    }

    pub(super) fn serial_read_timeout(&mut self, _timeout_ticks: u8) -> Option<char> {
        self.serial_input.pop_front()
    }

    pub(super) fn serial_pending(&self) -> bool {
        !self.serial_input.is_empty()
    }

    pub(super) fn lcd_bar_out(&mut self, row: usize, value: u8) {
        let mut line = vec![' '; LCD_COLUMNS];
        let segments = usize::from(value / 32).min(7);
        for column in 1..=segments {
            line[column] = '#';
        }
        self.lcd_write_line(row, line.into_iter().collect());
    }

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
