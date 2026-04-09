//! Identificadores type-safe para as entidades do sistema.
//! Usamos newtypes para evitar confusão entre IDs diferentes.

use serde::{Deserialize, Serialize};

/// Identificador único para uma janela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u32);

/// Identificador único para um workspace (área de trabalho).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceId(pub u32);

/// Identificador único para uma superfície de renderização.
/// Uma janela pode ter múltiplas superfícies (ex: decoração + conteúdo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SurfaceId(pub u64);
