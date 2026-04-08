//! de-protocol - Protocol Layer
//! 
//! Define todas as mensagens trocadas via IPC entre componentes.
//! Cada componente (WM, Compositor, Panel, etc) comunica-se exclusivamente
//! através destas mensagens.
//! 
//! ## Princípios:
//! - Mensagens são serializáveis (bincode/serde)
//! - Request/Response pattern para operações síncronas
//! - Event pattern para notificações assíncronas
//! - Versionado para compatibilidade futura

mod messages;
mod events;

pub use messages::*;
pub use events::*;
