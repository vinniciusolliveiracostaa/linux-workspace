//! IPC Client — Conecta ao IPC Bus e troca mensagens
//!
//! # Arquitetura
//! - Conecta ao servidor via Unix Domain Socket
//! - Envia ComponentType no handshake
//! - Spawna task de leitura (recebe mensagens do servidor)
//! - Expõe canal mpsc para componente receber mensagens
//!
//! # Uso
//! ```no_run
//! let (client, mut rx) = IpcClient::connect(
//!     PathBuf::from("/tmp/de-ipc.sock"),
//!     ComponentType::WindowManager
//! ).await?;
//!
//! // Enviar mensagem
//! client.send(Message::WindowCreated { ... }).await?;
//!
//! // Receber mensagens
//! while let Some(msg) = rx.recv().await {
//!     match msg {
//!         Message::ConfigChanged { key, value } => { ... }
//!         _ => {}
//!     }
//! }
//! ```

use crate::heartbeat::HEARTBEAT_INTERVAL;
use crate::protocol::{ComponentType, Message};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::OwnedWriteHalf;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Erro do IPC Client
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Connection closed")]
    ConnectionClosed,
}

pub struct IpcClient {
    write_half: tokio::sync::Mutex<OwnedWriteHalf>,
    component: ComponentType,
    next_request_id: AtomicU64,
}

impl IpcClient {
    pub async fn connect(
        socket_path: &Path,
        component: ComponentType,
    ) -> Result<(Self, mpsc::Receiver<Message>), ClientError> {
        let mut stream = UnixStream::connect(socket_path).await?;
        info!(component = ?component, "Connected to IPC Bus");

        // Handshake: enviar ComponentType com formato [4 bytes len][payload]
        // POR QUE esse formato? Para ser consistente com o protocolo de mensagens
        let component_bytes = bincode::serialize(&component)?;
        let len = component_bytes.len() as u32;

        // Enviar tamanho (4 bytes little-endian)
        stream.write_all(&len.to_le_bytes()).await?;

        // Enviar payload
        stream.write_all(&component_bytes).await?;

        let (read_half, write_half) = stream.into_split();

        let (tx, rx) = mpsc::channel::<Message>(100);

        tokio::spawn(async move {
            Self::read_loop(read_half, tx, component).await;
        });

        let client = Self {
            write_half: tokio::sync::Mutex::new(write_half),
            component,
            next_request_id: AtomicU64::new(1),
        };

        Ok((client, rx))
    }

    async fn read_loop(
        mut read_half: tokio::net::unix::OwnedReadHalf,
        tx: mpsc::Sender<Message>,
        component: ComponentType,
    ) {
        loop {
            let msg = match Self::read_message(&mut read_half).await {
                Ok(msg) => msg,
                Err(e) => {
                    error!(component = ?component, error = %e, "Read loop error, disconnecting");
                    break;
                }
            };

            if tx.send(msg).await.is_err() {
                info!(component = ?component, "Receiver dropped, stopping read loop");
                break;
            }
        }
        info!(component = ?component, "Read loop ended");
    }

    pub async fn send(&self, msg: Message) -> Result<(), ClientError> {
        let mut write = self.write_half.lock().await;
        Self::write_message(&mut write, &msg).await
    }

    pub async fn request(&self, request: crate::protocol::Request) -> Result<u64, ClientError> {
        let id = self.next_request_id.fetch_add(1, Ordering::Relaxed);

        let msg = Message::Request { id, request };
        self.send(msg).await?;

        Ok(id)
    }

    pub fn component(&self) -> ComponentType {
        self.component
    }

    pub fn start_heartbeat(self: &std::sync::Arc<Self>) -> tokio::task::JoinHandle<()> {
        let client = std::sync::Arc::clone(self);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);

            loop {
                interval.tick().await;

                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                let msg = Message::Heartbeat {
                    component: client.component,
                    timestamp,
                };

                if let Err(e) = client.send(msg).await {
                    error!(
                        component = ?client.component,
                        error = %e,
                        "Failed to send heartbeat, stopping"
                    );
                    break;
                }
            }
        })
    }

    async fn read_message(
        read_half: &mut tokio::net::unix::OwnedReadHalf,
    ) -> Result<Message, ClientError> {
        let mut len_buf = [0u8; 4];
        read_half.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut payload = vec![0u8; len];
        read_half.read_exact(&mut payload).await?;

        bincode::deserialize(&payload).map_err(|e| e.into())
    }

    async fn write_message(
        write_half: &mut OwnedWriteHalf,
        msg: &Message,
    ) -> Result<(), ClientError> {
        let payload = bincode::serialize(msg)?;
        let len = payload.len() as u32;

        write_half.write_all(&len.to_le_bytes()).await?;
        write_half.write_all(&payload).await?;

        Ok(())
    }
}
