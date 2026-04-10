use anyhow::Result;
use macrde_core::WindowId;
use macrde_ipc::{CompositorCommand, CompositorEvent, IpcError};
use macrde_x11::X11Connection;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

mod events;
mod hotkeys;
mod placement;
mod service;

// Placeholder para o servico principal
struct WindowManager {
    x11: Arc<X11Connection>,
    ipc_stream: UnixStream,
}

impl WindowManager {
    async fn new(x11: Arc<X11Connection>, ipc_stream: UnixStream) -> Self {
        Self { x11, ipc_stream }
    }

    async fn send_command(&mut self, cmd: CompositorCommand) -> Result<(), IpcError> {
        let encoded = bincode::encode_to_vec(&cmd, bincode::config::standard())?;
        // Prefixa com tamanho (u32) como no IPC antigo
        let len = encoded.len() as u32;
        self.ipc_stream.write_all(&len.to_le_bytes()).await?;
        self.ipc_stream.write_all(&encoded).await?;
        self.ipc_stream.flush().await?;

        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        tracing::info!("Window Manager started");
        // Loop principal vir'a aqui (X11 events + IPC receive)
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let x11 = Arc::new(X11Connection::connect()?);

    // Conecta ao socket do composer (ainda nao criado o composer IPC, mas path definido)
    let socket_path = "/tmp/macrde-compositor.sock";
    let ipc_stream = UnixStream::connect(socket_path).await?;

    let mut wm = WindowManager::new(x11, ipc_stream).await;
    wm.run().await?;

    Ok(())
}
