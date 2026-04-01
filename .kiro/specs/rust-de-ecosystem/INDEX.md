# Índice Completo — Rust DE/WM Ecosystem Spec

## Arquivos do Spec (`.kiro/specs/rust-de-ecosystem/`)

| Arquivo | Descrição | Quando Ler |
|---------|-----------|------------|
| `README.md` | Visão geral completa do projeto | **Primeiro** |
| `SUMMARY.md` | Resumo executivo (TL;DR) | Segundo |
| `ERRATA.md` | Correções críticas aplicadas | **OBRIGATÓRIO antes de codar** |
| `CHECKLIST.md` | Checklist pré-implementação | Antes de começar Slice 0 |
| `requirements.md` | 20 requisitos detalhados | Para entender o "o quê" |
| `design.md` | 7099 linhas de design técnico | Para entender o "como" |
| `tasks.md` | 80 tasks em 12 slices | Guia de implementação |
| `TROUBLESHOOTING.md` | Problemas comuns e soluções | Quando algo não funcionar |
| `.config.kiro` | Metadados do spec | Uso interno do Kiro |

## Steering Files (`.kiro/steering/`)

Carregados automaticamente em toda conversa:

| Arquivo | Conteúdo |
|---------|----------|
| `project-context.md` | Visão geral, restrições, filosofia |
| `dev-workflow.md` | Como agente se comporta, comandos úteis |
| `architecture-rules.md` | Dependency Rule, padrões obrigatórios |
| `current-slice.md` | Detalhe do slice atual (atualizar ao avançar) |
| `dependencies.md` | Cargo.toml correto com justificativas |
| `quick-reference.md` | Padrões de código, comandos, métricas (manual: `#quick-reference`) |

## Hooks (`.kiro/hooks/`)

Automação durante desenvolvimento:

| Hook | Trigger | Ação |
|------|---------|------|
| `rust-check-on-save` | Salvar `.rs` | Revisa arquitetura, unwrap, logging |
| `slice-checkpoint` | Editar `tasks.md` | Lembra checklist ao concluir slice |
| `new-crate-guide` | Criar arquivo em `crates/` | Orienta sobre camada correta |

## Ordem de Leitura Recomendada

### Para começar rápido (30 min)
1. `README.md` (5 min)
2. `SUMMARY.md` (3 min)
3. `ERRATA.md` (5 min) — **OBRIGATÓRIO**
4. `CHECKLIST.md` (2 min)
5. `tasks.md` — Slice 0 apenas (15 min)

### Para entendimento profundo (3-4h)
1. `README.md`
2. `SUMMARY.md`
3. `ERRATA.md` — **OBRIGATÓRIO**
4. `requirements.md` (30 min)
5. `design.md` (2-3h) — ler seções relevantes conforme avança
6. `tasks.md` — todos os slices (30 min)

### Durante implementação
- `tasks.md` — slice atual
- `TROUBLESHOOTING.md` — quando necessário
- `#quick-reference` no chat — padrões de código
- Steering files — carregados automaticamente

## Estrutura do Projeto (futuro)

```
rust-de-ecosystem/
├── Cargo.toml                    # Workspace root
├── .kiro/                        # Kiro config (já existe)
│   ├── specs/rust-de-ecosystem/  # Este spec
│   ├── steering/                 # Contexto automático
│   └── hooks/                    # Automação
├── crates/                       # Código (criar no Slice 0)
│   ├── de-core/                  # Domain
│   ├── de-x11/                   # Infrastructure
│   ├── de-render/                # Infrastructure
│   ├── de-ipc/                   # Infrastructure
│   ├── de-config/                # Infrastructure
│   ├── de-theme/                 # Domain+Application
│   ├── wm/                       # Application+Presentation
│   ├── compositor/               # Application+Presentation
│   ├── panel/                    # Application+Presentation
│   ├── launcher/                 # Application+Presentation
│   ├── notifications/            # Application+Presentation
│   └── session/                  # Application
├── config/                       # Default configs
├── assets/                       # Fonts, icons
├── docs/                         # User/dev documentation
└── scripts/                      # Build, install, uninstall
```

## Navegação Rápida

**Quero começar agora:**
→ `CHECKLIST.md` → `tasks.md` (Slice 0)

**Quero entender a arquitetura:**
→ `design.md` (seção Architecture)

**Quero ver as dependências:**
→ `.kiro/steering/dependencies.md`

**Algo não funciona:**
→ `TROUBLESHOOTING.md`

**Quero padrões de código:**
→ Chat: `#quick-reference`

**Quero ver o roadmap:**
→ `SUMMARY.md` ou `tasks.md`

## Status Atual

✅ Spec 100% completo  
✅ Errata aplicada  
✅ Steering files criados  
✅ Hooks configurados  
🔲 Implementação (próximo passo)

**Pronto para começar o Slice 0!**
