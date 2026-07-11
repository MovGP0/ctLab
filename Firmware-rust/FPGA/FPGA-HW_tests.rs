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
