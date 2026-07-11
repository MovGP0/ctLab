//! Hardware-facing portion of the FPGA module controller firmware.
//!
//! This is a behavioral port of `FPGA-HW.pas`. The original ATmega644 uses
//! SPI for a register-select transaction followed by an 8/16/32-bit data
//! transaction. Multi-byte values are transferred most-significant byte first.

pub const CORE_ACK: u8 = 0x06;
pub const CORE_BUFFER_CAPACITY: usize = 256;

pub trait FpgaHardware
{
    fn external_serial_write(&mut self, byte: u8);

    fn select_fpga_register(&mut self, register: u8);

    fn exchange_fpga_data(&mut self, tx: &[u8], rx: &mut [u8]);

    fn shift_configuration_byte(&mut self, byte: u8);

    fn set_configuration_program(&mut self, high: bool);

    fn configuration_done(&self) -> bool;

    fn delay_us(&mut self, microseconds: u16);
}

#[derive(Debug)]
pub struct FpgaBus<H>
{
    hardware: H,
    pub core_rx_subchannel: u8,
    pub core_tx_subchannel: u8,
    pub internal_serial: bool,
    receive_buffer: [u8; CORE_BUFFER_CAPACITY],
    receive_read_index: u8,
    receive_write_index: u8,
    receive_len: u16,
    dropped_receive_bytes: u32,
}

impl<H: FpgaHardware> FpgaBus<H>
{
    pub fn new(hardware: H, core_rx_subchannel: u8, core_tx_subchannel: u8) -> Self
    {
        Self
        {
            hardware,
            core_rx_subchannel,
            core_tx_subchannel,
            internal_serial: false,
            receive_buffer: [0; CORE_BUFFER_CAPACITY],
            receive_read_index: 0,
            receive_write_index: 0,
            receive_len: 0,
            dropped_receive_bytes: 0,
        }
    }

    pub fn hardware(&self) -> &H
    {
        &self.hardware
    }

    pub fn hardware_mut(&mut self) -> &mut H
    {
        &mut self.hardware
    }

    pub fn into_hardware(self) -> H
    {
        self.hardware
    }

    pub fn send_register(&mut self, register: u8)
    {
        self.hardware.select_fpga_register(register);
    }

    pub fn exchange_u8(&mut self, register: u8, value: u8) -> u8
    {
        self.send_register(register);
        let mut received = [0];
        self.hardware.exchange_fpga_data(&[value], &mut received);
        received[0]
    }

    pub fn exchange_u16(&mut self, register: u8, value: u16) -> u16
    {
        self.send_register(register);
        let sent = value.to_be_bytes();
        let mut received = [0; 2];
        self.hardware.exchange_fpga_data(&sent, &mut received);
        u16::from_be_bytes(received)
    }

    pub fn exchange_u32(&mut self, register: u8, value: u32) -> u32
    {
        self.send_register(register);
        let sent = value.to_be_bytes();
        let mut received = [0; 4];
        self.hardware.exchange_fpga_data(&sent, &mut received);
        u32::from_be_bytes(received)
    }

    pub fn exchange_registers_0_to_3(
        &mut self,
        output: &[u32; 4],
        input: &mut [u32; 4],
    )
    {
        for register in 0..4
        {
            input[register] = self.exchange_u32(register as u8, output[register]);
        }
    }

    pub fn route_serial_byte(&mut self, byte: u8)
    {
        if !self.internal_serial
        {
            self.hardware.external_serial_write(byte);
            return;
        }

        // Pascal's `mySerOut` suppresses LF on the internal FPGA channel and
        // limits the stream to 10 kcharacters/s with a 100 us delay.
        if byte == b'\n'
        {
            return;
        }

        self.exchange_u8(self.core_tx_subchannel, byte);
        self.hardware.delay_us(100);
    }

    pub fn route_serial_bytes(&mut self, bytes: &[u8])
    {
        for &byte in bytes
        {
            self.route_serial_byte(byte);
        }
    }

    pub fn send_core_ack(&mut self)
    {
        if self.internal_serial
        {
            self.exchange_u8(self.core_tx_subchannel, CORE_ACK);
        }
    }

