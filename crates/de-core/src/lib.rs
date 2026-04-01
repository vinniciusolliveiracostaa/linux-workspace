//! de-core — Domain Layer
//!
//! Este crate contém o núcleo do domínio seguindo Clean Architecture:
//! - domain: Entidades e Value Objects
//! - application: Use Cases e Domain Services
//! - ports: Traits/Interfaces (Dependency Inversion)

pub mod application;
pub mod domain;
pub mod error;
//pub mod ports;

// Re-exports principais
pub use application::WindowService;
pub use domain::{
    Color, Position, Rectangle, Size, Window, WindowId, WindowState, Workspace, WorkspaceId,
};
pub use error::WindowError;
