//! Window types - Entidades do domínio de gerenciamento de janelas

use crate::geometry::{Position, Rect, Size};

/// Identificador único de janela (wrapper type-safe)
/// 
/// Pode representar tanto uma janela X11 quanto Wayland
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(pub u64);

impl WindowId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u64> for WindowId {
    fn from(id: u64) -> Self {
        Self::new(id)
    }
}

impl From<u32> for WindowId {
    fn from(id: u32) -> Self {
        Self::new(id as u64)
    }
}

/// Estado de foco da janela
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusState {
    #[default]
    Unfocused,
    Focused,
    Active, // Focused + recebendo input
}

/// Estado de maximização
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaximizeState {
    #[default]
    None,
    Horizontal,
    Vertical,
    Both,
}

/// Estado de uma janela gerenciada
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowState {
    #[default]
    Normal,
    Maximized(MaximizeState),
    Minimized,
    Fullscreen,
}

/// Propriedades de uma janela (value object)
#[derive(Debug, Clone, PartialEq)]
pub struct WindowProperties {
    pub title: String,
    pub app_id: Option<String>,
    pub pid: Option<u32>,
    pub decorated: bool,
    pub resizable: bool,
    pub closable: bool,
    pub minimizable: bool,
    pub maximizable: bool,
}

impl Default for WindowProperties {
    fn default() -> Self {
        Self {
            title: String::new(),
            app_id: None,
            pid: None,
            decorated: true,
            resizable: true,
            closable: true,
            minimizable: true,
            maximizable: true,
        }
    }
}

/// Entidade Window - representa uma janela no sistema
#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub geometry: Rect,
    pub state: WindowState,
    pub focus_state: FocusState,
    pub z_order: i32,
    pub properties: WindowProperties,
    pub parent: Option<WindowId>,
    pub children: Vec<WindowId>,
}

impl Window {
    /// Cria uma nova janela
    pub fn new(id: WindowId, geometry: Rect) -> Self {
        Self {
            id,
            geometry,
            state: WindowState::Normal,
            focus_state: FocusState::Unfocused,
            z_order: 0,
            properties: WindowProperties::default(),
            parent: None,
            children: Vec::new(),
        }
    }
    
    /// Verifica se a janela está visível
    pub fn is_visible(&self) -> bool {
        self.state != WindowState::Minimized
    }
    
    /// Verifica se a janela pode receber foco
    pub fn can_focus(&self) -> bool {
        self.is_visible() && self.properties.closable
    }
    
    /// Verifica se um ponto está dentro da janela (considerando geometria)
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.geometry.contains(x, y)
    }
    
    /// Move a janela para uma nova posição
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.geometry = Rect::new(x, y, self.geometry.width(), self.geometry.height());
    }
    
    /// Redimensiona a janela
    pub fn resize(&mut self, width: i32, height: i32) {
        self.geometry = Rect::new(self.geometry.x(), self.geometry.y(), width, height);
    }
    
    /// Define o estado de foco
    pub fn set_focus(&mut self, focused: bool) {
        self.focus_state = if focused {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        };
    }
    
    /// Define o estado de maximização
    pub fn set_maximized(&mut self, state: MaximizeState) {
        self.state = match state {
            MaximizeState::None => WindowState::Normal,
            _ => WindowState::Maximized(state),
        };
    }
    
    /// Alterna estado de maximização
    pub fn toggle_maximize(&mut self) {
        self.state = match self.state {
            WindowState::Maximized(_) => WindowState::Normal,
            _ => WindowState::Maximized(MaximizeState::Both),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_window_creation() {
        let win = Window::new(WindowId::new(1), Rect::new(0, 0, 100, 100));
        assert_eq!(win.id.as_u64(), 1);
        assert_eq!(win.geometry.width(), 100);
        assert_eq!(win.state, WindowState::Normal);
    }
    
    #[test]
    fn test_window_move() {
        let mut win = Window::new(WindowId::new(1), Rect::new(0, 0, 100, 100));
        win.move_to(50, 60);
        assert_eq!(win.geometry.x(), 50);
        assert_eq!(win.geometry.y(), 60);
        assert_eq!(win.geometry.width(), 100); // largura não muda
    }
    
    #[test]
    fn test_window_resize() {
        let mut win = Window::new(WindowId::new(1), Rect::new(0, 0, 100, 100));
        win.resize(200, 150);
        assert_eq!(win.geometry.width(), 200);
        assert_eq!(win.geometry.height(), 150);
        assert_eq!(win.geometry.x(), 0); // posição não muda
    }
    
    #[test]
    fn test_window_contains_point() {
        let win = Window::new(WindowId::new(1), Rect::new(10, 10, 100, 100));
        assert!(win.contains_point(50, 50));
        assert!(!win.contains_point(5, 5));
    }
    
    #[test]
    fn test_window_toggle_maximize() {
        let mut win = Window::new(WindowId::new(1), Rect::new(0, 0, 100, 100));
        assert_eq!(win.state, WindowState::Normal);
        
        win.toggle_maximize();
        assert_eq!(win.state, WindowState::Maximized(MaximizeState::Both));
        
        win.toggle_maximize();
        assert_eq!(win.state, WindowState::Normal);
    }
}
