#[derive(Debug, Clone, Default)]
pub struct Mp3ControlState {
    pub track: u8,
    pub current_track: u8,
    pub db_correction: u8,
    pub is_on: bool,
}
