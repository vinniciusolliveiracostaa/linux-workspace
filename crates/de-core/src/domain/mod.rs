pub mod color;
pub mod geometry;
pub mod window;

// Re-exports para facilitar o uso
pub use color::Color;
pub use geometry::{Position, Rectangle, Size};
pub use window::{Window, WindowId, WindowState};
