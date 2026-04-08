# 🚀 Nova Estrutura do Projeto - Desktop Environment em Rust

## Visão Geral

Projeto refatorado seguindo arquitetura inspirada em macOS, KDE e GNOME, com separação clara de responsabilidades e foco em evolução incremental.

---

## 📁 Estrutura de Pastas

```
/workspace/
├── core/                    # Domain puro - Zero dependências externas
│   ├── de-domain/           # Entidades, value objects, regras de negócio
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── geometry.rs  # Rect, Position, Size, Vector
│   │   │   ├── window.rs    # Window, WindowId, WindowState
│   │   │   └── types.rs     # DamageRegion, SystemConfig, enums
│   │   └── Cargo.toml
│   │
│   └── de-protocol/         # Mensagens IPC entre componentes
│       ├── src/
│       │   ├── lib.rs
│       │   ├── messages.rs  # Request/Response commands
│       │   └── events.rs    # Broadcast events
│       └── Cargo.toml
│
├── infra/                   # Backends técnicos - Implementações
│   ├── de-x11/              # X11 backend (xcb wrapper)
│   ├── de-render-trait/     # Trait Renderer (abstração)
│   ├── de-render-cpu/       # tiny-skia implementation
│   ├── de-render-gpu/       # wgpu implementation (llvmpipe)
│   └── de-ipc/              # IPC transport (Unix sockets)
│
├── libs/                    # Bibliotecas compartilhadas
│   ├── de-theme/            # Design tokens (macOS Sonoma specs)
│   ├── de-widgets/          # Componentes UI reutilizáveis
│   └── de-config/           # Config management
│
└── apps/                    # Binários independentes
    ├── de-wm/               # Window Manager (foco inicial)
    ├── de-compositor/       # Compositor (wgpu + SDF shaders)
    ├── de-panel/            # Menu bar + Taskbar
    ├── de-launcher/         # App launcher
    └── de-notifications/    # Notification daemon
```

---

## 🏗️ Regras de Arquitetura

### 1. **Hierarquia de Dependências**

```
apps/ → libs/ → infra/ → core/
                      ↘
                       → infra/ (outro backend)
```

- `core/` NÃO depende de nada externo (apenas std + euclid)
- `infra/` só depende de `core/`
- `libs/` dependem de `core/` e opcionalmente `infra/`
- `apps/` podem depender de `core/`, `libs/`, e UM backend de `infra/`

### 2. **Isolamento de Apps**

**REGRA CRÍTICA:** Apps em `apps/` **NÃO** importam outros apps no `Cargo.toml`.

❌ Errado:
```toml
# apps/de-compositor/Cargo.toml
[dependencies]
de-wm = { path = "../de-wm" }  # PROIBIDO!
```

✅ Certo:
```toml
# apps/de-compositor/Cargo.toml
[dependencies]
de-domain = { path = "../../core/de-domain" }
de-protocol = { path = "../../core/de-protocol" }
de-render-gpu = { path = "../../infra/de-render-gpu" }
```

Comunicação entre apps ocorre **exclusivamente via IPC** (`de-protocol`).

### 3. **Traits vs Implementações**

Separar abstraction da implementação:

```rust
// infra/de-render-trait/src/lib.rs
pub trait Renderer: Send + Sync {
    fn draw_rect(&mut self, rect: Rect, color: Color, radius: f32);
    fn draw_shadow(&mut self, rect: Rect, blur: f32, color: Color);
    fn present(&mut self);
}

// infra/de-render-cpu/ → impl com tiny-skia
// infra/de-render-gpu/ → impl com wgpu
```

Apps escolhem backend via feature flag:
```toml
[features]
default = ["cpu"]
cpu = ["de-render-cpu"]
gpu = ["de-render-gpu"]
```

---

## 🎯 Plano de Implementação

### Fase 1: Fundação (SEMANA 1)
- [x] Estrutura de pastas criada
- [x] `de-domain` com tipos básicos
- [x] `de-protocol` com mensagens IPC
- [ ] `de-render-trait` com trait Renderer
- [ ] Testes unitários dos tipos de domínio

### Fase 2: WM Básico (SEMANA 2-3)
- [ ] `de-x11` backend funcional
- [ ] `de-wm` com gerenciamento básico de janelas
- [ ] Focus, raise, minimize
- [ ] Hotkeys (Super+Q, Super+Enter)
- [ ] Move/Resize via mouse grab

### Fase 3: Compositor com GPU (SEMANA 4-5)
- [ ] `de-render-gpu` com wgpu + llvmpipe
- [ ] Shaders WGSL para:
  - Rounded corners (SDF)
  - Sombras reais (Gaussian blur)
  - Traffic lights (macOS style)
- [ ] Damage tracking eficiente
- [ ] VSync via timer (16.67ms)

### Fase 4: Shell UI (SEMANA 6-8)
- [ ] `de-panel` (menu bar 24px altura)
- [ ] Taskbar com preview de janelas
- [ ] `de-launcher` com fuzzy search
- [ ] `de-notifications` (canto superior direito)
- [ ] `de-theme` com specs macOS Sonoma

### Fase 5: Polimento (SEMANA 9+)
- [ ] Settings system
- [ ] Config hot-reload
- [ ] Performance optimization
- [ ] Testes de integração
- [ ] Documentação

