# 📋 Plano de Refatoração — Rust DE Ecosystem 2026

**Data:** 2026-04-07  
**Status:** Análise completa do estado atual + definição de roadmap  
**Objetivo:** Refatorar o projeto para arquitetura escalável com wgpu, separação adequada de componentes e preparação para produção

---

## 1. 🔍 Análise do Estado Atual

### ✅ O Que Está Bem Feito

1. **Estrutura de Domain (de-core)**
   - `Size` com invariantes protegidas (campos privados, construtor validado)
   - `Rectangle::new` já corrigido para retornar `Option`
   - `Position`, `Color` como Value Objects
   - Separação clara domain/application/ports em `de-core`

2. **Arquitetura Base**
   - Dependency Rule respeitada (de-core não depende de infra)
   - Traits definidos em ports para inversão de dependência
   - IPC via Unix Domain Sockets implementado
   - X11 backend funcional

3. **Documentação**
   - `.kiro/specs/` com 8000+ linhas de design documentado
   - ERRATA.md com correções críticas aplicadas
   - Code review com issues identificadas e corrigidas

### ❌ Problemas Identificados

1. **Estrutura de Pastas Confusa**
   - Todos os crates em `/crates/` sem separação por camada
   - Mistura libs (`de-core`, `de-ipc`) com apps (`wm`, `panel`)
   - Difícil identificar o que é biblioteca vs aplicação

2. **Acoplamento entre Componentes**
   - Apps podem estar importando outros apps (verificar Cargo.toml)
   - Falta regra clara: "apps não dependem de apps"

3. **Renderização Apenas CPU**
   - `de-render` usa apenas `tiny-skia`
   - Sem suporte a GPU via wgpu/llvmpipe
   - Shaders SDF para rounded corners/sombras não implementados

4. **Bugs Pendentes (code_review.md)**
   - Bug 1: `Rectangle::new` bypassava invariante → ✅ JÁ CORRIGIDO
   - Bug 2: `intersection()` acessava campos diretos → ✅ JÁ CORRIGIDO
   - Design 1: `client.rs` incompleto → PENDENTE
   - Design 2: Dois blocos `impl` separados no EventDispatcher → PENDENTE
   - Design 3: `manage_window` pode criar Size inválido → PENDENTE

5. **Specs macOS Sonoma Não Implementados**
   - Traffic lights: 12px diâmetro, 8px spacing/margin
   - Title bar: 28px altura
   - Cores SF System não padronizadas
   - Menu bar: 24px altura (não 32px)
   - Notificações: 320px largura, canto superior DIREITO

---

## 2. 🏗️ Nova Estrutura de Projeto (Baseada em KDE/macOS)

### Princípios de Organização

1. **Separação por Camadas Físicas**
   - `core/` → Domain puro, zero deps externas
   - `infra/` → Backends técnicos (X11, rendering, IPC)
   - `apps/` → Binários independentes (WM, Compositor, Panel)
   - `libs/` → Bibliotecas compartilhadas (theme, widgets)

2. **Regra de Dependência CRÍTICA**
   ```
   apps/ NÃO pode depender de outros apps/
   apps/ DEPENDE apenas de core/, infra/, libs/
   infra/ DEPENDE apenas de core/
   core/ NÃO DEPENDE de nada além de serde/thiserror
   ```

3. **Cada App é um Processo Isolado**
   - WM, Compositor, Panel rodam como binários separados
   - Comunicação apenas via IPC (`de-protocol`)
   - Crash de um não derruba os outros

### Estrutura Proposta

