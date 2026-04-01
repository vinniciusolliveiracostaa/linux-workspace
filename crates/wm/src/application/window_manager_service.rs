//! Window Manager Service — Application Layer
//!
//! Orquestra operações de gerenciamento de janelas usando WindowService do de-core
//! e PlacementStrategy local.

use de_core::{Position, Rectangle, Size, Window, WindowId, WindowService};
use de_x11::X11Error;

use super::PlacementStrategy;

/// Service do Window Manager (Application Layer)
///
/// # Responsabilidades
/// - Orquestrar WindowService (de-core) + PlacementStrategy
/// - Calcular geometria para novas janelas
/// - Validar operações antes de aplicar
///
/// # NÃO faz
/// - Chamadas diretas ao X11 (responsabilidade do X11Adapter)
pub struct WindowManagerService {
    /// Service de janelas (de-core)
    window_service: WindowService,

    /// Estratégia de posicionamento
    placement_strategy: PlacementStrategy,
}

impl WindowManagerService {
    /// Cria um novo WindowManagerService
    pub fn new(workspace_count: u32) -> Self {
        Self {
            window_service: WindowService::new(workspace_count),
            placement_strategy: PlacementStrategy::Smart,
        }
    }

    /// Calcula geometria para nova janela e adiciona ao workspace
    ///
    /// # Retorna
    /// - Geometria calculada (para o adapter aplicar no X11)
    pub fn manage_window(
        &mut self,
        window_id: WindowId,
        screen: Rectangle,
    ) -> Result<Rectangle, X11Error> {
        // 1. Calcular geometria usando placement strategy
        let window_size = (800, 600); // TODO: query do X11
        let workspace = self
            .window_service
            .get_current_workspace()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;
        let existing_windows: Vec<&Window> = workspace.windows().collect();
        let geometry = self
            .placement_strategy
            .place(window_size, &screen, &existing_windows);

        // 2. Criar window entity e adicionar ao workspace
        let window = Window::new(window_id, geometry);
        self.window_service
            .add_window(window)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(geometry)
    }

    /// Remove janela do gerenciamento
    pub fn unmanage_window(&mut self, window_id: WindowId) -> Result<(), X11Error> {
        self.window_service
            .remove_window(window_id)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))
    }

    /// Move janela para nova posição
    pub fn move_window(
        &mut self,
        window_id: WindowId,
        new_position: Position,
    ) -> Result<(), X11Error> {
        let workspace = self
            .window_service
            .get_workspace_mut_for_window(window_id)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        workspace
            .get_window_mut(window_id)
            .ok_or_else(|| {
                X11Error::ProtocolError(format!("Window {:?} not in workspace", window_id))
            })?
            .move_to(new_position);

        Ok(())
    }

    /// Redimensiona janela
    pub fn resize_window(&mut self, window_id: WindowId, new_size: Size) -> Result<(), X11Error> {
        let workspace = self
            .window_service
            .get_workspace_mut_for_window(window_id)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        let window = workspace.get_window_mut(window_id).ok_or_else(|| {
            X11Error::ProtocolError(format!("Window {:?} not in workspace", window_id))
        })?;

        window
            .resize(new_size)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    /// Foca janela
    pub fn focus_window(&mut self, window_id: WindowId) -> Result<(), X11Error> {
        let workspace = self
            .window_service
            .get_workspace_mut_for_window(window_id)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        if !workspace.focus_window(window_id) {
            return Err(X11Error::ProtocolError(format!(
                "Window {:?} not in workspace",
                window_id
            )));
        }

        workspace.raise_window(window_id);

        Ok(())
    }

    /// Retorna todos os IDs de janelas gerenciadas
    pub fn all_window_ids(&self) -> Vec<WindowId> {
        self.window_service.all_window_ids()
    }

    /// Verifica se janela está gerenciada
    pub fn is_managed(&self, window_id: WindowId) -> bool {
        self.window_service
            .find_workspace_for_window(window_id)
            .is_some()
    }
}
