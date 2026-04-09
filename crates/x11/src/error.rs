use thiserror::Error;

#[derive(Debug, Error)]
pub enum X11Error {
    #[error("Failed to connect to X server")]
    ConnectionFailed(#[from] xcb::ConnError),
    #[error("X11 protocol error: {0:?}")]
    Protocol(#[from] xcb::ProtocolError),
    #[error("Generic XCB error: {0}")]
    Generic(#[from] xcb::Error),
    #[error("Missing expected X11 resource")]
    ResourceMissing,
}
