# Rust DE/WM Ecosystem — Spec Completo

**Status:** ✅ Design completo, pronto para implementação  
**Data:** 2026-04-01  
**Abordagem:** Requirements-First → Design → Tasks

## Arquivos do Spec

| Arquivo | Descrição | Linhas |
|---------|-----------|--------|
| `requirements.md` | 20 requisitos detalhados com acceptance criteria | ~600 |
| `design.md` | Design técnico completo (arquitetura, componentes, dependências, roadmap) | 7099 |
| `tasks.md` | 80 tasks em 12 slices com property tests e benchmarks | ~1200 |
| `ERRATA.md` | Correções críticas aplicadas após análise profunda | ~200 |
| `.config.kiro` | Metadados do spec para o Kiro | JSON |

## O que é este projeto

Um ecossistema completo de Desktop Environment (DE) e Window Manager (WM) escrito em Rust para Linux, inspirado fielmente no macOS Sonoma 14. Projetado para funcionar **sem aceleração gráfica** (acpi=off), usando renderização 100% CPU.

## Restrições de Hardware

- ❌ SEM GPU, SEM Vulkan, SEM OpenGL
- ✅ Renderização CPU com tiny-skia
- ✅ X11 obrigatório (Wayland é futuro)
- ✅ Display Manager será adicionado depois (Slice 12)

## Arquitetura

**Clean Architecture + DDD + SOLID**

```
Domain (de-core)
  ↑
Infrastructure (de-x11, de-render, de-ipc, de-config, de-theme)
  ↑
Application+Presentation (wm, compositor, panel, launcher, notifications, session)
```

**Multi-processo com IPC via Unix Domain Sockets**

Cada componente roda como processo separado para fault isolation. Comunicação via IPC Bus com serialização bincode.

## Dependências Principais (corrigidas)

- `tokio` — async runtime
- `xcb` + `xcb-wm` — X11 com EWMH/ICCCM prontos
- `xkbcommon` — keyboard handling real
- `tiny-skia` — CPU rendering 2D
- `fontdue` — text rasterization (NÃO rusttype)
- `bincode` — IPC serialization (pinnado em 1.3.3)
- `zbus` — D-Bus para notifications
- `nucleo-matcher` — fuzzy search

**⚠️ IMPORTANTE:** Leia `ERRATA.md` antes de começar — correções críticas foram aplicadas.

## Roadmap de Implementação

| Slice | Nome | Duração | Status |
|-------|------|---------|--------|
| 0 | Foundation MVP (X11 + Render) | 3-5 dias | 🔲 Não iniciado |
| 1 | Window Manager Básico | 5-7 dias | 🔲 Não iniciado |
| 2 | IPC Bus + Multi-Process | 4-6 dias | 🔲 Não iniciado |
| 3 | Compositor CPU | 5-7 dias | 🔲 Não iniciado |
| 4 | Panel + System Tray | 4-5 dias | 🔲 Não iniciado |
| 5 | Launcher + Fuzzy Search | 3-4 dias | 🔲 Não iniciado |
| 6 | Config Manager + Hot-reload | 3-4 dias | 🔲 Não iniciado |
| 7 | Theme Engine + Sonoma Theme | 3-4 dias | 🔲 Não iniciado |
| 8 | Notifications + D-Bus | 4-5 dias | 🔲 Não iniciado |
| 9 | Session Manager + Crash Recovery | 4-5 dias | 🔲 Não iniciado |
| 10 | Workspaces + Hotkeys | 3-4 dias | 🔲 Não iniciado |
| 11 | Polish + Performance + Testes | 5-7 dias | 🔲 Não iniciado |
| 12 | Display Manager (FUTURO) | - | ⏳ Planejado |

**Total estimado:** ~50-65 dias de trabalho solo

## Como Começar

### 1. Ler a documentação

```bash
# Requisitos
cat .kiro/specs/rust-de-ecosystem/requirements.md

# Design técnico (longo, mas vale a pena)
cat .kiro/specs/rust-de-ecosystem/design.md

# Correções críticas (OBRIGATÓRIO)
cat .kiro/specs/rust-de-ecosystem/ERRATA.md

# Tasks do Slice 0
cat .kiro/specs/rust-de-ecosystem/tasks.md | head -100
```

### 2. Setup do ambiente

```bash
# Instalar Rust stable
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instalar dependências do sistema (Debian/Ubuntu)
sudo apt install libxcb1-dev libxcb-xkb-dev libxkbcommon-dev

# Instalar Xephyr para testes
sudo apt install xserver-xephyr

# Verificar X11
echo $DISPLAY
xdpyinfo
```

### 3. Iniciar Slice 0

```bash
# Criar estrutura do workspace
mkdir -p crates/{de-core,de-x11,de-render,de-mvp}

# Criar Cargo.toml raiz
# (seguir tasks.md task 1)
```

### 4. Pedir ajuda ao agente

```
"Vamos começar o Slice 0. Estou na task 1 (Setup do Cargo Workspace). 
Me guia passo a passo no que fazer."
```

O agente vai:
- Explicar o PORQUÊ de cada decisão
- Mostrar o código que você deve escrever
- Revisar seu código quando você pedir
- Ensinar conceitos enquanto você programa

## Steering Files (contexto automático)

Estes arquivos são carregados automaticamente em toda conversa:

- `project-context.md` — visão geral, restrições, filosofia
- `dev-workflow.md` — como o agente se comporta, comandos úteis
- `architecture-rules.md` — Dependency Rule, padrões obrigatórios
- `current-slice.md` — detalhe do slice atual
- `dependencies.md` — Cargo.toml correto com justificativas

## Hooks (automação)

- `rust-check-on-save` — revisa arquitetura ao salvar `.rs`
- `slice-checkpoint` — lembra checklist ao concluir slice
- `new-crate-guide` — orienta ao criar novo arquivo

## Testes

- **Quando:** APÓS implementação (não TDD)
- **Meta:** 90% coverage
- **Tipos:** integração, property-based (proptest), benchmarks (criterion)
- **Slice 11:** dedicado a testes e performance

## Referência de Design

macOS Sonoma 14 — valores exatos:

- Traffic lights: 12px, espaçamento 8px, margem **8px** (não 20px)
- Title bar: 28px altura, corner radius 10px
- Panel: 24px altura (menu bar real do macOS)
- Notificações: 320px largura (não 400px)
- Cores: `#ECECEC` (window), `#007AFF` (accent light), `#0A84FF` (accent dark)

## Viabilidade

✅ **Projeto 100% viável** para desenvolvedor solo após correções da ERRATA.

- Arquitetura sólida e bem estruturada
- Dependências maduras e mantidas
- Escopo realista (~50-65 dias)
- Slices bem dimensionados (3-7 dias cada)
- Checkpoints claros a cada slice

## Próximos Passos

1. ✅ Spec completo (você está aqui)
2. 🔲 Implementar Slice 0 (Foundation MVP)
3. 🔲 Checkpoint: janela 800x600 com retângulo azul
4. 🔲 Implementar Slice 1 (Window Manager)
5. 🔲 Checkpoint: gerenciar janelas reais no Xephyr
6. ... continuar pelos slices

---

**Pronto para começar?** Abra uma nova conversa e diga: "Vamos começar o Slice 0"
