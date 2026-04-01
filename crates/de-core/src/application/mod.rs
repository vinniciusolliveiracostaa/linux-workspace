//! Application Layer — Use Cases e Domain Services
//!
//! Esta camada contém:
//! - Use Cases: orquestração de operações de domínio
//! - Domain Services: lógica de negócio que não pertence a uma entidade específica
//!
//! # Regras
//! - Pode depender de: domain, ports
//! - NÃO pode depender de: infrastructure (de-x11, de-render, etc.)

pub mod window_service;

pub use window_service::WindowService;
