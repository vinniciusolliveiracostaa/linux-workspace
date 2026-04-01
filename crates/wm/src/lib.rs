//! Window Manager — Gerenciamento de janelas e workspaces
//!
//! Este crate segue Clean Architecture:
//! - application: Services com lógica de negócio
//! - presentation: Event handlers e adapters X11

pub mod application;
pub mod error;
pub mod presentation;

// Re-exports públicos
pub use application::WindowManagerService;
pub use error::WmError;
pub use presentation::{EventDispatcher, X11Adapter};