    /// Mirrors the INT2 handler: fetch one byte from the configured core RX
    /// register and enqueue it unless it is NUL or LF.
    pub fn receive_core_interrupt(&mut self)
    {
        let received = self.exchange_u8(self.core_rx_subchannel, 0);
        if matches!(received, 0 | b'\n')
        {
            return;
        }

        if usize::from(self.receive_len) == CORE_BUFFER_CAPACITY
        {
            self.dropped_receive_bytes = self.dropped_receive_bytes.saturating_add(1);
            return;
        }

        self.receive_buffer[usize::from(self.receive_write_index)] = received;
        self.receive_write_index = self.receive_write_index.wrapping_add(1);
        self.receive_len += 1;
    }

    pub fn core_serial_pending(&self) -> bool
    {
        self.receive_len != 0
    }

    pub fn read_core_serial(&mut self) -> Option<u8>
    {
        if self.receive_len == 0
        {
            return None;
        }
        let received = self.receive_buffer[usize::from(self.receive_read_index)];
        self.receive_read_index = self.receive_read_index.wrapping_add(1);
        self.receive_len -= 1;
        Some(received)
    }

    pub fn dropped_receive_bytes(&self) -> u32
    {
        self.dropped_receive_bytes
    }

    pub fn shift_configuration(&mut self, bytes: &[u8])
    {
        for &byte in bytes
        {
            self.hardware.shift_configuration_byte(byte);
        }
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::VecDeque;

    use super::*;

    #[derive(Default, Debug)]
    struct MockHardware
    {
        external: Vec<u8>,
        selected: Vec<u8>,
        transfers: Vec<Vec<u8>>,
        replies: VecDeque<Vec<u8>>,
        configured: Vec<u8>,
        program_levels: Vec<bool>,
        configuration_done: bool,
        delays: Vec<u16>,
    }

    impl FpgaHardware for MockHardware
    {
        fn external_serial_write(&mut self, byte: u8)
        {
            self.external.push(byte);
        }

        fn select_fpga_register(&mut self, register: u8)
        {
            self.selected.push(register);
        }

        fn exchange_fpga_data(&mut self, tx: &[u8], rx: &mut [u8])
        {
            self.transfers.push(tx.to_vec());
            if let Some(reply) = self.replies.pop_front()
            {
                rx.copy_from_slice(&reply);
            }
        }

        fn shift_configuration_byte(&mut self, byte: u8)
        {
            self.configured.push(byte);
        }

        fn set_configuration_program(&mut self, high: bool)
        {
            self.program_levels.push(high);
        }

        fn configuration_done(&self) -> bool
        {
            self.configuration_done
        }

        fn delay_us(&mut self, microseconds: u16)
        {
            self.delays.push(microseconds);
        }
    }

    #[test]
    fn multibyte_spi_transfers_are_most_significant_byte_first()
    {
        let mut hardware = MockHardware::default();
        hardware.replies.push_back(vec![0xA1, 0xB2, 0xC3, 0xD4]);
        let mut bus = FpgaBus::new(hardware, 64, 65);

        let received = bus.exchange_u32(7, 0x1020_3040);

        assert_eq!(received, 0xA1B2_C3D4);
        assert_eq!(bus.hardware().selected, vec![7]);
        assert_eq!(bus.hardware().transfers, vec![vec![0x10, 0x20, 0x30, 0x40]]);
    }

    #[test]
    fn internal_serial_suppresses_lf_and_uses_core_tx_register()
    {
        let hardware = MockHardware::default();
        let mut bus = FpgaBus::new(hardware, 64, 65);
        bus.internal_serial = true;

        bus.route_serial_bytes(b"A\nB");

        assert_eq!(bus.hardware().selected, vec![65, 65]);
        assert_eq!(bus.hardware().transfers, vec![vec![b'A'], vec![b'B']]);
        assert_eq!(bus.hardware().delays, vec![100, 100]);
    }

    #[test]
    fn interrupt_buffer_ignores_nul_and_lf_but_keeps_other_bytes()
    {
        let mut hardware = MockHardware::default();
        hardware.replies.extend([vec![0], vec![b'\n'], vec![b'X']]);
        let mut bus = FpgaBus::new(hardware, 64, 65);

        bus.receive_core_interrupt();
        bus.receive_core_interrupt();
        bus.receive_core_interrupt();

        assert_eq!(bus.read_core_serial(), Some(b'X'));
        assert_eq!(bus.read_core_serial(), None);
    }

    #[test]
    fn configuration_stream_preserves_byte_order()
    {
        let hardware = MockHardware::default();
        let mut bus = FpgaBus::new(hardware, 64, 65);

        bus.shift_configuration(&[0xAA, 0x55, 0x12]);

        assert_eq!(bus.hardware().configured, vec![0xAA, 0x55, 0x12]);
    }
}
