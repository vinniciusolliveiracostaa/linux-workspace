# Troubleshooting — Rust DE/WM

Problemas comuns e soluções durante a implementação.

## Build Errors

### `error: could not find xcb in registry`

**Causa:** Dependências do sistema não instaladas.

**Solução:**
```bash
# Debian/Ubuntu
sudo apt install libxcb1-dev libxcb-xkb-dev libxkbcommon-dev \
                 libxcb-randr0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
                 libxcb-composite0-dev

# Fedora
sudo dnf install libxcb-devel libxkbcommon-devel

# Arch
sudo pacman -S libxcb libxkbcommon
```

### `error: async fn in trait is not supported`

**Causa:** Usando Rust < 1.75 ou tentando usar `async_trait`.

**Solução:**
```bash
# Atualizar Rust
rustup update stable

# Verificar versão
rustc --version  # deve ser >= 1.75

# Remover async_trait do Cargo.toml se existir
```

### `error: bincode 2.x API incompatível`

**Causa:** Cargo resolveu para bincode 2.x ao invés de 1.3.3.

**Solução:**
```toml
# Cargo.toml — pinnar versão exata
bincode = "=1.3.3"  # = força versão exata
```

### `error: fontdue not found`

**Causa:** Ainda usando rusttype no código.

**Solução:**
```rust
// ERRADO
use rusttype::{Font, Scale};

// CORRETO
use fontdue::{Font, FontSettings};
```

## Runtime Errors

### `Error: Cannot connect to X server`

**Causa:** X11 não está rodando ou DISPLAY não está configurado.

**Diagnóstico:**
```bash
echo $DISPLAY          # deve mostrar :0 ou :1
xdpyinfo               # deve listar info do X server
ps aux | grep X        # verificar se X está rodando
```

**Solução:**
```bash
# Se DISPLAY vazio
export DISPLAY=:0

# Se X não está rodando (improvável em desktop Linux)
startx

# Para testes, usar Xephyr
Xephyr :1 -screen 1920x1080 &
export DISPLAY=:1
```

### `Error: Another window manager is already running`

**Causa:** Já existe um WM gerenciando o X server (SubstructureRedirect).

**Diagnóstico:**
```bash
# Ver qual WM está rodando
wmctrl -m
ps aux | grep -E 'openbox|i3|awesome|dwm'
```

**Solução:**
```bash
# Opção 1: Matar WM existente (cuidado!)
killall openbox

# Opção 2: Usar Xephyr (recomendado para testes)
Xephyr :1 -screen 1920x1080 &
DISPLAY=:1 cargo run --bin wm

# Opção 3: Testar em TTY (Ctrl+Alt+F2)
# Fazer login, export DISPLAY=:0, rodar WM
```

### `Error: Permission denied on /tmp/de-ipc.sock`

**Causa:** Socket IPC com permissões erradas ou já existe.

**Solução:**
```bash
# Remover socket antigo
rm /tmp/de-ipc.sock

# Verificar permissões
ls -la /tmp/de-ipc.sock

# Se necessário, ajustar no código
// Rust
std::fs::remove_file("/tmp/de-ipc.sock").ok();
```

### `Panic: unwrap() on None`

**Causa:** Violação da regra "sem unwrap() em produção".

**Solução:**
```rust
// ERRADO
let window = workspace.get_window(id).unwrap();

// CORRETO
let window = workspace.get_window(id)
    .ok_or(WindowError::NotFound(id))?;
```

## X11 Issues

### Janela criada mas não aparece

**Diagnóstico:**
```bash
# Listar janelas
xwininfo -root -tree

# Ver propriedades da janela
xprop -id <window_id>
```

**Causas comuns:**
1. Janela não foi mapeada (`xcb::x::MapWindow`)
2. Janela está fora da tela (geometria negativa)
3. Janela está atrás de outras (stacking order)

**Solução:**
```rust
// Garantir que janela é mapeada
conn.send_request(&x::MapWindow { window });
conn.flush()?;

// Verificar geometria
assert!(geometry.position.x >= 0);
assert!(geometry.position.y >= 0);
```

### Eventos não chegam

**Causa:** Event mask não configurado.

**Solução:**
```rust
// Ao criar janela, definir event mask
let event_mask = x::EventMask::KEY_PRESS
    | x::EventMask::KEY_RELEASE
    | x::EventMask::BUTTON_PRESS
    | x::EventMask::BUTTON_RELEASE
    | x::EventMask::POINTER_MOTION
    | x::EventMask::EXPOSURE
    | x::EventMask::STRUCTURE_NOTIFY;

conn.send_request(&x::CreateWindow {
    // ...
    event_mask,
    // ...
});
```

### EWMH properties não funcionam

**Causa:** Atoms não foram internados ou root window não foi configurado.

**Solução:**
```rust
// Usar xcb-wm ao invés de implementar manualmente
use xcb_wm::ewmh;

let ewmh_conn = ewmh::Connection::connect(&conn)?;
ewmh_conn.set_supported(&[
    ewmh_conn.atoms._NET_SUPPORTED,
    ewmh_conn.atoms._NET_CLIENT_LIST,
    // ...
])?;
```

## Rendering Issues

### Tela preta / nada renderiza

**Diagnóstico:**
```rust
// Adicionar logs
tracing::info!("Rendering frame");
tracing::debug!(?pixmap_size, "Pixmap created");
tracing::debug!(?buffer_len, "Buffer size");
```

**Causas comuns:**
1. Pixmap não foi criado
2. Buffer não foi copiado para X11
3. PutImage com parâmetros errados

