//! de-domain - Domain Layer
//! 
//! Contém a lógica de negócio pura do Desktop Environment.
//! Zero dependências externas (apenas std lib + euclid para math).
//! 
//! ## Responsabilidades:
//! - Tipos de domínio (WindowId, Geometry, Rect, Size, Position)
//! - Regras de negócio (window management, focus, z-order)
//! - Value objects e entities do DDD
//! 
//! ## NÃO deve:
//! - Depender de X11, Wayland, ou qualquer backend
//! - Conter código de rendering
//! - Conter código de IPC

pub use euclid;

mod geometry;
mod window;
mod types;

pub use geometry::*;
pub use window::*;
pub use types::*;
