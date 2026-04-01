#[derive(Debug, thiserror::Error)]
pub enum X11Error {
    #[error("Failed to connect to X11 server: {0}")]
    ConnectionFailed(String),

    #[error("X11 protocol error: {0}")]
    ProtocolError(String),

    #[error("Invalid screen number: {0}")]
    InvalidScreen(i32),
}
