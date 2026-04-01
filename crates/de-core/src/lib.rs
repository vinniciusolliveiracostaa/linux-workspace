// de-core — Domain Layer
// Entidades, Value Objects e Ports (traits) do domínio

pub mod domain;
pub mod error;
pub mod ports;

// Re-exports principais
pub use domain::{
    Color, Position, Rectangle, Size, Window, WindowId, WindowState, Workspace, WorkspaceId,
};
pub use error::WindowError;
