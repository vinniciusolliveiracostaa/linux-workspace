use super::{Position, Rectangle, Size};
use crate::error::WindowError;
use serde::{Deserialize, Serialize};

pub const WINDOW_MIN_WIDTH: u32 = 100;
pub const WINDOW_MIN_HEIGHT: u32 = 50;

/// Identificador único de uma janela no sistema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    Normal,
    Maximized,
    Minimized,
    Fullscreen,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Window {
    pub id: WindowId,
    pub geometry: Rectangle,
    pub state: WindowState,
    pub is_focused: bool,
    pub is_decorated: bool,
}

impl Window {
    pub fn new(id: WindowId, geometry: Rectangle) -> Self {
        Self {
            id,
            geometry,
            state: WindowState::Normal,
            is_focused: false,
            is_decorated: true,
        }
    }

    pub fn move_to(&mut self, position: Position) {
        self.geometry.position = position;
    }

    pub fn resize(&mut self, size: Size) -> Result<(), WindowError> {
        if size.width < WINDOW_MIN_WIDTH || size.height < WINDOW_MIN_HEIGHT {
            return Err(WindowError::SizeTooSmall {
                requested: size,
                min: Size {
                    width: WINDOW_MIN_WIDTH,
                    height: WINDOW_MIN_HEIGHT,
                },
            });
        }

        self.geometry.size = size;
        Ok(())
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }
}