```
/workspace/
├── Cargo.toml                    # Workspace root
├── README.md
├── docs/                         # Documentação geral
│   └── code_review.md
├── .kiro/                        # Steering docs (manter)
│   ├── specs/
│   └── steering/
│
├── core/                         # DOMAIN PURO
│   ├── de-domain/                # Entidades, VOs, Ports (atual de-core)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entities/         # Window, Workspace
│   │       ├── value_objects/    # Position, Size, Rectangle, Color
│   │       ├── ports/            # Traits: WindowSystem, Renderer, IPC
│   │       └── errors.rs
│   │
│   └── de-protocol/              # Protocolo IPC (mensagens)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── messages.rs       # Enum Message com variantes
│           ├── component.rs      # ComponentType enum
│           └── serialization.rs  # bincode wrappers
│
├── infra/                        # BACKENDS TÉCNICOS
│   ├── de-x11/                   # X11 backend (atual crates/de-x11)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── connection.rs
│   │       ├── events.rs
│   │       ├── adapter.rs        # X11Adapter impl WindowSystem port
│   │       └── error.rs
│   │
│   ├── de-render-trait/          # TRAIT de rendering (NOVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs       # trait Renderer
│   │       ├── primitives.rs     # Rect, Circle, Text structs
│   │       └── error.rs
│   │
│   ├── de-render-cpu/            # tiny-skia impl (NOVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs       # impl Renderer for CpuRenderer
│   │       ├── buffer.rs
│   │       └── text.rs           # fontdue integration
│   │
│   ├── de-render-gpu/            # wgpu impl (NOVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs       # impl Renderer for GpuRenderer
│   │       ├── pipeline.rs       # wgpu render pipelines
│   │       ├── shaders/          # WGSL shaders
│   │       │   ├── rounded_rect.wgsl
│   │       │   ├── shadow.wgsl
│   │       │   └── blur.wgsl
│   │       └── text.rs           # glyphon integration
│   │
│   ├── de-ipc-transport/         # Unix sockets transport (NOVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── socket.rs
│   │       ├── server.rs
│   │       └── client.rs
│   │
│   └── de-ipc-bus/               # IPC Bus actor system (refatorar atual)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── bus.rs            # Actor com mpsc channels
│           ├── registry.rs       # Component registry
│           └── heartbeat.rs
│
├── libs/                         # BIBLIOTECAS COMPARTILHADAS
│   ├── de-theme/                 # Theme tokens (mover crates/de-theme)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── colors.rs         # SF System colors
│   │       ├── spacing.rs        # 8px grid system
│   │       ├── typography.rs     # SF Pro specs
│   │       └── sonoma.rs         # macOS Sonoma theme
│   │
│   ├── de-widgets/               # Widget library (NOVO)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── button.rs
│   │       ├── label.rs
│   │       ├── panel.rs
│   │       └── dock.rs
│   │
│   └── de-config/                # Config manager (mover crates/de-config)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── parser.rs         # TOML parsing
│           ├── schema.rs         # Config schemas
│           └── hot_reload.rs
│
└── apps/                         # APLICAÇÕES (BINÁRIOS)
    ├── de-wm/                    # Window Manager (mover crates/wm)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── application/
    │       │   ├── mod.rs
    │       │   ├── window_manager.rs
    │       │   ├── workspace_manager.rs
    │       │   └── hotkeys.rs
    │       ├── presentation/
    │       │   ├── mod.rs
    │       │   ├── event_dispatcher.rs
    │       │   └── x11_adapter.rs
    │       └── error.rs
    │
    ├── de-compositor/            # Compositor (mover crates/compositor)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── scene_graph.rs
    │       ├── damage_tracker.rs
    │       ├── decorations.rs    # Rounded corners, shadows via SDF
    │       └── vsync.rs
    │
    ├── de-panel/                 # Panel/Menu bar (mover crates/panel)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── menu_bar.rs       # 24px height, SF Pro 13px
    │       ├── status_icons.rs
    │       └── widgets/
    │
    ├── de-launcher/              # App launcher (mover crates/launcher)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── fuzzy_search.rs   # nucleo-matcher
    │       └── ui.rs
    │
    ├── de-notifications/         # Notification daemon (mover crates/notifications)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── daemon.rs
    │       ├── dbus.rs           # freedesktop.org compat
    │       └── ui.rs
    │
    ├── de-session/               # Session manager (mover crates/session)
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs
    │       ├── startup.rs
    │       ├── monitor.rs        # Health check, restart
    │       └── shutdown.rs
    │
    └── de-mvp/                   # MVP de integração (manter para testes)
        ├── Cargo.toml
        └── src/
            └── main.rs
```

---

## 3. 📦 Dependências por Camada

### core/de-domain
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
# ZERO dependências externas além dessas
```

### core/de-protocol
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
bincode = "=1.3.3"  # pinnado
de-domain = { path = "../core/de-domain" }
```

### infra/de-render-trait
```toml
[dependencies]
de-domain = { path = "../../core/de-domain" }
thiserror = "1.0"
```

### infra/de-render-cpu
```toml
[dependencies]
de-render-trait = { path = "../de-render-trait" }
de-domain = { path = "../../core/de-domain" }
tiny-skia = "0.11"
fontdue = "0.7"
```

