mod events;
mod hotkeys;
mod placement;
mod service;

use anyhow::Result;
use macrde_ipc::{CompositorCommand, CompositorEvent, IpcError};
use macrde_x11::X11Connection;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::service::WindowManagerService;

// Placeholder para o servico principal
// main.rs
pub(crate) struct WindowManager {
    pub(crate) x11: Arc<X11Connection>,
    pub(crate) service: WindowManagerService,
    pub(crate) ipc_stream: UnixStream,
}

impl WindowManager {
    async fn new(x11: Arc<X11Connection>, ipc_stream: UnixStream) -> Self {
        Self {
            x11,
            service: WindowManagerService::new(),
            ipc_stream,
        }
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

    async fn receive_ipc_event(&mut self) -> Result<CompositorEvent> {
        let mut len_buf = [0u8; 4];
        self.ipc_stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        self.ipc_stream.read_exact(&mut buf).await?;
        let (event, _): (CompositorEvent, _) =
            bincode::decode_from_slice(&buf, bincode::config::standard())?;
        Ok(event)
    }

    async fn handle_compositor_event(&mut self, event: CompositorEvent) -> Result<()> {
        match event {
            CompositorEvent::WindowClosed { window_id } => {
                self.service.remove_window(window_id);
                // Opcional: enviar evento de destruição ao X11
            }
            CompositorEvent::WindowResized {
                window_id: _,
                new_geometry: _,
            } => {
                // Atualiza a geometria no service
            }
        }
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                x11_res = tokio::task::spawn_blocking({
                    let x11 = self.x11.clone();
                    move || x11.wait_for_event()
                }) => {
                    match x11_res {
                        Ok(Ok(event)) => { self.handle_x11_event(event).await?; }
                        Ok(Err(e)) => tracing::error!("X11 event error: {}", e),
                        Err(e) => tracing::error!("spawn_blocking error: {}", e),
                    }
                }
                ipc_res = self.receive_ipc_event() => {
                    match ipc_res {
                        Ok(event) => self.handle_compositor_event(event).await?,
                        Err(e) => tracing::warn!("IPC receive error: {}", e),
                    }
                }
            }
        }
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