**Solução:**
```rust
// Verificar criação do pixmap
let pixmap = Pixmap::new(width, height)
    .ok_or(RenderError::PixmapCreationFailed)?;

// Verificar buffer
let buffer = pixmap.data();
assert_eq!(buffer.len(), (width * height * 4) as usize);

// PutImage correto
conn.send_request(&x::PutImage {
    format: x::ImageFormat::ZPixmap,
    drawable: x::Drawable::Window(window),
    gc,
    width: width as u16,
    height: height as u16,
    dst_x: 0,
    dst_y: 0,
    left_pad: 0,
    depth: 24,
    data: buffer,
});
conn.flush()?;
```

### Texto não aparece

**Causa:** fontdue não está rasterizando ou blending está errado.

**Solução:**
```rust
// Verificar rasterização
let (metrics, bitmap) = font.rasterize('A', 14.0);
tracing::debug!(?metrics, bitmap_len = bitmap.len(), "Rasterized glyph");

// Verificar blending
for (i, alpha) in bitmap.iter().enumerate() {
    if *alpha > 0 {  // só desenhar se alpha > 0
        let x = base_x + metrics.xmin + (i % metrics.width) as i32;
        let y = base_y + metrics.ymin + (i / metrics.width) as i32;
        
        // Blend alpha com cor do texto
        let final_alpha = (*alpha as f32 / 255.0 * text_color.a as f32) as u8;
        let color = Color::rgba(text_color.r, text_color.g, text_color.b, final_alpha);
        
        self.set_pixel(x as u32, y as u32, color);
    }
}
```

## IPC Issues

### Componentes não se comunicam

**Diagnóstico:**
```bash
# Verificar se IPC bus está rodando
ps aux | grep de-ipc

# Verificar socket
ls -la /tmp/de-ipc.sock

# Testar conexão
nc -U /tmp/de-ipc.sock
```

**Solução:**
```rust
// Adicionar logs detalhados
tracing::info!("Connecting to IPC bus");
let client = IpcClient::connect("/tmp/de-ipc.sock", ComponentType::WindowManager)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to connect to IPC bus");
        e
    })?;
tracing::info!("Connected to IPC bus");
```

### Mensagens não chegam

**Causa:** Serialização/deserialização falhando silenciosamente.

**Solução:**
```rust
// Adicionar validação
let serialized = bincode::serialize(&message)
    .map_err(|e| {
        tracing::error!(error = %e, ?message, "Failed to serialize message");
        IpcError::Serialization(e)
    })?;

tracing::debug!(
    message_type = ?message,
    size = serialized.len(),
    "Sending message"
);
```

## Performance Issues

### FPS baixo (<30)

**Diagnóstico:**
```bash
# Profiling
cargo flamegraph --bin compositor

# Verificar CPU usage
top -p $(pgrep compositor)
```

**Causas comuns:**
1. Redesenhando tudo a cada frame (sem dirty regions)
2. Alocações no hot path
3. SIMD não ativado

**Solução:**
```rust
// 1. Usar dirty regions
if !damage_tracker.has_damage() {
    return; // skip frame
}

// 2. Evitar alocações
// ERRADO
let buffer = vec![0u8; width * height * 4];

// CORRETO
self.buffer.clear();
self.buffer.resize(width * height * 4, 0);

// 3. Ativar SIMD
// Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1

// Build
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Memória crescendo (leak)

**Diagnóstico:**
```bash
# Monitorar memória
watch -n 1 'ps aux | grep compositor'

# Valgrind (se disponível)
valgrind --leak-check=full cargo run --bin compositor
```

**Causas comuns:**
1. Pixmaps não sendo liberados
2. Histórico de notificações sem limite
3. Cache de fontes crescendo indefinidamente

**Solução:**
```rust
// Limitar histórico
const MAX_NOTIFICATIONS: usize = 50;
if self.history.len() > MAX_NOTIFICATIONS {
    self.history.remove(0);
}

// Limpar cache periodicamente
if self.font_cache.len() > 100 {
    self.font_cache.clear();
}
```

## Dependency Rule Violations

### Erro: de-core importando xcb

**Causa:** Violação da Dependency Rule.

**Solução:**
```rust
// ERRADO — de-core/src/domain/window.rs
use xcb::x::Window;  // ❌ Domain não pode importar Infrastructure

// CORRETO — usar newtype
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);  // ✅ Domain define seu próprio tipo

// Infrastructure faz a conversão
// de-x11/src/lib.rs
impl From<xcb::x::Window> for WindowId {
    fn from(w: xcb::x::Window) -> Self {
        WindowId(w.resource_id())
    }
}
```

## Checklist de Debug

Quando algo não funciona:

1. ✅ Verificar logs (`RUST_LOG=debug`)
2. ✅ Verificar se X11 está rodando (`xdpyinfo`)
3. ✅ Verificar se janela foi criada (`xwininfo -root -tree`)
4. ✅ Verificar event mask
5. ✅ Verificar se buffer tem tamanho correto
6. ✅ Verificar se IPC socket existe
7. ✅ Verificar permissões de arquivos
8. ✅ Verificar se não há unwrap() falhando
9. ✅ Verificar Dependency Rule
10. ✅ Ler ERRATA.md novamente

## Pedir Ajuda

Se nada funcionar:

```
"Estou no Slice X, task Y. Implementei Z mas está acontecendo W. 
Aqui está o erro: [colar erro completo].
Aqui está o código relevante: [colar código].
O que pode estar errado?"
```

O agente vai:
- Analisar o erro
- Identificar a causa raiz
- Explicar o conceito por trás
- Guiar você para a solução (não resolver diretamente)
