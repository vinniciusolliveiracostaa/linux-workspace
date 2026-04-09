use thiserror::Error;

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Encode error: {0}")]
    BincodeEncode(#[from] bincode::error::EncodeError),
    #[error("Decode error: {0}")]
    BincodeDecode(#[from] bincode::error::DecodeError),
}
