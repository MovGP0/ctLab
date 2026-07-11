use crate::test_failures::TestFailures;
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
    fn external_serial_write(&mut self, _byte: u8) {
        // This controller test does not observe forwarded serial bytes.
    }

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

    fn delay_us(&mut self, _microseconds: u16) {
        // Timing is intentionally omitted from the deterministic test double.
    }
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
    let files = MemoryFiles {
        present: true,
        ..MemoryFiles::default()
    };
    FpgaController::new(MockHardware::default(), files, EepromSettings::default())
}

#[test]
fn eeprom_defaults_match_pascal_layout()
{
    let mut assert = TestFailures::default();

    let controller = controller();
    assert.eq(controller.main_channel, 9);
    assert.eq(controller.bus.core_rx_subchannel, 64);
    assert.eq(controller.bus.core_tx_subchannel, 65);
    assert.eq(controller.auto_increment_register, 128);
    assert.finish();
}

#[test]
fn configuration_load_pulses_program_and_streams_file()
{
    let mut assert = TestFailures::default();

    let mut controller = controller();
    controller.files.files.insert("FPGA0.BIN".to_string(), vec![1, 2, 3]);

    assert.eq(controller.load_fpga_configuration("FPGA0.BIN"), Ok(3));
    assert.eq(controller.bus.hardware().program.as_slice(), [false, true]);
    assert.eq(controller.bus.hardware().configuration.as_slice(), [1, 2, 3, 0xFF]);
    assert.finish();
}

#[test]
fn parser_dispatches_register_arithmetic()
{
    let mut assert = TestFailures::default();

    let mut controller = controller();
    assert.eq(
        controller.parse_and_execute("9:IDN"),
        Ok(Response::Text("2.61 [FPGA by CM/c't 06/2008]".to_string())),
    );
    controller.parse_and_execute("9:REG 0=5").unwrap();
    controller.parse_and_execute("9:REG 1=3").unwrap();
    controller.parse_and_execute("9:ADD 1=0").unwrap();

    assert.eq(controller.parse_and_execute("9:REG 0"), Ok(Response::Number(8.0)));
    assert.finish();
}

#[test]
fn little_endian_data_file_is_sent_as_big_endian_spi_words()
{
    let mut assert = TestFailures::default();

    let mut controller = controller();
    controller.auto_increment_width = 4;
    controller.files.files.insert("DATA.DAT".to_string(), vec![1, 2, 3, 4]);

    assert.eq(controller.load_data_file("DATA.DAT"), Ok(4));
    assert.is_true(controller.bus.hardware().sent.contains(&vec![4, 3, 2, 1]));
    assert.finish();
}

#[test]
fn directory_is_limited_to_pascal_capacity()
{
    let mut assert = TestFailures::default();

    let mut controller = controller();
    for index in 0..70
    {
        controller.files.files.insert(format!("{index:02}.BIN"), Vec::new());
    }

    assert.eq(controller.refresh_directory().unwrap().len(), 64);
    assert.finish();
}
