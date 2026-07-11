/// Value returned by command dispatch before serial framing is applied.

///
/// Separating protocol values from transport formatting keeps calculations and
/// register access testable without reproducing UART side effects.
#[derive(Debug, Clone, PartialEq)]
pub enum Response
{
    /// A setter, ignored channel, or unsupported getter has no payload.
    None,

    /// A calculator register or other floating-point protocol value.
    Number(f64),

    /// A counter, length, or raw FPGA register value.
    Integer(i64),

    /// Version and filename data that must remain textual.
    Text(String),
}
