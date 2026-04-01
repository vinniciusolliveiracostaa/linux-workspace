# Pre-Implementation Checklist

Antes de começar o Slice 0, verificar:

## Documentação

- [ ] Li `README.md` completo
- [ ] Li `ERRATA.md` (OBRIGATÓRIO)
- [ ] Li `SUMMARY.md` para visão geral
- [ ] Entendi a estrutura de slices
- [ ] Sei onde está o `tasks.md`

## Ambiente

- [ ] Rust stable instalado (`rustc --version >= 1.75`)
- [ ] X11 rodando (`xdpyinfo` funciona)
- [ ] Xephyr instalado (`which Xephyr`)
- [ ] Dependências do sistema instaladas (libxcb, xkbcommon)
- [ ] Editor configurado (rust-analyzer, clippy)

## Conhecimento

- [ ] Entendo Clean Architecture (Domain → Application → Infrastructure)
- [ ] Entendo Dependency Rule (camadas internas não dependem de externas)
- [ ] Sei a diferença entre Entity e Value Object
- [ ] Sei quando usar async vs sync
- [ ] Sei que não devo usar unwrap() em produção

## Ferramentas

- [ ] `cargo clippy` funciona
- [ ] `cargo fmt` funciona
- [ ] `cargo test` funciona
- [ ] Sei como rodar em Xephyr

## Pronto!

Se todos os itens acima estão ✅, você pode começar:

```bash
# Criar workspace
mkdir -p crates/{de-core,de-x11,de-render,de-mvp}

# Pedir ajuda ao agente
# "Vamos começar o Slice 0, task 1"
```
