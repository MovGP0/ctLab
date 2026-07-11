#[derive(Debug, Clone, Copy, PartialEq, Eq)]

/// Logical names for the shared Port-B serial and latch lines.
pub enum ControlBit {
    /// Bit clock used by every external ADC/DAC serial transaction.
    Sclk,

    /// Controller-to-converter serial data output.
    SDataOut,

    /// DAC or shift-register latch/strobe line.
    StrDac,

    /// Active-low conversion/latch line for the LTC1864 ADC.
    StrAd16,

    /// Converter-to-controller serial data input.
    SDataIn1,
}
