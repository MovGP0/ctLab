//! ATmega644 controller for the c't-Lab FPGA module.
//!
//! The original `FPGA.pas` is controller firmware, not FPGA gateware. This
//! port covers its stateful register bridge, serial parser dispatch, FAT-style
//! file operations, FPGA configuration stream, and auto-increment data paths.

use super::fpga_hw::{FpgaBus, FpgaHardware};
use super::fpga_parser::{parse_frame, Parameter, ParseError, ParsedFrame};

pub const VERSION: &str = "2.61 [FPGA by CM/c't 06/2008]";
pub const EEPROM_INITIALIZED: u16 = 0xAA55;
pub const DEFAULT_INIT_FILE: &str = "BASIC.INI";
pub const DEFAULT_DATA_FILE: &str = "DATAFILE.XLS";
pub const REGISTER_COUNT: usize = 10;
pub const FPGA_REGISTER_COUNT: usize = 64;

pub trait FileSystem
{
    type Error;

    fn card_present(&mut self) -> bool;

    fn list_root(&mut self) -> Result<Vec<String>, Self::Error>;

    fn read_file(&mut self, name: &str) -> Result<Vec<u8>, Self::Error>;

    fn write_file(&mut self, name: &str, data: &[u8]) -> Result<(), Self::Error>;

    fn append_file(&mut self, name: &str, data: &[u8]) -> Result<(), Self::Error>;

