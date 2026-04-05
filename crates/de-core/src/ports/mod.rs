//! Ports (Interfaces) — Dependency Inversion Principle
//!
//! Esta camada define traits que serão implementados pela Infrastructure layer.
//! Exemplos: WindowSystem, Renderer, ConfigLoader, etc.
//!
//! # Regra de Ouro
//! - Domain define a interface (trait)
//! - Infrastructure implementa a interface
//! - Application usa a interface (não a implementação concreta)

// TODO: Adicionar traits quando começar Slice 1
// Exemplo futuro:
// pub trait WindowSystem: Send + Sync {
//     async fn map_window(&self, id: WindowId) -> Result<(), WindowSystemError>;
//     async fn unmap_window(&self, id: WindowId) -> Result<(), WindowSystemError>;
// }
