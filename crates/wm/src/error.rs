use de_core::WindowError;
use de_ipc::IpcError;
use de_x11::X11Error;

#[derive(Debug, thiserror::Error)]
pub enum WmError {
    #[error("Window error: {0}")]
    Window(#[from] WindowError),

    #[error("X11 error: {0}")]
    X11(#[from] X11Error),

    #[error("IPC error: {0}")]
    Ipc(#[from] IpcError),
}
