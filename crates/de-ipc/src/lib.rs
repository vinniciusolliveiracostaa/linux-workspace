//! de-ipc — IPC Bus Infrastructure
//!
//! Sistema de comunicação inter-processos usando Unix Domain Sockets
//!
//! # Arquitetura
//! - `protocol`: Definição de mensagens (Message, Request, Response)
//! - `server`: IPC Bus (gerencia conexões de clientes)
//! - `client`: IPC Client (conecta ao bus, envia/recebe mensagens)
//! - `heartbeat`: Detecção de componentes mortos

pub mod client;
pub mod heartbeat;
pub mod protocol;
pub mod server;

// Re-exports principais
pub use client::{ClientError, IpcClient};
pub use protocol::{
    CloseReason, ComponentType, Message, NotificationInfo, Request, Response, Urgency, WindowInfo,
    WorkspaceInfo,
};
pub use server::{IpcBus, IpcError};
