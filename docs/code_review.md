	# 🔍 Revisão de Código — linux-workspace

**Data**: 2026-04-06  
**Status do Projeto**: Slice 0 ✅ + Slice 1 ✅ (pendente teste Xephyr)  
**Compilação**: `cargo check` ✅ | `cargo clippy` ✅ (zero warnings)

---

## 📊 Resumo Executivo

O projeto está **muito bem estruturado** para alguém aprendendo Rust. A separação em camadas (Domain → Application → Presentation) está respeitada, os padrões de design são bem aplicados, e a documentação inline com "POR QUE" é excelente para memória muscular.

Encontrei **7 issues** que precisam ser corrigidas, organizadas por severidade:

| Severidade | Quantidade | Tipo |
|-----------|-----------|------|
| 🔴 Bug | 2 | Vai causar falha em runtime |
| 🟡 Design | 3 | Violação de princípios/padrões |
| 🔵 Melhoria | 2 | Qualidade de código |

---

## 🔴 Bug 1 — `Rectangle::new` bypassa a invariante de `Size`

### Onde
`crates/de-core/src/domain/geometry.rs` — linhas 96-101

### O Problema
```rust
impl Rectangle {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            position: Position { x, y },
            size: Size { width, height }, // ← ACESSO DIRETO a campos privados!
        }
    }
```

Você dedicou tempo a proteger `Size` com campos privados e construtor validado (`Size::new()` que retorna `Option`). Mas `Rectangle::new` **acessa os campos diretamente** porque está no mesmo crate. Em Rust, campos privados são visíveis **dentro do mesmo crate** (não apenas dentro do mesmo módulo — o compilador resolve visibilidade por crate).

Isso significa:
1. `Rectangle::new(0, 0, 0, 0)` cria um `Size` com `width=0, height=0` — **violando a invariante**
2. Todo o trabalho de encapsulamento do `Size` é anulado

### A Mesma Coisa Acontece Em
- `geometry.rs` L130-131 — `intersection()` acessa `self.size.width` diretamente
- `geometry.rs` L146-153 — `offset()` cria `Rectangle` com acesso direto

### Por Que Isso Importa (DDD)
Em DDD, um **Value Object** tem como responsabilidade principal **garantir suas próprias invariantes**. Se outros tipos no mesmo crate podem burlar essas invariantes, o encapsulamento é uma ilusão — funciona para código externo mas não para o próprio domain.

### Correção

```rust
impl Rectangle {
    /// Construtor validado — retorna None se width ou height == 0
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Option<Self> {
        Some(Self {
            position: Position::new(x, y),
            size: Size::new(width, height)?, // ← Respeita a invariante!
        })
    }

    /// Construtor para valores conhecidos (ex: posição do retângulo de interseção)
    /// # Safety: caller garante width > 0 && height > 0
    pub fn new_unchecked(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            position: Position::new(x, y),
            size: Size::new_unchecked(width, height),
        }
    }
}
```

> ⚠️ **BREAKING CHANGE**: Todo lugar que chama `Rectangle::new` precisará migrar.
> Mudar agora custa 15min, mudar depois com 50 chamadores custa 2h.
> Os callers que sabem que os valores são válidos podem usar `new_unchecked`.

---

## 🔴 Bug 2 — `intersection()` usa acesso direto aos campos privados de `Size`

### Onde
`crates/de-core/src/domain/geometry.rs` — linhas 125-144

### O Problema
```rust
pub fn intersection(&self, other: Rectangle) -> Option<Rectangle> {
    // ...
    let self_right = self.position.x + self.size.width as i32;  // ← campo privado!
    let self_bottom = self.position.y + self.size.height as i32; // ← campo privado!
    // ...
    Some(Rectangle::new(x, y, width, height))  // ← Pode criar Size com width=0
}
```

Duas issues aqui:
1. Acessa `self.size.width` direto ao invés de `self.size.width()`
2. A interseção poderia ter `width=0` ou `height=0` se dois retângulos se tocam exatamente na borda

