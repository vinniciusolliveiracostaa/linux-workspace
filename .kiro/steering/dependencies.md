---
inclusion: always
---

# Dependências Corretas — Rust DE/WM

## Cargo.toml Workspace (VERSÃO CORRIGIDA)

```toml
[workspace.dependencies]
# Core Rust async runtime
tokio = { version = "1.35", features = ["full"] }
# async-trait NÃO é necessário — Rust 1.75+ tem async fn em traits nativo

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "=1.3.3"  # PINNADO — evitar breaking changes do 2.x
toml = "0.8"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"

# X11
xcb = { version = "1.3", features = ["xkb", "randr", "shape", "xfixes", "composite"] }
xcb-wm = "0.4"       # EWMH/ICCCM prontos — não implementar do zero
xkbcommon = "0.7"    # Keyboard handling real

# Rendering
tiny-skia = "0.11"
fontdue = "0.7"      # Text rendering — NÃO rusttype (arquivado)

# IPC and system
zbus = { version = "4", default-features = false, features = ["tokio"] }
notify = "6.1"

# Utilities
chrono = "0.4"
dirs = "5.0"
nucleo-matcher = "0.3"  # Fuzzy search — NÃO fuzzy-matcher (pouco mantido)
ini = "1.3"

# Testing (dev-dependencies)
proptest = "1.4"
criterion = "0.5"
```

## Por que cada dependência (resumo)

| Crate | Versão | Justificativa |
|-------|--------|---------------|
| `tokio` | 1.35 | Runtime async maduro, necessário para IPC/filesystem não-bloqueante |
| `serde` | 1.0 | Padrão de serialization, zero-cost abstractions |
| `bincode` | =1.3.3 | Serialização binária rápida para IPC, pinnado para evitar 2.x |
| `toml` | 0.8 | Config files legíveis, comentários nativos |
| `thiserror` | 1.0 | Error types customizados com derive macro |
| `tracing` | 0.1 | Logging estruturado, padrão da indústria |
| `xcb` | 1.3 | X11 bindings thread-safe, maduro |
| `xcb-wm` | 0.4 | EWMH/ICCCM prontos, economiza semanas |
| `xkbcommon` | 0.7 | Keyboard layouts, keysyms, modifiers |
| `tiny-skia` | 0.11 | CPU rendering 2D, SIMD, zero unsafe |
| `fontdue` | 0.7 | Text rasterization rápida, puro Rust |
| `zbus` | 4 | D-Bus async para notifications (freedesktop.org spec) |
| `notify` | 6.1 | File watching para hot-reload de configs |
| `nucleo-matcher` | 0.3 | Fuzzy search mantido ativamente |
| `proptest` | 1.4 | Property-based testing |
| `criterion` | 0.5 | Benchmarking preciso |

## Dependências por Crate

### de-core (Domain — mínimas)
```toml
[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
```

### de-x11 (Infrastructure)
```toml
[dependencies]
de-core = { path = "../de-core" }
xcb = { workspace = true }
xcb-wm = { workspace = true }
xkbcommon = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
```

### de-render (Infrastructure)
```toml
[dependencies]
de-core = { path = "../de-core" }
tiny-skia = { workspace = true }
fontdue = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
```

### de-ipc (Infrastructure)
```toml
[dependencies]
de-core = { path = "../de-core" }
tokio = { workspace = true }
serde = { workspace = true }
bincode = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
```

### wm (Application+Presentation)
```toml
[dependencies]
de-core = { path = "../de-core" }
de-x11 = { path = "../de-x11" }
de-ipc = { path = "../de-ipc" }
de-config = { path = "../de-config" }
tokio = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
```

## Verificação de Compatibilidade

Antes de adicionar qualquer dependência nova:
1. Verificar se é mantida ativamente (último commit <6 meses)
2. Verificar se compila com Rust stable (sem nightly)
3. Verificar se não traz dependências de GPU (vulkan, opengl, mesa)
4. Verificar tamanho do binário final (`cargo bloat --release`)
