//! IPC message protocol for the DE ecosystem.
//! Defines commands and events exchanged between WM and Compositor.

use macrde_core::{Rectangle, WindowId};
use serde::{Deserialize, Serialize};

pub mod error;
pub use error::IpcError;

/// Comandos envidados do Windows Manager para o Compositor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositorCommand {
    /// Adiciona uma nova janela ao gerenciamento visual.
    AddWindow {
        window_id: WindowId,
        /// Geometria inicial da janela.
        geometry: Rectangle,
    },
    /// Remove uma janela.
    RemoveWindow { window_id: WindowId },
    /// Move uma janela para uma nova posição.
    MoveWindow {
        window_id: WindowId,
        /// Nova geometria da janela.
        geometry: Rectangle,
    },
    /// Solicita que o compositor foque uma janela.
    FocusWindow { window_id: WindowId },
    // Adicionaremos comandos de redimensionamento mais tarde.
}

/// Eventos enviados do Compositor para o Window Manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositorEvent {
    /// O usuário fechou a janela (ex: clicou no botão X).
    WindowClosed { window_id: WindowId },
    /// O usuário redimensionou a janela pelas bordas.
    WindowResized {
        window_id: WindowId,
        new_geometry: Rectangle,
    },
}
