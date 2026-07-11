//! Width-aware FPGA register exchange and bounded interrupt receive buffering.

use super::*;

/// Register-oriented SPI bridge plus interrupt-safe core serial buffering.
///
/// The ATmega selects an FPGA register first and then exchanges a width-specific
/// big-endian payload. The fixed ring buffer mirrors the bounded ISR storage of
/// the controller and avoids allocation in interrupt context.
#[derive(Debug)]
pub struct FpgaBus<H>
{
    /// Concrete pin/SPI/UART implementation kept private so all transfers obey bus sequencing.
    hardware: H,

    /// FPGA register read by the external interrupt handler for core-originated serial bytes.
    pub core_rx_subchannel: u8,

    /// FPGA register written when serial output is routed through the core.
    pub core_tx_subchannel: u8,

    /// Selects core-register routing instead of the physical ATmega UART.
    pub internal_serial: bool,

    /// Bounded storage shared conceptually by the interrupt producer and foreground consumer.
    receive_buffer: [u8; CORE_BUFFER_CAPACITY],

    /// Next buffered byte consumed by the foreground loop.
    receive_read_index: u8,

    /// Next slot filled by the receive interrupt.
    receive_write_index: u8,

    /// Occupancy disambiguating empty and full states when byte indices wrap.
    receive_len: u16,

    /// Saturating diagnostic counter for bytes discarded rather than overwriting unread data.
    dropped_receive_bytes: u32,
}

impl<H: FpgaHardware> FpgaBus<H>
{
    /// Creates an idle bridge using EEPROM-selected core serial registers.
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

    /// Borrows the backend for status reads that do not alter bus ownership.
    pub fn hardware(&self) -> &H
    {
        &self.hardware
    }

    /// Borrows the backend for controller-level pin sequences such as `PROG` pulsing.
    pub fn hardware_mut(&mut self) -> &mut H
    {
        &mut self.hardware
    }

    /// Returns the backend after the protocol bridge is no longer needed.
    pub fn into_hardware(self) -> H
    {
        self.hardware
    }

    /// Clocks the address phase required before a separate FPGA data transaction.
    pub fn send_register(&mut self, register: u8)
    {
        self.hardware.select_fpga_register(register);
    }

    /// Exchanges one byte with a selected register.
    pub fn exchange_u8(&mut self, register: u8, value: u8) -> u8
    {
        self.send_register(register);
        let mut received = [0];
        self.hardware.exchange_fpga_data(&[value], &mut received);
        received[0]
    }

    /// Exchanges a 16-bit value most-significant byte first, matching the FPGA SPI contract.
    pub fn exchange_u16(&mut self, register: u8, value: u16) -> u16
    {
        self.send_register(register);
        let sent = value.to_be_bytes();
        let mut received = [0; 2];
        self.hardware.exchange_fpga_data(&sent, &mut received);
        u16::from_be_bytes(received)
    }

    /// Exchanges a 32-bit value most-significant byte first, matching the FPGA SPI contract.
    pub fn exchange_u32(&mut self, register: u8, value: u32) -> u32
    {
        self.send_register(register);
        let sent = value.to_be_bytes();
        let mut received = [0; 4];
        self.hardware.exchange_fpga_data(&sent, &mut received);
        u32::from_be_bytes(received)
    }

    /// Refreshes the four continuously mirrored registers in deterministic index order.
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

    /// Routes one byte to the selected serial endpoint while preserving internal-link pacing.
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

    /// Routes a complete slice without changing per-byte newline or delay semantics.
    pub fn route_serial_bytes(&mut self, bytes: &[u8])
    {
        for &byte in bytes
        {
            self.route_serial_byte(byte);
        }
    }

    /// Sends the ACK byte only when a core serial peer exists to receive it.
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

    /// Reports whether the foreground loop has an interrupt-supplied byte to consume.
    pub fn core_serial_pending(&self) -> bool
    {
        self.receive_len != 0
    }

    /// Removes the oldest core byte without blocking when the ring is empty.
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

    /// Returns how often ISR input was dropped to protect unread buffered bytes.
    pub fn dropped_receive_bytes(&self) -> u32
    {
        self.dropped_receive_bytes
    }

    /// Streams configuration bytes unchanged because bitstream byte order is significant.
    pub fn shift_configuration(&mut self, bytes: &[u8])
    {
        for &byte in bytes
        {
            self.hardware.shift_configuration_byte(byte);
        }
    }
}
