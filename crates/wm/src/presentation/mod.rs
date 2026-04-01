//! Presentation Layer — Event handlers e adapters X11
//!
//! Esta camada contém:
//! - X11Adapter: traduz operações de domain para chamadas X11
//! - EventDispatcher: recebe eventos X11 e despacha para services

pub mod event_dispatcher;
pub mod x11_adapter;

pub use event_dispatcher::EventDispatcher;
pub use x11_adapter::X11Adapter;
