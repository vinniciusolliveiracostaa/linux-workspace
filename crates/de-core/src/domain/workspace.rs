use crate::domain::window::{Window, WindowId};
use std::collections::HashMap;

/// ID único de um workspace (newtype para type safety)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(pub u32);

/// Workspace (área de trabalho virtual) que gerencia um conjunto de janelas
pub struct Workspace {
    id: WorkspaceId,

    /// Mapa de janelas por ID (acesso rápido O(1))
    windows: HashMap<WindowId, Window>,

    /// Stack de janelas (ordem Z, do fundo para o topo)
    window_stack: Vec<WindowId>,

    /// ID da janela focada (se houver)
    focused_window: Option<WindowId>,
}

impl Workspace {
    /// Cria um novo workspace vazio
    pub fn new(id: WorkspaceId) -> Self {
        Self {
            id,
            windows: HashMap::new(),
            window_stack: Vec::new(),
            focused_window: None,
        }
    }

    /// Retorna o ID do workspace
    pub fn id(&self) -> WorkspaceId {
        self.id
    }

    /// Adiciona uma janela ao workspace
    ///
    /// A janela é adicionada no topo do stack (visível)
    pub fn add_window(&mut self, window: Window) {
        let window_id = window.id;

        // Adiciona ao HashMap
        self.windows.insert(window_id, window);

        // Adiciona ao topo do stack (Última posição = topo)
        self.window_stack.push(window_id);
    }

    /// Remove uma janela do workspace
    ///
    /// Retorna a janela removida, ou None se não existir
    pub fn remove_window(&mut self, window_id: WindowId) -> Option<Window> {
        // Remove do HashMap
        let window = self.windows.remove(&window_id)?;

        // Remove do stack (mantem ordem dos outros)
        self.window_stack.retain(|&id| id != window_id);

        // Se era a janela focada, remove o foco
        if self.focused_window == Some(window_id) {
            self.focused_window = None;
        }

        Some(window)
    }

    /// Foca uma janela (apenas uma pode estar focada)
    ///
    /// Remove o foco de todas as outras janelas
    pub fn focus_window(&mut self, window_id: WindowId) -> bool {
        // Verifica se a janela existe
        if !self.windows.contains_key(&window_id) {
            return false;
        }

        // Remove foco de todas as janelas
        for window in self.windows.values_mut() {
            window.is_focused = false;
        }

        // Foca a janela desejada
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.is_focused = true;
            self.focused_window = Some(window_id);
            return true;
        }

        false
    }

    /// Retorna o ID da janela focada (se houver)
    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    /// Move uma janela para o topo do stack (torna visível)
    pub fn raise_window(&mut self, window_id: WindowId) -> bool {
        // Verifica se a janela existe
        if !self.windows.contains_key(&window_id) {
            return false;
        }

        // Remove do stack
        self.window_stack.retain(|&id| id != window_id);

        // Adiciona no topo (final do Vec)
        self.window_stack.push(window_id);

        true
    }

    /// Move uma janela para o fundo do stack (fica atrás de todas)
    pub fn lower_window(&mut self, window_id: WindowId) -> bool {
        // Verifica se a janela existe
        if !self.windows.contains_key(&window_id) {
            return false;
        }

        // Remove do stack
        self.window_stack.retain(|&id| id != window_id);

        // Adiciona no fundo (início do Vec)
        self.window_stack.insert(0, window_id);

        true
    }

    /// Retorna janelas em ordem de stack (fundo para topo)
    ///
    /// Útil para renderização (renderizar do fundo para o topo)
    pub fn windows_in_stack_order(&self) -> impl Iterator<Item = &Window> {
        self.window_stack
            .iter()
            .filter_map(|&id| self.windows.get(&id))
    }

    /// Retorna todas as janelas (sem ordem garantida)
    pub fn windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.values()
    }

    /// Retorna número de janelas no workspace
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }
}
