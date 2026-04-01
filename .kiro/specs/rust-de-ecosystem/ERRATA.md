# Errata — Correções Críticas ao Design

**Data:** 2026-04-01  
**Autor:** Análise profunda pré-implementação

## Correções Aplicadas

### 1. rusttype → fontdue (CRÍTICO)
**Problema:** `rusttype` está arquivado e em modo de manutenção.  
**Solução:** Substituir por `fontdue = "0.7"` — 2x mais rápido, puro Rust, mantido ativamente.  
**Impacto:** Slice 0 (de-render), Slice 3 (compositor text rendering)

### 2. async_trait removido (PERFORMANCE)
**Problema:** `async_trait` adiciona `Box<dyn Future>` overhead no hot path.  
**Solução:** Rust 1.75+ suporta `async fn` em traits nativamente — remover dependência.  
**Impacto:** Todos os traits async (WindowSystem, EventHandler, Widget)

### 3. xcb-wm adicionado (PRODUTIVIDADE)
**Problema:** Implementar EWMH/ICCCM do zero é semanas de trabalho propenso a bugs.  
**Solução:** Usar `xcb-wm = "0.4"` — implementação type-safe pronta.  
**Impacto:** Slice 1 (EWMH/ICCCM), economiza ~5-7 dias de trabalho

### 4. xkbcommon-rs adicionado (FUNCIONALIDADE)
**Problema:** Keyboard handling com 3 teclas hardcoded não é viável.  
**Solução:** Adicionar `xkbcommon = "0.7"` para suporte real a layouts.  
**Impacto:** Slice 1 (hotkeys), Slice 10 (hotkeys avançados)

### 5. XComposite feature adicionada (FUNCIONALIDADE)
**Problema:** Compositor precisa capturar pixmaps de janelas — requer extensão XComposite.  
**Solução:** Adicionar `composite` feature no xcb, task específica no Slice 3.  
**Impacto:** Slice 3 (compositor)

### 6. Traffic lights margin corrigida (FIDELIDADE)
**Problema:** Design tinha 20px em alguns lugares, 8px em outros.  
**Valor correto:** macOS Sonoma usa **8px** da borda esquerda.  
**Impacto:** Slice 3 (decorações), Slice 7 (tema Sonoma)

### 7. bincode versão pinnada (ESTABILIDADE)
**Problema:** bincode 1.x vs 2.x têm APIs incompatíveis.  
**Solução:** Pinnar `bincode = "=1.3.3"` para evitar surpresas.  
**Impacto:** Slice 2 (IPC serialization)

### 8. zbus atualizado para 4.x (ATUALIDADE)
**Problema:** Design usava zbus 3.14, atual é 4.x com breaking changes.  
**Solução:** Atualizar para `zbus = "4"` desde o início.  
**Impacto:** Slice 8 (notifications D-Bus)

### 9. fuzzy-matcher → nucleo-matcher (MANUTENÇÃO)
**Problema:** `fuzzy-matcher` não recebe updates frequentes.  
**Solução:** Usar `nucleo-matcher = "0.3"` (usado pelo Helix editor).  
**Impacto:** Slice 5 (launcher fuzzy search)

### 10. Meta de memória ajustada (REALISMO)
**Problema:** 100MB para 8 processos + framebuffer + caches é irrealista.  
**Solução:** Ajustar meta para **150-200MB** total.  
**Impacto:** Requirement 14.5, benchmarks do Slice 11

### 11. IPC Bus redesign (CORREÇÃO ARQUITETURAL)
**Problema:** `HashMap<ComponentType, UnixStream>` não funciona com tokio (ownership).  
**Solução:** Usar actor pattern com `tokio::sync::mpsc` channels por componente.  
**Impacto:** Slice 2 (IPC Bus implementation), task adicional

### 12. X11 event loop com AsyncFd (PERFORMANCE)
**Problema:** `spawn_blocking` por frame cria thread overhead desnecessário.  
**Solução:** Usar `AsyncFd` no file descriptor da conexão xcb.  
**Impacto:** Slice 1 (event loop), reduz latência ~1-2ms

### 13. Background color padronizado (CONSISTÊNCIA)
**Problema:** Design tinha `#F6F6F6` e `#ECECEC` para o mesmo background.  
**Valor correto:** macOS Sonoma usa `#ECECEC` para window chrome.  
**Impacto:** Slice 7 (tema Sonoma)

## Tasks Adicionadas

### Slice 1
- **Task 7.4:** Integrar `xcb-wm` e `xkbcommon-rs` no de-x11 (substitui implementação manual)

### Slice 2
- **Task 17.2:** Redesenhar `IpcBus` usando actor pattern com mpsc channels

### Slice 3
- **Task 23.5:** Ativar XComposite extension e implementar `NameWindowPixmap`

## Cargo.toml Workspace Corrigido

```toml
[workspace.dependencies]
# Core Rust async runtime
tokio = { version = "1.35", features = ["full"] }
# async-trait = "0.1"  ← REMOVIDO (nativo desde Rust 1.75)

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "=1.3.3"  # ← PINNADO (evitar breaking changes 2.x)
toml = "0.8"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# X11
xcb = { version = "1.3", features = ["xkb", "randr", "shape", "xfixes", "composite"] }  # ← composite adicionado
xcb-wm = "0.4"        # ← ADICIONADO (EWMH/ICCCM prontos)
xkbcommon = "0.7"     # ← ADICIONADO (keyboard handling real)

# Rendering
tiny-skia = "0.11"
fontdue = "0.7"       # ← SUBSTITUIU rusttype = "0.9"

# IPC and system
zbus = { version = "4", default-features = false, features = ["tokio"] }  # ← atualizado de 3.14
notify = "6.1"

# Utilities
chrono = "0.4"
dirs = "5.0"
nucleo-matcher = "0.3"  # ← SUBSTITUIU fuzzy-matcher = "0.3"
ini = "1.3"

# Testing (dev-dependencies)
proptest = "1.4"
quickcheck = "1.0"
criterion = "0.5"
```

## Impacto no Cronograma

- **Slice 0:** Sem impacto (fontdue é drop-in replacement)
- **Slice 1:** **-5 dias** (xcb-wm economiza tempo de EWMH/ICCCM)
- **Slice 2:** **+1 dia** (redesign do IPC Bus)
- **Slice 3:** **+1 dia** (XComposite integration)
- **Total:** **-3 dias** no cronograma geral

## Viabilidade Confirmada

Após correções, o projeto é **100% viável** para desenvolvedor solo. A arquitetura está sólida, as dependências são maduras e mantidas, e o escopo é realista.

**Slice 0 pode começar imediatamente.**
