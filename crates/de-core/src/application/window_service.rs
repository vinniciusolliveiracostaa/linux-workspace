//! Window Service — Domain Service para operações de janelas
//!
//! Este service contém lógica de negócio que não pertence a uma entidade específica.
//! Por exemplo: placement strategies, validações complexas, orquestração entre entidades.

use crate::domain::{Window, WindowId, Workspace, WorkspaceId};
use crate::error::WindowError;
use std::collections::HashMap;

/// Service de gerenciamento de janelas (Application Layer)
///
/// # Responsabilidades
/// - Orquestrar operações entre Window e Workspace
/// - Validar regras de negócio complexas
/// - Manter invariantes (ex: window_to_workspace sempre sincronizado)
///
/// # NÃO faz
/// - Chamadas a X11 (isso é responsabilidade de adapters)
/// - Conversão de eventos (isso é responsabilidade de presentation layer)
pub struct WindowService {
    /// Workspaces virtuais
    workspaces: Vec<Workspace>,

    /// Workspace atualmente ativo
    current_workspace: WorkspaceId,

    /// Mapeamento: qual janela está em qual workspace (invariante)
    window_to_workspace: HashMap<WindowId, WorkspaceId>,
}

impl WindowService {
    /// Cria um novo WindowService
    ///
    /// # Argumentos
    /// - `workspace_count`: Número de workspaces a criar
    pub fn new(workspace_count: u32) -> Result<Self, WindowError> {
        if workspace_count == 0 {
            return Err(WindowError::InvalidOperation(
                "workspace_count must be at least 1".to_string(),
            ));
        }

        let workspaces: Vec<Workspace> = (0..workspace_count)
            .map(|i| Workspace::new(WorkspaceId(i)))
            .collect();

        Ok(Self {
            workspaces,
            current_workspace: WorkspaceId(0),
            window_to_workspace: HashMap::new(),
        })
    }

    /// Adiciona janela ao workspace atual
    ///
    /// # Argumentos
    /// - `window`: Window entity já criada
    ///
    /// # Retorna
    /// - `Ok(())`: Janela adicionada com sucesso
    /// - `Err(WindowError)`: Workspace atual não encontrado
    pub fn add_window(&mut self, window: Window) -> Result<(), WindowError> {
        let window_id = window.id;

        // Adicionar ao workspace atual
        let workspace = self.get_current_workspace_mut()?;
        workspace.add_window(window);

        // Registrar mapeamento
        self.window_to_workspace
            .insert(window_id, self.current_workspace);

        Ok(())
    }

    /// Remove janela do gerenciamento
    ///
    /// # Argumentos
    /// - `window_id`: ID da janela a remover
    ///
    /// # Retorna
    /// - `Ok(())`: Janela removida com sucesso
    /// - `Err(WindowError)`: Janela não estava gerenciada
    pub fn remove_window(&mut self, window_id: WindowId) -> Result<(), WindowError> {
        // Encontrar workspace da janela
        let workspace_id = self
            .window_to_workspace
            .remove(&window_id)
            .ok_or(WindowError::NotFound(window_id))?;

        // Remover do workspace
        let workspace = self.find_workspace_mut(workspace_id)?;
        workspace.remove_window(window_id);

        Ok(())
    }

    /// Retorna todos os IDs de janelas gerenciadas
    pub fn all_window_ids(&self) -> Vec<WindowId> {
        self.workspaces
            .iter()
            .flat_map(|ws| ws.windows())
            .map(|w| w.id)
            .collect()
    }

    /// Retorna workspace atual
    pub fn current_workspace(&self) -> WorkspaceId {
        self.current_workspace
    }

    /// Retorna referência ao workspace atual
    pub fn get_current_workspace(&self) -> Result<&Workspace, WindowError> {
        self.workspaces
            .iter()
            .find(|ws| ws.id() == self.current_workspace)
            .ok_or(WindowError::WorkspaceNotFound(self.current_workspace))
    }

    /// Retorna referência mutável ao workspace atual
    fn get_current_workspace_mut(&mut self) -> Result<&mut Workspace, WindowError> {
        self.workspaces
            .iter_mut()
            .find(|ws| ws.id() == self.current_workspace)
            .ok_or(WindowError::WorkspaceNotFound(self.current_workspace))
    }

    /// Busca workspace por ID
    fn find_workspace_mut(
        &mut self,
        workspace_id: WorkspaceId,
    ) -> Result<&mut Workspace, WindowError> {
        self.workspaces
            .iter_mut()
            .find(|ws| ws.id() == workspace_id)
            .ok_or(WindowError::WorkspaceNotFound(workspace_id))
    }

    /// Retorna workspace que contém a janela
    pub fn find_workspace_for_window(&self, window_id: WindowId) -> Option<WorkspaceId> {
        self.window_to_workspace.get(&window_id).copied()
    }

    /// Retorna referência mutável ao workspace que contém a janela
    pub fn get_workspace_mut_for_window(
        &mut self,
        window_id: WindowId,
    ) -> Result<&mut Workspace, WindowError> {
        let workspace_id = self
            .window_to_workspace
            .get(&window_id)
            .copied()
            .ok_or(WindowError::NotFound(window_id))?;

        self.find_workspace_mut(workspace_id)
    }

    /// Retorna referência mutável a uma janela específica
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Result<&mut Window, WindowError> {
        let workspace_id = self
            .window_to_workspace
            .get(&window_id)
            .copied()
            .ok_or(WindowError::NotFound(window_id))?;

        let workspace = self
            .workspaces
            .iter_mut()
            .find(|ws| ws.id() == workspace_id)
            .ok_or(WindowError::WorkspaceNotFound(workspace_id))?;

        workspace
            .get_window_mut(window_id)
            .ok_or(WindowError::NotFound(window_id))
    }
}
