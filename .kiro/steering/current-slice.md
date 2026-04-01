---
inclusion: always
---

# Slice Atual — Slice 0: Foundation MVP

## Status: 🔲 Não iniciado

## Objetivo

Provar que conseguimos conectar X11 → Domain → Rendering. Ao final deste slice, deve existir uma janela simples abrindo na tela com um retângulo renderizado via CPU.

## O que implementar neste slice

### 1. Setup do Cargo Workspace (~1h)
- Criar `Cargo.toml` na raiz com todos os membros do workspace
- Criar estrutura de pastas para `de-core`, `de-x11`, `de-render`
- Criar um binário temporário `de-mvp` para integrar tudo

### 2. de-core — Domain básico (~2h)
Arquivos a criar:
- `crates/de-core/src/domain/geometry.rs` — `Position`, `Size`, `Rectangle`
- `crates/de-core/src/domain/color.rs` — `Color` (r, g, b, a)
- `crates/de-core/src/domain/window.rs` — `WindowId`, `Window` (versão mínima)
- `crates/de-core/src/error.rs` — `CoreError`
- `crates/de-core/src/lib.rs` — re-exports

### 3. de-x11 — Conexão básica (~4h)
Arquivos a criar:
- `crates/de-x11/src/connection.rs` — conectar ao X11, criar janela, event loop
- `crates/de-x11/src/error.rs` — `X11Error`
- `crates/de-x11/src/lib.rs`

Funcionalidades mínimas:
- Conectar ao X11 server
- Criar uma janela com tamanho configurável
- Event loop que responde a `KeyPress` (ESC fecha) e `Expose`

### 4. de-render — Renderer básico (~4h)
Arquivos a criar:
- `crates/de-render/src/renderer.rs` — `SoftwareRenderer` com tiny-skia
- `crates/de-render/src/buffer.rs` — gerenciar framebuffer
- `crates/de-render/src/error.rs` — `RenderError`
- `crates/de-render/src/lib.rs`

Funcionalidades mínimas:
- Criar `Pixmap` do tiny-skia com tamanho da janela
- Renderizar retângulo colorido
- Copiar buffer para X11 via `xcb::x::PutImage`

### 5. Integração — de-mvp (~2h)
- `crates/de-mvp/src/main.rs` — conecta X11 + Renderer
- Ao receber `Expose`: renderiza e apresenta
- Ao receber `ESC`: fecha

## Checkpoint de Conclusão

```bash
cargo build --workspace          # zero warnings
cargo clippy --workspace -- -D warnings  # zero erros
cargo run --bin de-mvp           # abre janela 800x600 com retângulo azul
# ESC deve fechar a janela
```

## Dependências deste Slice (Cargo.toml)

```toml
# de-core — zero dependências externas além de:
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"

# de-x11
xcb = { version = "1.3", features = ["xkb"] }
thiserror = "1.0"

# de-render
tiny-skia = "0.11"
fontdue = "0.7"      # ← text rendering (NÃO rusttype)
thiserror = "1.0"
```

## ⚠️ IMPORTANTE: Dependências Corrigidas

Leia `.kiro/specs/rust-de-ecosystem/ERRATA.md` antes de começar.
Principais mudanças:
- **fontdue** ao invés de rusttype (2x mais rápido)
- **SEM async_trait** (nativo desde Rust 1.75)
- xcb-wm e xkbcommon-rs serão adicionados no Slice 1

## Conceitos que você vai aprender neste slice

- **X11 connection model**: como o protocolo X11 funciona (client-server, display, screen, root window)
- **X11 window lifecycle**: criar janela → mapear → receber eventos → destruir
- **Framebuffer**: o que é um pixmap, como pixels são organizados em memória
- **tiny-skia API**: como criar um Pixmap, desenhar primitivas, acessar os bytes
- **PutImage**: como enviar pixels do CPU para o X11 server exibir na tela
- **Cargo workspace**: como organizar múltiplos crates em um projeto

## Próximo Slice

Após concluir: **Slice 1 — Window Manager Básico** (gerenciar janelas de apps reais, EWMH/ICCCM, move/resize, hotkeys)

---
*Atualizar este arquivo quando avançar para o próximo slice*
