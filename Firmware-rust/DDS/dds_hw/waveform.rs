#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Waveform {
    Off,
    Sine,
    Triangle,
    Square,
    Logic,
    External(u8),
}
