//! Error types para o domain layer

use crate::domain::{Size, WindowId, WorkspaceId};

/// Erros relacionados a operações de janelas
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    /// Janela não encontrada
    #[error("Window {0:?} not found")]
    NotFound(WindowId),

    /// Workspace não encontrado
    #[error("Workspace {0:?} not found")]
    WorkspaceNotFound(WorkspaceId),

    /// Tamanho abaixo do mínimo
    #[error("Window size {requested:?} is below minimum {min:?}")]
    SizeTooSmall { requested: Size, min: Size },

    /// Operação inválida
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
