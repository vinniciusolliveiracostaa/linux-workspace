use crate::{RenderError, Renderer, TextStyle};
use macrde_core::{Color, Position, Rectangle, Size};
use std::sync::{Arc, Mutex};

/// A mock renderer that does nothing but log calls.
/// Useful for testing the compositor logic without a real window.
#[derive(Clone, Default)]
pub struct MockRenderer {
    // We wrap state in Arc<Mutex> to satisfy Send + Sync and allow interior mutability.
    state: Arc<Mutex<MockState>>,
}

#[derive(Default)]
struct MockState {
    pub last_clear_color: Option<Color>,
    pub draw_calls: Vec<String>,
}

impl Renderer for MockRenderer {
    fn init(
        &mut self,
        _surface: &impl raw_window_handle::HasWindowHandle,
    ) -> Result<(), RenderError> {
        println!("[MockRenderer] Initialized.");
        Ok(())
    }

    fn resize(&mut self, new_size: Size) -> Result<(), RenderError> {
        println!(
            "[MockRenderer] Resized to {}x{}",
            new_size.width(),
            new_size.height()
        );
        Ok(())
    }

    fn clear(&mut self, color: Color) {
        if let Ok(mut state) = self.state.lock() {
            state.last_clear_color = Some(color);
        }
        println!("[MockRenderer] Clear with color {:?}", color);
    }

    fn draw_rect(&mut self, rect: Rectangle, color: Color, corner_radius: f32) {
        if let Ok(mut state) = self.state.lock() {
            state.draw_calls.push(format!(
                "draw_rect({:?}, {:?}, {})",
                rect, color, corner_radius
            ));
        }
    }

    fn draw_text(
        &mut self,
        text: &str,
        pos: Position,
        _style: TextStyle,
    ) -> Result<(), RenderError> {
        println!("[MockRenderer] Draw text '{}' at {:?}", text, pos);
        Ok(())
    }

    fn present(&mut self) -> Result<(), RenderError> {
        println!("[MockRenderer] Present frame.");
        Ok(())
    }
}
