//! Renderer abstraction layer for the DE ecosystem.
//! Defines a trait for platform-agnostic drawing operations.

pub mod types;

use macrde_core::{Color, Position, Rectangle, Size};
pub use types::{RenderError, TextStyle};

/// The core rendering interface.
/// Implementors translate these high-level commands into GPU or CPU drawing calls.
pub trait Renderer: Send + Sync {
    /// Initialize the renderer with a surface to draw on.
    /// The surface is typically a window handle provided by the platform (e.g., X11 Window).
    fn init(&mut self, width: u32, height: u32) -> Result<(), RenderError>;

    /// Resize the rendering surface.
    fn resize(&mut self, new_size: Size) -> Result<(), RenderError>;

    /// Clean the entire surface with a single color.
    fn clear(&mut self, color: Color);

    /// Draw a filled rectangle with optional corner radius.
    fn draw_rect(&mut self, rect: Rectangle, color: Color, corner_radius: f32);

    /// Draw a text string at a given position
    fn draw_text(&mut self, text: &str, pos: Position, style: TextStyle)
    -> Result<(), RenderError>;

    /// Present the rendered frame to the screen.
    fn present(&mut self) -> Result<(), RenderError>;
}

pub mod mock;
pub use mock::MockRenderer;
