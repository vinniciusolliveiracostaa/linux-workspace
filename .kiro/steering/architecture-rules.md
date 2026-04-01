---
inclusion: always
---

# Regras de Arquitetura — Rust DE/WM

## ⚠️ ERRATA APLICADA — Leia ERRATA.md

Correções críticas foram aplicadas ao design original:
- rusttype → fontdue (2x mais rápido, mantido)
- async_trait removido (nativo desde Rust 1.75)
- xcb-wm adicionado (EWMH/ICCCM prontos)
- xkbcommon-rs adicionado (keyboard real)
- bincode pinnado em 1.3.3
- zbus atualizado para 4.x
- Traffic lights: 8px margin (não 20px)

## Dependency Rule (NUNCA violar)

```
Presentation → Application → Domain ← Infrastructure
```

- `de-core` (Domain) não importa NADA externo além de `serde` e `thiserror`
- `de-x11`, `de-render`, `de-ipc` (Infrastructure) implementam traits definidos em `de-core`
- `wm`, `compositor`, `panel`, etc. (Application+Presentation) dependem de `de-core` e das infra crates
- **NUNCA** um crate de domínio importa um crate de infraestrutura

## Estrutura de Crates

| Crate | Camada | Responsabilidade |
|-------|--------|-----------------|
| `de-core` | Domain | Entidades, Value Objects, Ports (traits), Erros de domínio |
| `de-ipc` | Infrastructure | IPC Bus via Unix Domain Sockets, protocolo de mensagens |
| `de-config` | Infrastructure | Parsing TOML, hot-reload, validação de schema |
| `de-theme` | Domain+Application | Definições de tema, macOS Sonoma theme |
| `de-x11` | Infrastructure | Conexão X11, EWMH, ICCCM, event loop |
| `de-render` | Infrastructure | Software renderer (tiny-skia), text rendering |
| `wm` | Application+Presentation | Window Manager, workspaces, hotkeys, decorações |
| `compositor` | Application+Presentation | Compositor CPU, scene graph, damage tracking |
| `panel` | Application+Presentation | Barra de sistema, widgets, menu |
| `launcher` | Application+Presentation | Lançador de apps, fuzzy search |
| `notifications` | Application+Presentation | Notificações, D-Bus, histórico |
| `session` | Application | Session manager, startup, crash recovery |

## Padrões Obrigatórios

### Erros — sempre usar thiserror

```rust
// CORRETO
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Window {0:?} not found")]
    NotFound(WindowId),
    #[error("Size {size:?} violates minimum constraint {min:?}")]
    SizeTooSmall { size: Size, min: Size },
}

// ERRADO — nunca fazer isso
fn foo() -> Result<(), String> { ... }
fn bar() -> Option<Window> { ... }  // quando pode retornar erro
```

### Entidades de Domínio — IDs como newtypes

```rust
// CORRETO — type-safe, não confunde WindowId com WorkspaceId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(pub u32);

// ERRADO
type WindowId = u32;  // pode ser confundido com qualquer u32
```

### Traits para Ports (Dependency Inversion)

```rust
// CORRETO — Domain define a interface, Infrastructure implementa
// em de-core/src/ports/window_system.rs
pub trait WindowSystem: Send + Sync {
    fn map_window(&self, id: WindowId) -> Result<(), WindowSystemError>;
    fn move_window(&self, id: WindowId, pos: Position) -> Result<(), WindowSystemError>;
}

// em de-x11/src/lib.rs
pub struct X11Backend { ... }
impl WindowSystem for X11Backend { ... }

// ERRADO — Application layer importando X11 diretamente
use xcb::Connection;  // nunca em wm/ ou compositor/
```

### Async vs Sync

```rust
// Async: operações de I/O (IPC, filesystem, D-Bus, process spawn)
// IMPORTANTE: Desde Rust 1.75, async fn em traits é nativo — NÃO usar async_trait
pub trait WindowSystem: Send + Sync {
    async fn map_window(&self, id: WindowId) -> Result<(), WindowSystemError>;
}

// Sync: lógica de domínio pura (sem I/O)
pub fn resize(&mut self, size: Size) -> Result<(), WindowError> { ... }
pub fn contains_point(&self, point: Position) -> bool { ... }
```

### Sem unwrap() em código de produção

```rust
// CORRETO
let window = workspace.get_window(id)
    .ok_or(WindowError::NotFound(id))?;

// ERRADO — panic em produção
let window = workspace.get_window(id).unwrap();
```

## Estrutura de Pastas dentro de cada Crate

```
crate-name/
├── Cargo.toml
└── src/
    ├── lib.rs          # Re-exports públicos, doc do crate
    ├── domain/         # (só em de-core) entidades e value objects
    ├── ports/          # (só em de-core) traits/interfaces
    ├── error.rs        # Tipos de erro do crate
    └── <módulos>.rs    # Implementações
```

## IPC — Protocolo de Mensagens

Todas as mensagens entre processos passam pelo `de-ipc`. Nunca comunicação direta entre processos sem passar pelo bus. Mensagens são definidas em `de-ipc/src/protocol.rs` como enum `Message` com variantes tipadas.

## Logging — sempre usar tracing

```rust
use tracing::{debug, info, warn, error};

// CORRETO
info!(window_id = ?id, "Window created");
debug!(position = ?pos, "Moving window");
error!(error = %e, "Failed to connect to X11");

// ERRADO
println!("Window created: {:?}", id);
eprintln!("Error: {}", e);
```