### Correção
```rust
pub fn intersection(&self, other: Rectangle) -> Option<Rectangle> {
    if !self.intersects(other) {
        return None;
    }

    let self_right = self.position.x + self.size.width() as i32;
    let self_bottom = self.position.y + self.size.height() as i32;
    let other_right = other.position.x + other.size.width() as i32;
    let other_bottom = other.position.y + other.size.height() as i32;

    let x = self.position.x.max(other.position.x);
    let y = self.position.y.max(other.position.y);
    let right = self_right.min(other_right);
    let bottom = self_bottom.min(other_bottom);

    let width = (right - x) as u32;
    let height = (bottom - y) as u32;

    // Size::new retorna None se width==0 ou height==0
    // Trata edge case de retângulos que apenas se tocam na borda
    let size = Size::new(width, height)?;
    Some(Rectangle {
        position: Position::new(x, y),
        size,
    })
}
```

---

## 🟡 Design 1 — `client.rs` incompleto e módulo não declarado

### Onde
`crates/de-ipc/src/client.rs` — linhas 28-30

### O Problema
```rust
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{} // ← import vazio, arquivo incompleto
```

O `lib.rs` do `de-ipc` **não** declara `pub mod client`, então o arquivo existe mas **não compila**.
Alguém olhando o diretório acha que existe um client implementado.

### Correção
**Deletar `client.rs`** e criar quando implementar o Slice 2.

---

## 🟡 Design 2 — `EventDispatcher` tem dois blocos `impl` separados sem necessidade

### Onde
`crates/wm/src/presentation/event_dispatcher.rs` — linhas 353-409

### O Problema
```rust
// Comentário ERRADO:
// 6. Impl EventHandler — TODOS os handlers + métodos privados auxiliares
impl<'a> EventDispatcher<'a> {
    fn execute_hotkey_action(&mut self, action: HotkeyAction) { ... }
}
```

Esse bloco NÃO é `impl EventHandler`, é um `impl EventDispatcher` normal.

### Correção
Mover `execute_hotkey_action` para o primeiro `impl EventDispatcher` (junto de `new` e `register_as_wm`).

---

## 🟡 Design 3 — `manage_window` cria `Size` com possibilidade de panic

### Onde
`crates/wm/src/application/window_manager_service.rs` — linhas 66-79

### O Problema
```rust
let requested_size = Size::new(window_size.0, window_size.1)
    .unwrap_or_else(|| Size::new_unchecked(window_size.0, window_size.1));
    //                                      ^^^^^^^^^^ pode ser 0! → debug_assert panic
```

### Correção
```rust
let requested = Size::new_unchecked(
    window_size.0.max(1),
    window_size.1.max(1),
);
```

---

## 🔵 Melhoria 1 — `focus_window` no Workspace tem lógica duplicada

### Onde
`crates/de-core/src/domain/workspace.rs` — linhas 79-98

### Correção
```rust
pub fn focus_window(&mut self, window_id: WindowId) -> bool {
    if !self.windows.contains_key(&window_id) {
        return false;
    }

    for window in self.windows.values_mut() {
        window.is_focused = window.id == window_id;
    }

    self.focused_window = Some(window_id);
    true
}
```

---

## 🔵 Melhoria 2 — `X11EventLoop::subclassify_root` duplica `X11Adapter::register_as_wm`

### Onde
- `crates/de-x11/src/events.rs` L43-65
- `crates/wm/src/presentation/x11_adapter.rs` L30-56

### Correção
Deletar `subclassify_root` do `X11EventLoop` — código morto que duplica lógica da camada acima.

---

## ✅ O Que Está Bem Feito

### Arquitetura
- ✅ **Dependency Rule** correta: `de-core` não depende de `de-x11`/`de-render`
- ✅ **Separação clara** Domain → Application → Presentation
- ✅ **Ports/Interfaces** preparados para inversão de dependência

### Padrões de Design
- ✅ Strategy Pattern (PlacementStrategy)
- ✅ State Machine (GrabState)
- ✅ Command Pattern (HotkeyAction)
- ✅ Adapter Pattern (X11Adapter)
- ✅ Newtype Pattern (WindowId, WorkspaceId)

### Rust Idiomático
- ✅ Value Objects com invariantes
- ✅ Error handling com `thiserror` + `Result`
- ✅ `const fn` onde possível
- ✅ `impl Drop` no `IpcBus`

### Documentação
- ✅ Comentários "POR QUE"
- ✅ Doc comments nos módulos
- ✅ Referências a specs
