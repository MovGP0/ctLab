//! Names the command bytes accepted by the YAMPP Industrial III decoder.

/// Selects one fixed YAMPP Industrial III player operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Yi3Command {
    /// Stops playback while retaining decoder power.
    Stop = 0xF0,

    /// Disables decoder-managed track looping.
    NoLoop = 0xF1,

    /// Enables decoder-managed track looping.
    Loop = 0xF4,

    /// Resets the decoder controller.
    Reset = 0xF7,

    /// Pauses the active track.
    Pause = 0xF8,

    /// Selects the reference volume to which board calibration is added.
    MidVolume = 0xA8,

    /// Mutes audio before a quiet shutdown.
    Mute = 0x80,
}

impl Yi3Command {
    /// Returns the byte transmitted over the bit-banged auxiliary UART.
    pub const fn byte(self) -> u8 {
        self as u8
    }
}