### infra/de-render-gpu
```toml
[dependencies]
de-render-trait = { path = "../de-render-trait" }
de-domain = { path = "../../core/de-domain" }
wgpu = "0.20"
glyphon = "0.6"  # text rendering com wgpu
bytemuck = "1.14"
```

### apps/de-wm
```toml
[dependencies]
de-domain = { path = "../../core/de-domain" }
de-protocol = { path = "../../core/de-protocol" }
de-x11 = { path = "../../infra/de-x11" }
tokio = { version = "1.35", features = ["full"] }
tracing = "0.1"
xcb = { version = "1.3", features = ["xkb", "randr", "composite"] }
xcb-wm = "0.4"
xkbcommon = "0.7"
```

### apps/de-compositor
```toml
[dependencies]
de-domain = { path = "../../core/de-domain" }
de-protocol = { path = "../../core/de-protocol" }
de-render-trait = { path = "../../infra/de-render-trait" }
de-theme = { path = "../../libs/de-theme" }
# Escolha UM backend via feature flag
[features]
default = ["cpu"]
cpu = ["de-render-cpu"]
gpu = ["de-render-gpu"]
```

---

## 4. 🎯 Especificações macOS Sonoma para Implementação

### Window Chrome
```rust
// libs/de-theme/src/sonoma.rs
pub const WINDOW_TITLE_BAR_HEIGHT: f32 = 28.0;
pub const WINDOW_CORNER_RADIUS: f32 = 10.0;
pub const TRAFFIC_LIGHT_DIAMETER: f32 = 12.0;
pub const TRAFFIC_LIGHT_SPACING: f32 = 8.0;
pub const TRAFFIC_LIGHT_MARGIN_LEFT: f32 = 8.0;
pub const TRAFFIC_LIGHT_MARGIN_TOP: f32 = 8.0; // (28-12)/2 = 8
pub const SHADOW_BLUR_RADIUS: f32 = 20.0;
pub const SHADOW_OFFSET_Y: f32 = 4.0;
```

### Cores SF System
```rust
// libs/de-theme/src/colors.rs
pub mod light {
    pub const BACKGROUND_WINDOW: Color = Color::rgb(236, 236, 236);  // #ECECEC
    pub const ACCENT_BLUE: Color = Color::rgb(0, 122, 255);          // #007AFF
    pub const TRAFFIC_CLOSE: Color = Color::rgb(255, 95, 87);        // #FF5F57
    pub const TRAFFIC_MINIMIZE: Color = Color::rgb(254, 188, 46);    // #FEBC2E
    pub const TRAFFIC_MAXIMIZE: Color = Color::rgb(40, 200, 64);     // #28C840
}

pub mod dark {
    pub const BACKGROUND_WINDOW: Color = Color::rgb(28, 28, 30);     // #1C1C1E
    pub const ACCENT_BLUE: Color = Color::rgb(10, 132, 255);         // #0A84FF
}
```

### Menu Bar
```rust
pub const MENU_BAR_HEIGHT: f32 = 24.0;  // NÃO 32px!
pub const MENU_BAR_PADDING_H: f32 = 8.0;
pub const MENU_BAR_FONT_SIZE: f32 = 13.0;
pub const MENU_BAR_ICON_SIZE: f32 = 18.0;
```

### Notificações
```rust
pub const NOTIFICATION_WIDTH: f32 = 320.0;  // NÃO 400px!
pub const NOTIFICATION_PADDING: f32 = 16.0;
pub const NOTIFICATION_CORNER_RADIUS: f32 = 10.0;
pub const NOTIFICATION_POSITION: Position = Position::TopRight;
pub const NOTIFICATION_MARGIN: f32 = 20.0;
```

---

## 5. 🚀 Roadmap de Implementação

### Fase 1: Correções Imediatas (1-2 dias)
**Objetivo:** Resolver bugs pendentes e validar funcionamento atual

1. ✅ Corrigir `client.rs` incompleto (deletar ou implementar)
2. ✅ Unificar blocos `impl` no EventDispatcher
3. ✅ Corrigir `manage_window` para evitar Size inválido
4. ✅ Testar WM no Xephyr: `Xephyr :1 -screen 1920x1080 & DISPLAY=:1 cargo run --bin wm`
5. ✅ Validar hotkeys funcionando DURANTE drag (Bug BUTTON1)

### Fase 2: Extração de Core (2-3 dias)
**Objetivo:** Separar domain e protocol em crates independentes

