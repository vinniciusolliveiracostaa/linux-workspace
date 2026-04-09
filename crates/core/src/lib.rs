//! Core domain types for the DE ecosystem.
//! This crate has no external dependencies.

pub mod color;
pub mod geometry;
pub mod id;

// Re-exportações para facilitar o uso
pub use color::Color;
pub use geometry::{Position, Rectangle, Size};
pub use id::{SurfaceId, WindowId, WorkspaceId};
