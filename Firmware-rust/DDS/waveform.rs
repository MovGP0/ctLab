//! Defines the waveform selection shared by DDS firmware, SQG firmware, and hardware control.

/// Waveform routing selected through the serial protocol, EEPROM defaults, or front panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    /// Disables generated output while leaving the controller responsive.
    Off,

    /// Programs the AD9833 sine mode and leaves square and external routing inactive.
    Sine,

    /// Programs the AD9833 triangle mode.
    Triangle,

    /// Routes the comparator-derived square output through the analogue output stage.
    Square,

    /// Routes the logic-level square output instead of the analogue waveform path.
    Logic,

    /// Selects an external source; the payload retains its zero-based route index.
    External(u8),
}

impl Waveform {
    /// Decodes the byte stored in EEPROM or carried by the DDS wire protocol.
    ///
    /// Codes 5 through 249 select external routes. The Pascal byte-overflow
    /// sentinels 250 through 255 fall back to `Off`, keeping malformed settings
    /// from enabling an output unexpectedly.
    pub const fn from_byte(value: u8) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::Sine,
            2 => Self::Triangle,
            3 => Self::Square,
            4 => Self::Logic,
            5..=249 => Self::External(value - 5),
            _ => Self::Off,
        }
    }

    /// Encodes the waveform for EEPROM storage and serial protocol responses.
    pub const fn as_byte(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Sine => 1,
            Self::Triangle => 2,
            Self::Square => 3,
            Self::Logic => 4,
            Self::External(index) => 5u8.saturating_add(index),
        }
    }

    /// Returns the compact waveform label used by the original LCD menu.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Sine => "Sin",
            Self::Triangle => "Tri",
            Self::Square => "Squ",
            Self::Logic => "Lgc",
            Self::External(_) => "Ext",
        }
    }

    /// Decodes an SQG waveform parameter and reproduces its Pascal limit correction.
    ///
    /// SQG accepts only codes 0 through 3. Values 4 through 127 clamp to
    /// `Square`, while byte values above 127 wrap to `Off` before the upper
    /// limit is applied. The boolean reports whether correction was required.
    pub const fn from_sqg_byte(value: u8) -> (Self, bool) {
        match value {
            0 => (Self::Off, false),
            1 => (Self::Sine, false),
            2 => (Self::Triangle, false),
            3 => (Self::Square, false),
            4..=127 => (Self::Square, true),
            _ => (Self::Off, true),
        }
    }

    /// Reports whether SQG accepts the waveform without invoking its range correction.
    pub const fn is_supported_by_sqg(self) -> bool {
        matches!(self, Self::Off | Self::Sine | Self::Triangle | Self::Square)
    }
}