1. Criar `core/de-domain/` com código atual de `de-core/src/domain/`
2. Criar `core/de-protocol/` extraindo mensagens de `de-ipc/`
3. Atualizar imports em todos os crates
4. Validar compilação: `cargo build --workspace`

### Fase 3: Infra Rendering (3-4 dias)
**Objetivo:** Criar trait Renderer e backends CPU/GPU

1. Criar `infra/de-render-trait/` com trait `Renderer`
2. Mover `de-render` atual para `infra/de-render-cpu/`
3. Criar `infra/de-render-gpu/` com wgpu + OpenGL backend
4. Testar llvmpipe: `WGPU_BACKEND=gl cargo run --bin de-compositor --features gpu`

### Fase 4: Reorganização de Apps (2-3 dias)
**Objetivo:** Mover apps para estrutura final

1. Mover `crates/wm` → `apps/de-wm/`
2. Mover `crates/compositor` → `apps/de-compositor/`
3. Mover `crates/panel` → `apps/de-panel/`
4. Mover libs restantes para `libs/`
5. Atualizar paths no workspace Cargo.toml

### Fase 5: Implementação macOS Specs (4-5 dias)
**Objetivo:** Aplicar design system Sonoma

1. Implementar `libs/de-theme/` com cores e medidas exatas
2. Atualizar decorações de janela no compositor
3. Implementar menu bar 24px no panel
4. Implementar notificações 320px canto direito
5. Adicionar fontes Inter (fallback SF Pro)

### Fase 6: Shaders SDF (3-4 dias)
**Objetivo:** Rounded corners e sombras via GPU

1. Criar shader `rounded_rect.wgsl` com SDF
2. Criar shader `shadow.wgsl` com Gaussian blur
3. Integrar no `de-render-gpu`
4. Benchmark CPU vs GPU (mesmo com llvmpipe)

---

## 6. ⚠️ Riscos e Mitigações

| Risco | Impacto | Mitigação |
|-------|---------|-----------|
| wgpu não funcionar com llvmpipe | Alto | Manter fallback CPU, testar cedo |
| Refatoração quebrar IPC | Médio | Testes de integração após cada fase |
| Performance GPU < CPU no llvmpipe | Baixo | Benchmarks, usar só se valer pena |
| Complexidade excessiva | Médio | Manter simples, iterar rápido |

---

## 7. ✅ Critérios de Conclusão

### Fase 1
- [ ] Zero warnings no `cargo clippy --workspace`
- [ ] WM rodando no Xephyr
- [ ] Hotkeys funcionando durante drag

### Fase 2
- [ ] `core/de-domain` compila isolado
- [ ] `core/de-protocol` compila isolado
- [ ] Todos os apps compilam com novos paths

### Fase 3
- [ ] Trait `Renderer` definido em `de-render-trait`
- [ ] Backend CPU funcional
- [ ] Backend GPU compila (mesmo sem teste visual)

### Fase 4
- [ ] Estrutura `core/`, `infra/`, `libs/`, `apps/` completa
- [ ] Nenhum app importa outro app
- [ ] Workspace compila inteiro

### Fase 5
- [ ] Cores SF System aplicadas
- [ ] Medidas macOS Sonoma respeitadas
- [ ] Screenshot comparativo com macOS

### Fase 6
- [ ] Shaders WGSL implementados
- [ ] Rounded corners anti-aliased
- [ ] Sombras reais com blur
- [ ] Benchmark: GPU >= CPU em performance

---

## 8. 📊 Métricas de Sucesso

- **Cobertura de testes:** >90% (após implementação base)
- **Performance:** <16ms por frame (60 FPS) mesmo com llvmpipe
- **Memória:** <200MB total com 8 janelas abertas
- **Startup:** <2s do login ao desktop utilizável
- **Crash recovery:** Restart automático em <1s

---

## 9. 🔧 Próximos Passos Imediatos

**HOJE:**
1. Validar bugs da Fase 1 estão realmente corrigidos
2. Rodar `cargo build --workspace` e garantir zero errors
3. Testar no Xephyr

**AMANHÃ:**
1. Iniciar Fase 2 (extração de core)
2. Criar `core/de-domain/` e `core/de-protocol/`
3. Migrar código existente

**ESTA SEMANA:**
1. Completar Fases 1-3
2. Ter trait Renderer funcionando com ambos backends
3. Validar llvmpipe com wgpu

---

*Este documento será atualizado conforme o progresso das fases.*
