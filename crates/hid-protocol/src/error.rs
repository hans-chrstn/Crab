use thiserror::Error;

#[derive(Error, Debug)]
pub enum HidError {
    #[error("packet too short: expected {expected} bytes, got {got} bytes")]
    PacketTooShort { expected: usize, got: usize },
    #[error("packet too long: expected {expected} bytes, got {got} bytes")]
    PacketTooLong { expected: usize, got: usize },
    #[error("unknown header byte: {0:#04x}")]
    InvalidHeader(u8),
    #[error("io error occured")]
    ReadError(#[from] std::io::Error),
}
