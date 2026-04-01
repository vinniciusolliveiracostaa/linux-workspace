---
inclusion: always
---

# Rust DE/WM Ecosystem — Contexto do Projeto

## O que é este projeto

Um ecossistema completo de Desktop Environment (DE) e Window Manager (WM) escrito em Rust para Linux, inspirado fielmente no macOS Sonoma 14 em design, comportamento e filosofia de integração software/hardware. O projeto é pessoal, fechado, feito exclusivamente para o dono.

## Restrições de Hardware CRÍTICAS

- Notebook com `acpi=off` — **SEM aceleração gráfica, SEM Vulkan, SEM OpenGL**
- Renderização 100% CPU usando `tiny-skia`
- X11 obrigatório (Wayland é futuro distante)
- Display Manager (DM) será adicionado futuramente — não implementar agora

## Quem vai codar

O próprio usuário. O agente **não escreve código**, apenas:
- Guia a implementação passo a passo
- Explica o PORQUÊ de cada decisão
- Aponta o próximo passo claro
- Revisa e sugere melhorias
- Ensina enquanto o usuário programa

## Princípios de Desenvolvimento

- **DDD** (Domain-Driven Design) — entidades, value objects, domain services
- **Clean Architecture** — Domain → Application → Infrastructure → Presentation
- **SOLID** — cada módulo tem responsabilidade única e clara
- Sem if/else hell, sem try/catch desnecessários
- Código limpo e legível acima de tudo
- Async onde faz sentido (I/O), sync onde é mais simples
- Dependências mínimas — só adicionar se realmente necessário, preferir crates estáveis

## Abordagem de Testes

- Testes são escritos **APÓS** a implementação (não TDD)
- Meta: 90% de coverage
- Tipos: integração, property-based (proptest), benchmarks (criterion)
- Não cobrar testes durante implementação — cobrar depois

## Arquivos do Spec

- Requisitos: `.kiro/specs/rust-de-ecosystem/requirements.md`
- Design técnico: `.kiro/specs/rust-de-ecosystem/design.md`
- Tasks: `.kiro/specs/rust-de-ecosystem/tasks.md`

## Referência de Design

macOS Sonoma 14 — fiel ao design system da Apple:
- Traffic lights (botões de janela): vermelho/amarelo/verde, 12px, espaçamento 8px, margem 20px
- Menu bar no topo, Dock na base
- Tipografia: SF Pro (ou Inter como fallback open source)
- Corner radius de janelas: 10px
- Sombras suaves sem GPU (múltiplas camadas alpha)
