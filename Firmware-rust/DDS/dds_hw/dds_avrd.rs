//! Maps DDS serial, relay, and converter operations to AVR registers and pins.

use super::*;

/// AVR implementation of the cycle-level DDS interface, including interrupt masking around contiguous serial frames.
pub struct DdsAvrd<M: Mcu> {
    /// Owns the hardware adapter, ensuring all side effects are routed through one testable boundary.
    pub(super) io: AvrdPortIo<M>,

    /// Caches `saved_status` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub(super) saved_status: u8,

    /// Caches `_marker` across bit-level or interrupt phases, allowing the next phase to continue without recomputing or observing a half-written hardware transaction.
    pub(super) _marker: PhantomData<M>,
}
impl<M: Mcu> Default for DdsAvrd<M> {
    /// Creates the zero-sized AVR DDS adapter without touching registers; port direction setup remains explicit.
    fn default() -> Self {
        Self {
            io: AvrdPortIo::default(),
            saved_status: 0,
            _marker: PhantomData,
        }
    }
}
impl<M: Mcu> DdsAvrd<M> {
    /// Initializes ports before dependent calculations or outputs are enabled.
    pub fn init_ports(&mut self) {
        self.io.init_port(RegisterPort::B, 0b0001_1111, 0b0001_0111);
        self.io.init_port(RegisterPort::C, 0b1111_1100, 0b1111_1111);
        self.io.init_port(RegisterPort::D, 0b0000_1100, 0b1111_1100);
    }

    /// Maps a logical DDS port group to its AVR `PORT` and `DDR` addresses so shift code remains board-layout agnostic.
    fn map_port(port: PortKind) -> RegisterPort {
        match port {
            PortKind::DdsOut | PortKind::ControlBit => RegisterPort::B,
            PortKind::Extension => RegisterPort::C,
            PortKind::LedOut => RegisterPort::D,
        }
    }
}
impl<M: Mcu> DdsHardwareIo for DdsAvrd<M> {
    /// Raises one logical output bit without exposing target register addresses to the shift routines.
    fn set_bit(&mut self, port: PortKind, bit: u8) {
        self.io.write_bit(Self::map_port(port), bit, true);
    }

    /// Lowers one logical output bit without exposing target register addresses to the shift routines.
    fn clear_bit(&mut self, port: PortKind, bit: u8) {
        self.io.write_bit(Self::map_port(port), bit, false);
    }

    /// Emits one AVR no-operation cycle used to satisfy converter setup and hold times without touching registers.
    fn nop(&mut self) {
        self.io.spin_delay_cycles(1);
    }

    /// Expands a board timing unit into deterministic no-operation cycles between serial edges.
    fn delay_units(&mut self, units: u8) {
        self.io
            .spin_delay_cycles(u16::from(units) * SER_AUX_DELAY_CYCLES_PER_UNIT);
    }

    /// Provides board-timed settling while keeping the delay mechanism replaceable in host tests.
    fn delay_ms(&mut self, milliseconds: u16) {
        for _ in 0..milliseconds {
            self.io.spin_delay_cycles(16_000);
        }
    }

    /// Saves interrupt state and excludes timer service until the complete DDS or DAC frame has been clocked.
    fn begin_critical_section(&mut self) {
        #[cfg(target_arch = "avr")]
        {
            self.saved_status = unsafe { core::ptr::read_volatile(AVR_SREG_ADDRESS) };
            unsafe {
                core::ptr::write_volatile(
                    AVR_SREG_ADDRESS,
                    self.saved_status & !AVR_SREG_INTERRUPT_ENABLE_MASK,
                );
            }
        }
    }

    /// Restores the interrupt state captured before the contiguous DDS or DAC frame.
    fn end_critical_section(&mut self) {
        #[cfg(target_arch = "avr")]
        unsafe {
            core::ptr::write_volatile(AVR_SREG_ADDRESS, self.saved_status);
        }

        #[cfg(not(target_arch = "avr"))]
        {
            let _ = self.saved_status;
        }
    }
}
