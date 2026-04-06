//! Window Manager Service — Application Layer
//!
//! Orquestra operações de gerenciamento de janelas usando WindowService do de-core
//! e PlacementStrategy local.

use de_core::{Position, Rectangle, Size, Window, WindowError, WindowId, WindowService};

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
    ///
    /// # POR QUE retornar Result?
    /// - workspace_count == 0 é inválido por regra de domínio
    /// - Preferimos falhar cedo (fail-fast) com erro claro
    /// - Evita panic no meio da execução do WM
    ///
    /// # Retorna
    /// - `Ok(Self)` se workspace_count >= 1
    /// - `Err(WindowError)` se workspace_count == 0 (inválido)
    pub fn new(workspace_count: u32) -> Result<Self, WindowError> {
        // O ? propaga o WindowError se workspace_count == 0
        // POR QUE aqui e não no caller?
        // - Este service é quem "conhece" que depende do WindowService
        // - Ele é o ponto de integração (Application Layer)
        Ok(Self {
            window_service: WindowService::new(workspace_count)?, // ← ? aqui!
            placement_strategy: PlacementStrategy::Smart,
        })
    }

    /// Calcula geometria para nova janela e adiciona ao workspace
    ///
    /// # Retorna
    /// - Geometria calculada (para o adapter aplicar no X11)
    pub fn manage_window(
        &mut self,
        window_id: WindowId,
        requested_size: Option<(u32, u32)>,
        screen: Rectangle,
    ) -> Result<Rectangle, WindowError> {
        // Usar tamanho solicitado ou fallback para 800x600
        let window_size = requested_size.unwrap_or((800, 600));

        // Validar tamanho mínimo
        // POR QUE validar aqui?
        // - Application Layer é responsável por validar inputs antes de passar pro Domain
        // - Evita criar Size inválido
        if window_size.0 < 100 || window_size.1 < 50 {
            // Criar Size mínimo usando construtor unchecked (sabemos que 100 e 50 > 0)
            let min_size = Size::new_unchecked(100, 50);

            let requested = Size::new_unchecked(
                window_size.0.max(1),
                window_size.1.max(1),
            );

            return Err(WindowError::SizeTooSmall {
                requested,
                min: min_size,
            });
        }

        // Calcular geometria usando placement strategy
        let workspace = self.window_service.get_current_workspace()?;
        let existing_windows: Vec<&Window> = workspace.windows().collect();
        let geometry = self
            .placement_strategy
            .place(window_size, &screen, &existing_windows);

        // Criar window entity e adicionar ao workspace
        let window = Window::new(window_id, geometry);
        self.window_service.add_window(window)?;

        Ok(geometry)
    }

    /// Remove janela do gerenciamento
    pub fn unmanage_window(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        self.window_service.remove_window(window_id)
    }

    /// Move janela para nova posição
    pub fn move_window(
        &mut self,
        window_id: WindowId,
        new_position: Position,
    ) -> Result<(), WindowError> {
        let workspace = self
            .window_service
            .get_workspace_mut_for_window(window_id)?;

        workspace
            .get_window_mut(window_id)
            .ok_or(WindowError::NotFound(window_id))?
            .move_to(new_position);

        Ok(())
    }

    /// Redimensiona janela
    pub fn resize_window(
        &mut self,
        window_id: WindowId,
        new_size: Size,
    ) -> Result<(), WindowError> {
        let workspace = self
            .window_service
            .get_workspace_mut_for_window(window_id)?;

        let window = workspace
            .get_window_mut(window_id)
            .ok_or(WindowError::NotFound(window_id))?;

        window.resize(new_size)?;

        Ok(())
    }

    /// Foca janela
    pub fn focus_window(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        let workspace = self
            .window_service
            .get_workspace_mut_for_window(window_id)?;

        if !workspace.focus_window(window_id) {
            return Err(WindowError::NotFound(window_id));
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

    /// Retorna a janela focada
    pub fn focused_window(&self) -> Option<WindowId> {
        self.window_service
            .get_current_workspace()
            .ok() // Converter Result → Option
            .and_then(|ws| ws.focused_window())
    }

    /// Retorna geometria de uma janela
    pub fn window_geometry(&self, window_id: WindowId) -> Option<Rectangle> {
        // POR QUE ESTA ABORDAGEM:
        // - get_workspace_mut_for_window() sabemos que existe (você já usa em outros métodos)
        // - Mas precisamos de referência imutável, não mutável
        // - Solução: buscar diretamente via find + get_current_workspace

        // Se a janela está no workspace atual, pegar geometria
        if let Ok(workspace) = self.window_service.get_current_workspace() {
            if let Some(geom) = workspace.window_geometry(window_id) {
                return Some(geom);
            }
        }

        // TODO: Buscar em outros workspaces quando implementarmos multi-workspace
        None
    }

    /// Toggle maximize da janela focada
    pub fn toggle_maximize(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        let workspace = self
            .window_service
            .get_workspace_mut_for_window(window_id)?;

        workspace.toggle_maximize(window_id)
    }
}