    fn delete_file(&mut self, name: &str) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerError<E>
{
    Parse(ParseError),
    NoCard,
    File(E),
    ConfigurationFailed,
    InvalidRegister,
    InvalidParameter,
    DivisionByZero,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response
{
    None,
    Number(f64),
    Integer(i64),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct EepromSettings
{
    pub initialized: u16,
    pub serial_baud_register: u8,
    pub hex_mode: bool,
    pub options: [i16; 20],
    pub initial_registers: [i32; 4],
    pub init_file_name: String,
    pub data_file_name: String,
}

impl Default for EepromSettings
{
    fn default() -> Self
    {
        Self
        {
            initialized: EEPROM_INITIALIZED,
            serial_baud_register: 51,
            hex_mode: false,
            options: [
                255, 255, 0, 128, 0, 10, 7, 255, 9, 500, 64, 65, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            initial_registers: [0; 4],
            init_file_name: DEFAULT_INIT_FILE.to_string(),
            data_file_name: DEFAULT_DATA_FILE.to_string(),
        }
    }
}

pub struct FpgaController<H, F>
{
    pub bus: FpgaBus<H>,
    pub files: F,
    pub eeprom: EepromSettings,
    pub main_channel: u8,
    pub transfer_main_channel: u8,
    pub transfer_subchannel: u16,
    pub registers: [f64; REGISTER_COUNT],
    pub output_registers: [u32; FPGA_REGISTER_COUNT],
    pub input_registers: [u32; FPGA_REGISTER_COUNT],
    pub auto_increment_register: u8,
    pub auto_increment_select: u8,
    pub auto_increment_width: u8,
    pub auto_increment_start: u32,
    pub auto_increment_end: u32,
    pub directory: Vec<String>,
    pub card_ok: bool,
    pub error_count: u32,
}

impl<H: FpgaHardware, F: FileSystem> FpgaController<H, F>
{
    pub fn new(hardware: H, files: F, eeprom: EepromSettings) -> Self
    {
        let mut registers = [0.0; REGISTER_COUNT];
        for (target, source) in registers.iter_mut().zip(eeprom.initial_registers)
        {
            *target = source as f64;
        }
        let main_channel = eeprom.options[8] as u8;
        let core_rx_subchannel = eeprom.options[10] as u8;
        let core_tx_subchannel = eeprom.options[11] as u8;
        Self
        {
            bus: FpgaBus::new(hardware, core_rx_subchannel, core_tx_subchannel),
            files,
            eeprom,
            main_channel,
            transfer_main_channel: 0,
            transfer_subchannel: 10,
            registers,
            output_registers: [0; FPGA_REGISTER_COUNT],
            input_registers: [0; FPGA_REGISTER_COUNT],
            auto_increment_register: 128,
            auto_increment_select: 0,
            auto_increment_width: 4,
            auto_increment_start: 0,
            auto_increment_end: 0,
            directory: Vec::new(),
            card_ok: false,
            error_count: 0,
        }
    }

    pub fn check_card(&mut self) -> bool
    {
        self.card_ok = self.files.card_present();
        self.card_ok
    }

    pub fn refresh_directory(&mut self) -> Result<&[String], ControllerError<F::Error>>
    {
        if !self.check_card()
        {
            return Err(ControllerError::NoCard);
        }
        self.directory = self.files.list_root().map_err(ControllerError::File)?;
        self.directory.truncate(64);
        Ok(&self.directory)
    }

    /// Configures the FPGA from a bitstream, preserving the Pascal PROG pulse,
    /// DONE-low acknowledgement, 256-byte streaming, and trailing clocks.
    pub fn load_fpga_configuration(&mut self, file_name: &str) -> Result<usize, ControllerError<F::Error>>
    {
        if !self.check_card()
        {
            return Err(ControllerError::NoCard);
        }
        let data = self.files.read_file(file_name).map_err(ControllerError::File)?;
        self.bus.hardware_mut().set_configuration_program(false);
        self.bus.hardware_mut().delay_us(1);
        self.bus.hardware_mut().set_configuration_program(true);
        self.bus.hardware_mut().delay_us(10_000);
        if self.bus.hardware().configuration_done()
        {
            return Err(ControllerError::ConfigurationFailed);
        }

        let mut sent = 0;
        for block in data.chunks(256)
        {
            self.bus.shift_configuration(block);
            sent += block.len();
            if self.bus.hardware().configuration_done()
            {
                break;
            }
        }
        self.bus.shift_configuration(&[0xFF]);
        if !self.bus.hardware().configuration_done()
        {
            return Err(ControllerError::ConfigurationFailed);
        }
        Ok(sent)
    }

    pub fn exchange_fpga_register(&mut self, register: u8) -> Result<u32, ControllerError<F::Error>>
    {
        let index = register as usize;
        if index >= FPGA_REGISTER_COUNT
        {
            return Err(ControllerError::InvalidRegister);
        }
        let received = self.bus.exchange_u32(register, self.output_registers[index]);
        self.input_registers[index] = received;
        Ok(received)
    }

    pub fn setup_auto_increment(&mut self, for_read: bool)
    {
        self.bus.exchange_u8(self.auto_increment_register.wrapping_add(1), self.auto_increment_select);
        let address_register = self.auto_increment_register.wrapping_add(if for_read { 3 } else { 2 });
        self.bus.exchange_u32(address_register, self.auto_increment_start);
        self.bus.send_register(self.auto_increment_register);
    }

    pub fn reset_auto_increment(&mut self)
    {
        self.bus.exchange_u8(self.auto_increment_register.wrapping_add(1), 0);
    }

    pub fn load_data_file(&mut self, file_name: &str) -> Result<usize, ControllerError<F::Error>>
    {
        if !self.check_card()
        {
            return Err(ControllerError::NoCard);
        }
        let data = self.files.read_file(file_name).map_err(ControllerError::File)?;
        self.setup_auto_increment(false);
        match self.auto_increment_width
        {
            2 =>
            {
                for bytes in data.chunks_exact(2)
                {
                    self.bus.exchange_u16(self.auto_increment_register, u16::from_le_bytes([bytes[0], bytes[1]]));
                }
            }
            4 =>
            {
                for bytes in data.chunks_exact(4)
                {
                    self.bus.exchange_u32(
                        self.auto_increment_register,
                        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    );
                }
            }
            _ =>
            {
                for byte in &data
                {
                    self.bus.exchange_u8(self.auto_increment_register, *byte);
                }
            }
        }
        self.reset_auto_increment();
        Ok(data.len())
    }

    pub fn save_data_file(&mut self, file_name: &str) -> Result<usize, ControllerError<F::Error>>
    {
        if !self.check_card()
        {
            return Err(ControllerError::NoCard);
        }
        let byte_count = self.auto_increment_end.saturating_sub(self.auto_increment_start) as usize;
        self.setup_auto_increment(true);
        let mut data = Vec::with_capacity(byte_count);
        for _ in 0..byte_count
        {
            data.push(self.bus.exchange_u8(self.auto_increment_register, 0));
        }
        self.reset_auto_increment();
        self.files.write_file(file_name, &data).map_err(ControllerError::File)?;
        Ok(data.len())
    }

    pub fn type_file(&mut self, file_name: &str, register: Option<u8>) -> Result<usize, ControllerError<F::Error>>
    {
        if !self.check_card()
        {
            return Err(ControllerError::NoCard);
        }
        let data = self.files.read_file(file_name).map_err(ControllerError::File)?;
        match register
        {
            Some(register) =>
            {
                for byte in &data
                {
                    self.bus.exchange_u8(register, *byte);
                    self.bus.hardware_mut().delay_us(if *byte == b'\r' { 50_000 } else { 10 });
                }
            }
            None =>
            {
                for byte in &data
                {
                    self.bus.route_serial_byte(*byte);
                    if *byte == b'\r'
                    {
                        self.bus.route_serial_byte(b'\n');
                        self.bus.hardware_mut().delay_us(10_000);
                    }
                }
            }
        }
        Ok(data.len())
    }

    pub fn parse_and_execute(&mut self, input: &str) -> Result<Response, ControllerError<F::Error>>
    {
        let frame = parse_frame(input).map_err(ControllerError::Parse)?;
        if let Some(channel) = frame.main_channel
        {
            if channel != self.main_channel && channel != 9
            {
                return Ok(Response::None);
            }
        }
        self.execute(frame)
    }

    fn execute(&mut self, frame: ParsedFrame) -> Result<Response, ControllerError<F::Error>>
    {
        if frame.is_request
        {
            return self.get_subchannel(frame.subchannel);
        }
        self.set_subchannel(frame.subchannel, frame.parameter)?;
        Ok(Response::None)
    }

    fn get_subchannel(&mut self, subchannel: u16) -> Result<Response, ControllerError<F::Error>>
    {
        match subchannel
        {
            254 => Ok(Response::Text(VERSION.to_string())),
            251 => Ok(Response::Integer(self.error_count as i64)),
            300..=309 => Ok(Response::Number(self.registers[(subchannel - 300) as usize])),
            2000..=2063 => self
                .exchange_fpga_register((subchannel - 2000) as u8)
                .map(|value| Response::Integer(value as i64)),
            242 => Ok(Response::Integer(self.directory.len() as i64)),
            243 => Ok(Response::Text(self.eeprom.data_file_name.clone())),
            _ => Ok(Response::None),
        }
    }

    fn set_subchannel(&mut self, subchannel: u16, parameter: Parameter) -> Result<(), ControllerError<F::Error>>
    {
        let number = parameter_number(&parameter);
        match subchannel
        {
            300..=309 => self.registers[(subchannel - 300) as usize] = number?,
            310..=319 =>
            {
                let destination = (subchannel - 310) as usize;
                let source = number? as usize;
                if source >= REGISTER_COUNT
                {
                    return Err(ControllerError::InvalidRegister);
                }
                self.registers[destination] = self.registers[source];
            }
            320..=329 => self.registers[(subchannel - 320) as usize] -= 1.0,
            330..=339 => self.registers[(subchannel - 330) as usize] += 1.0,
            350..=359 =>
            {
                let other = (subchannel - 350) as usize;
                self.registers.swap(0, other);
            }
            600..=609 => self.registers[0] *= self.registers[(subchannel - 600) as usize],
            610..=619 =>
            {
                let divisor = self.registers[(subchannel - 610) as usize];
                if divisor == 0.0
                {
                    return Err(ControllerError::DivisionByZero);
                }
                self.registers[0] /= divisor;
            }
            620..=629 => self.registers[0] += self.registers[(subchannel - 620) as usize],
            630..=639 => self.registers[0] -= self.registers[(subchannel - 630) as usize],
            640..=649 =>
            {
                let index = (subchannel - 640) as usize;
                self.registers[index] = self.registers[index].sqrt();
            }
            650..=659 =>
            {
                let index = (subchannel - 650) as usize;
                self.registers[index] *= self.registers[index];
            }
            660..=669 =>
            {
                let index = (subchannel - 660) as usize;
                self.registers[index] = -self.registers[index];
            }
            2000..=2063 =>
            {
                let register = (subchannel - 2000) as usize;
                self.output_registers[register] = number? as u32;
                self.exchange_fpga_register(register as u8)?;
            }
            280 => self.auto_increment_register = number? as u8,
            281 => self.auto_increment_select = number? as u8,
            282 => self.auto_increment_width = number? as u8,
            285 => self.auto_increment_start = number? as u32,
            286 => self.auto_increment_end = number? as u32,
            243 => self.eeprom.data_file_name = parameter_text(parameter)?,
            244 =>
            {
                let file_name = parameter_text(parameter)?;
                self.files.delete_file(&file_name).map_err(ControllerError::File)?;
                self.refresh_directory()?;
            }
            251 => self.error_count = 0,
            252 => self.eeprom.serial_baud_register = number? as u8,
            _ => {}
        }
        Ok(())
    }
}

fn parameter_number<E>(parameter: &Parameter) -> Result<f64, ControllerError<E>>
{
    match parameter
    {
        Parameter::Number(value) => Ok(*value),
        _ => Err(ControllerError::InvalidParameter),
    }
}

fn parameter_text<E>(parameter: Parameter) -> Result<String, ControllerError<E>>
{
    match parameter
    {
        Parameter::Text(value) => Ok(value),
        _ => Err(ControllerError::InvalidParameter),
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::{BTreeMap, VecDeque};

    use super::*;

    #[derive(Default)]
    struct MockHardware
    {
        selected: Vec<u8>,
        sent: Vec<Vec<u8>>,
        replies: VecDeque<Vec<u8>>,
        configuration: Vec<u8>,
        program: Vec<bool>,
        done_reads: std::cell::Cell<usize>,
    }

    impl FpgaHardware for MockHardware
    {
        fn external_serial_write(&mut self, _byte: u8) {}

        fn select_fpga_register(&mut self, register: u8)
        {
            self.selected.push(register);
        }

        fn exchange_fpga_data(&mut self, tx: &[u8], rx: &mut [u8])
        {
            self.sent.push(tx.to_vec());
            if let Some(reply) = self.replies.pop_front()
            {
                rx.copy_from_slice(&reply);
            }
        }

        fn shift_configuration_byte(&mut self, byte: u8)
        {
            self.configuration.push(byte);
        }

        fn set_configuration_program(&mut self, high: bool)
        {
            self.program.push(high);
        }

        fn configuration_done(&self) -> bool
        {
            let read = self.done_reads.get();
            self.done_reads.set(read + 1);
            read >= 1
        }

        fn delay_us(&mut self, _microseconds: u16) {}
    }

    #[derive(Default)]
    struct MemoryFiles
    {
        present: bool,
        files: BTreeMap<String, Vec<u8>>,
    }

    impl FileSystem for MemoryFiles
    {
        type Error = ();

        fn card_present(&mut self) -> bool
        {
            self.present
        }

        fn list_root(&mut self) -> Result<Vec<String>, Self::Error>
        {
            Ok(self.files.keys().cloned().collect())
        }

        fn read_file(&mut self, name: &str) -> Result<Vec<u8>, Self::Error>
        {
            self.files.get(name).cloned().ok_or(())
        }

        fn write_file(&mut self, name: &str, data: &[u8]) -> Result<(), Self::Error>
        {
            self.files.insert(name.to_string(), data.to_vec());
            Ok(())
        }

        fn append_file(&mut self, name: &str, data: &[u8]) -> Result<(), Self::Error>
        {
            self.files.entry(name.to_string()).or_default().extend_from_slice(data);
            Ok(())
        }

        fn delete_file(&mut self, name: &str) -> Result<(), Self::Error>
        {
            self.files.remove(name).map(|_| ()).ok_or(())
        }
    }

    fn controller() -> FpgaController<MockHardware, MemoryFiles>
    {
        let mut files = MemoryFiles::default();
        files.present = true;
        FpgaController::new(MockHardware::default(), files, EepromSettings::default())
    }

    #[test]
    fn eeprom_defaults_match_pascal_layout()
    {
        let controller = controller();
        assert_eq!(controller.main_channel, 9);
        assert_eq!(controller.bus.core_rx_subchannel, 64);
        assert_eq!(controller.bus.core_tx_subchannel, 65);
        assert_eq!(controller.auto_increment_register, 128);
    }

    #[test]
    fn configuration_load_pulses_program_and_streams_file()
    {
        let mut controller = controller();
        controller.files.files.insert("FPGA0.BIN".to_string(), vec![1, 2, 3]);

        assert_eq!(controller.load_fpga_configuration("FPGA0.BIN"), Ok(3));
        assert_eq!(controller.bus.hardware().program, vec![false, true]);
        assert_eq!(controller.bus.hardware().configuration, vec![1, 2, 3, 0xFF]);
    }

    #[test]
    fn parser_dispatches_register_arithmetic()
    {
        let mut controller = controller();
        assert_eq!(
            controller.parse_and_execute("9:IDN"),
            Ok(Response::Text("2.61 [FPGA by CM/c't 06/2008]".to_string()))
        );
        controller.parse_and_execute("9:REG 0=5").unwrap();
        controller.parse_and_execute("9:REG 1=3").unwrap();
        controller.parse_and_execute("9:ADD 1=0").unwrap();

        assert_eq!(controller.parse_and_execute("9:REG 0"), Ok(Response::Number(8.0)));
    }

    #[test]
    fn little_endian_data_file_is_sent_as_big_endian_spi_words()
    {
        let mut controller = controller();
        controller.auto_increment_width = 4;
        controller.files.files.insert("DATA.DAT".to_string(), vec![1, 2, 3, 4]);

        assert_eq!(controller.load_data_file("DATA.DAT"), Ok(4));
        assert!(controller.bus.hardware().sent.contains(&vec![4, 3, 2, 1]));
    }

    #[test]
    fn directory_is_limited_to_pascal_capacity()
    {
        let mut controller = controller();
        for index in 0..70
        {
            controller.files.files.insert(format!("{index:02}.BIN"), Vec::new());
        }

        assert_eq!(controller.refresh_directory().unwrap().len(), 64);
    }
}
