//! Stores decoder power, track, and volume shadows needed to order YAMPP commands.

/// MP3 control shadow state used to serialize power, track, and volume changes in the order expected by the decoder board.
#[derive(Debug, Clone, Default)]
pub struct Mp3ControlState {
    /// Tracks `track` across MP3 commands because power, track, and volume updates must be resent in decoder-specific order.
    pub track: u8,

    /// Tracks `current_track` across MP3 commands because power, track, and volume updates must be resent in decoder-specific order.
    pub current_track: u8,

    /// Tracks `db_correction` across MP3 commands because power, track, and volume updates must be resent in decoder-specific order.
    pub db_correction: u8,

    /// Tracks `is_on` across MP3 commands because power, track, and volume updates must be resent in decoder-specific order.
    pub is_on: bool,
}