---

## 🎨 Design System - macOS Sonoma Specs

### Window Chrome
```
Title bar height:        28px (padrão), 22px (compact)
Traffic light diameter:  12px cada botão
Traffic light spacing:   8px entre botões
Traffic light margin:    8px da borda esquerda/topo
Window corner radius:    10px
Shadow blur:             20px, offset Y: 4px
```

### Cores (Light Mode)
```
Background window:    #ECECEC
Accent blue:          #007AFF
Traffic close:        #FF5F57
Traffic minimize:     #FEBC2E
Traffic maximize:     #28C840
Label primary:        rgba(0,0,0,1.0)
Label secondary:      rgba(0,0,0,0.6)
Separator:            rgba(0,0,0,0.09)
```

### Menu Bar
```
Altura:               24px (NÃO 32px!)
Padding horizontal:   8px
Font:                 SF Pro Text 13px, weight 400
Fallback Linux:       "Inter"
```

### Notificações
```
Largura:              320px (NÃO 400px!)
Padding:              16px
Corner radius:        10px
Posição:              canto superior DIREITO, 20px das bordas
Stack gap:            8px
Timeout:              5s (normal), sem timeout (critical)
```

---

## 🔧 Stack Técnica

### Rendering
- **wgpu 0.20** - GPU abstraction (Vulkan → OpenGL → llvmpipe)
- **tiny-skia 0.11** - CPU fallback
- **glyphon 0.7** - Text rendering via GPU
- **Shaders WGSL** - SDF para primitivos geométricos

### X11 Backend
- **xcb 1.5** - X11 protocol
- **xcb-wm 0.4** - WM helpers (EWMH/ICCCM)
- **xkbcommon 0.7** - Keyboard input

### IPC
- **zbus 4** - D-Bus (futuro)
- **Unix sockets** - IPC atual via `de-ipc`
- **bincode 1.3.3** - Serialização

### Utils
- **euclid 0.22** - Math types (Rect, Point, Vector)
- **tokio** - Async runtime
- **tracing** - Logging estruturado

---

## 🚦 Como Começar

### 1. Verificar Estado Atual
```bash
# Ainda não temos Rust instalado neste ambiente
# Quando tiver Rust:
cargo check --workspace
cargo clippy --workspace
```

### 2. Implementar Próximos Passos

**Ordem recomendada:**

1. **`de-render-trait`** - Definir trait `Renderer`
2. **`de-x11`** - Wrapper seguro do xcb
3. **`de-wm`** - Window manager básico
4. **Testar no Xephyr** - Validar funcionamento
5. **`de-render-gpu`** - wgpu + shaders SDF
6. **`de-compositor`** - Composição com GPU

---

## 📝 Lições Aprendidas (Backup Antigo)

### Bugs Corrigidos na Refatoração
1. ~~Hotkeys não disparam durante drag~~ → Resolver no `de-wm` stripando bits de mouse
2. ~~GrabState leak se janela fecha~~ → Resetar estado em `handle_destroy_notify`
3. ~~Warn spam de janelas não gerenciadas~~ → Silenciar logs de UnmapNotify
4. ~~`edition = "2024"`~~ → Usar `edition = "2021"`
5. ~~`src/main.rs` na raiz~~ → Removido

### Decisões de Arquitetura
- ✅ Workspace único para desenvolvimento (KDE usa repos separados, mas somos 1 pessoa)
- ✅ IPC obrigatório entre apps (modelo macOS distnoted)
- ✅ Trait Renderer para swap de backends (CPU/GPU)
- ✅ Domain puro sem deps externas (DDD verdadeiro)

---

## 🔍 Referências

### Arquitetura
- [macOS WindowServer Architecture](https://developer.apple.com/)
- [KDE Plasma Components](https://invent.kde.org/plasma/)
- [GNOME Mutter + Shell](https://gitlab.gnome.org/GNOME/)
- [Cosmic DE (System76)](https://github.com/pop-os/cosmic)

### Rendering
- [wgpu Documentation](https://docs.rs/wgpu)
- [40tude - GPU Selection Guide](https://www.40tude.fr/docs/06_programmation/rust/017_game_of_life/gpu.html)
- [GPUI Framework (Zed)](https://github.com/zed-industries/zed)

### Design
- [Apple HIG - macOS](https://developer.apple.com/design/human-interface-guidelines/macos/overview/themes/)
- [SF Pro Typography](https://developer.apple.com/fonts/)

---

## 📊 Status Atual

| Componente | Status | Próximo Passo |
|------------|--------|---------------|
| de-domain | ✅ Criado | Adicionar mais testes |
| de-protocol | ✅ Criado | Expandir mensagens |
| de-render-trait | ⏳ Pendente | Definir trait Renderer |
| de-x11 | ⏳ Pendente | Migrar código do backup |
| de-wm | ⏳ Pendente | Implementar event loop |
| de-compositor | ⏳ Pendente | Aguardar render-trait |
| de-panel | ⏳ Pendente | Aguardar compositor |
| de-render-gpu | ⏳ Pendente | Pesquisa wgpu + llvmpipe |

**Próxima ação imediata:** Implementar `de-render-trait` com trait `Renderer`.

---

*Documento criado em 2026-04-07 durante refatoração do projeto.*
