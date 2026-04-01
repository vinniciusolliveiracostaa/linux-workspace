//! Application Layer — Lógica de negócio do Window Manager
//!
//! Esta camada contém:
//! - WindowManagerService: orquestração de operações de janelas
//! - PlacementStrategy: estratégias de posicionamento

pub mod placement_strategy;
pub mod window_manager_service;

pub use placement_strategy::PlacementStrategy;
pub use window_manager_service::WindowManagerService;
