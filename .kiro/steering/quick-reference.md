---
inclusion: manual
---

# Quick Reference — Rust DE/WM

Use `#quick-reference` no chat para carregar este guia.

## Comandos Essenciais

```bash
# Build incremental (rápido)
cargo build -p de-core
cargo build -p de-x11
cargo build -p wm

# Build completo
cargo build --workspace --release

# Checks antes de commit
cargo clippy --workspace -- -D warnings
cargo fmt --all
cargo test --workspace

# Rodar componente específico
cargo run --bin de-mvp
cargo run --bin wm
cargo run --bin compositor

# Testar em Xephyr (display virtual)
Xephyr :1 -screen 1920x1080 &
DISPLAY=:1 cargo run --bin wm

# Debug com logs
RUST_LOG=debug cargo run --bin wm
RUST_LOG=de_x11=trace,wm=debug cargo run --bin wm

# Profiling
cargo flamegraph --bin compositor
perf record -g cargo run --release --bin compositor
perf report

# Benchmarks
cargo bench --workspace
cargo bench -p de-render -- render_frame
```

## Padrões de Código Rápidos

### Criar novo error type

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("Something failed: {0}")]
    Failed(String),
    
    #[error("X11 error")]
    X11(#[from] X11Error),
    
    #[error("IO error")]
    Io(#[from] std::io::Error),
}
```

### Criar entity de domínio

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MyEntity {
    id: MyEntityId,
    // campos privados
}

impl MyEntity {
    pub fn new(id: MyEntityId) -> Self {
        Self { id }
    }
    
    // métodos públicos com validação
    pub fn do_something(&mut self, param: Param) -> Result<(), MyError> {
        // validação de domain rules aqui
        Ok(())
    }
}
```

### Criar value object

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MyId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MyValue {
    pub field: u32,
}

impl MyValue {
    pub fn new(field: u32) -> Result<Self, MyError> {
        if field == 0 {
            return Err(MyError::InvalidValue);
        }
        Ok(Self { field })
    }
}
```

### Trait async (SEM async_trait)

```rust
// Rust 1.75+ — nativo
pub trait MyTrait: Send + Sync {
    async fn do_async_thing(&self) -> Result<(), MyError>;
}

// Implementação
impl MyTrait for MyStruct {
    async fn do_async_thing(&self) -> Result<(), MyError> {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
}
```

### IPC message handling

```rust
// Enviar
ipc.send(Message::WindowCreated {
    window_id: WindowId(123),
    geometry: rect,
}).await?;

// Receber (loop)
loop {
    let msg = ipc.receive().await?;
    match msg {
        Message::WindowCreated { window_id, geometry } => {
            // handle
        }
        Message::ConfigReloaded => {
            // reload config
        }
        _ => {}
    }
}

// Request-response
let response = ipc.request(Request::GetConfig {
    key: "window.border_width".to_string(),
}).await?;
```

### Logging com tracing

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument(skip(self))]  // auto-log entrada/saída
pub async fn my_function(&self, window_id: WindowId) -> Result<(), MyError> {
    info!(?window_id, "Starting operation");
    
    match self.do_thing().await {
        Ok(result) => {
            debug!(?result, "Operation succeeded");
            Ok(result)
        }
        Err(e) => {
            error!(error = %e, "Operation failed");
            Err(e)
        }
    }
}
```

### X11 event handling

```rust
// Event loop
loop {
    let event = conn.wait_for_event()?;
    
    match event {
        Event::X(x::Event::MapRequest(e)) => {
            let window = e.window();
            self.manage_window(window).await?;
        }
        Event::X(x::Event::KeyPress(e)) => {
            let keycode = e.detail();
            let modifiers = e.state();
            self.handle_hotkey(keycode, modifiers).await?;
        }
        _ => {}
    }
}
```

### tiny-skia rendering

```rust
use tiny_skia::{Pixmap, Paint, Color, PathBuilder, Transform, FillRule};

let mut pixmap = Pixmap::new(width, height).unwrap();

// Clear
pixmap.fill(Color::from_rgba8(236, 236, 236, 255));

// Rectangle com corner radius
let mut pb = PathBuilder::new();
pb.move_to(x + r, y);
pb.line_to(x + w - r, y);
pb.quad_to(x + w, y, x + w, y + r);
// ... resto do rounded rect
pb.close();
let path = pb.finish().unwrap();

let mut paint = Paint::default();
paint.set_color_rgba8(r, g, b, a);
paint.anti_alias = true;

pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);

// Acessar bytes
let buffer: &[u8] = pixmap.data();
```

### fontdue text rendering

```rust
use fontdue::{Font, FontSettings};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

// Carregar fonte
let font_data = include_bytes!("../assets/fonts/Inter-Regular.ttf");
let font = Font::from_bytes(font_data as &[u8], FontSettings::default()).unwrap();

// Rasterizar glyph
let (metrics, bitmap) = font.rasterize('A', 14.0);

// Copiar para pixmap
for (i, alpha) in bitmap.iter().enumerate() {
    let x = metrics.xmin + (i % metrics.width);
    let y = metrics.ymin + (i / metrics.width);
    // blend alpha no pixmap
}
```

## Debugging Tips

### X11 não conecta
```bash
echo $DISPLAY  # deve ser :0 ou :1
xdpyinfo       # verifica se X11 está rodando
```

### Janela não aparece
```bash
# Verificar se janela foi criada
xwininfo -root -tree

# Verificar propriedades EWMH
xprop -root | grep _NET_
```

### IPC não conecta
```bash
# Verificar se socket existe
ls -la /tmp/de-ipc.sock

# Testar conexão manual
nc -U /tmp/de-ipc.sock
```

### Performance ruim
```bash
# Profiling
cargo flamegraph --bin compositor

# Verificar SIMD
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Memory usage
/usr/bin/time -v cargo run --bin compositor
```

## Atalhos do macOS Sonoma (referência)

| Atalho | Ação |
|--------|------|
| Cmd+Q | Fechar app |
| Cmd+W | Fechar janela |
| Cmd+M | Minimizar |
| Cmd+H | Esconder app |
| Cmd+Tab | Trocar app |
| Cmd+` | Trocar janela do mesmo app |
| Ctrl+Up | Mission Control |
| Ctrl+Down | App Exposé |
| Ctrl+Left/Right | Trocar workspace |

**Mapeamento para Linux:**
- Cmd → Super (tecla Windows)
- Ctrl → Ctrl (mantém)

## Métricas macOS Sonoma (valores exatos)

```rust
// Window decorations
pub const TITLE_BAR_HEIGHT: u32 = 28;
pub const CORNER_RADIUS: f32 = 10.0;

// Traffic lights
pub const TRAFFIC_LIGHT_SIZE: u32 = 12;
pub const TRAFFIC_LIGHT_SPACING: u32 = 8;
pub const TRAFFIC_LIGHT_MARGIN_LEFT: u32 = 8;   // ← 8px, NÃO 20px
pub const TRAFFIC_LIGHT_MARGIN_TOP: u32 = 8;

// Colors (Light mode)
pub const WINDOW_BACKGROUND: Color = Color::rgb(236, 236, 236);  // #ECECEC
pub const CONTENT_BACKGROUND: Color = Color::rgb(242, 242, 247); // #F2F2F7
pub const ACCENT: Color = Color::rgb(0, 122, 255);               // #007AFF

// Colors (Dark mode)
pub const WINDOW_BACKGROUND_DARK: Color = Color::rgb(28, 28, 30);   // #1C1C1E
pub const CONTENT_BACKGROUND_DARK: Color = Color::rgb(44, 44, 46);  // #2C2C2E
pub const ACCENT_DARK: Color = Color::rgb(10, 132, 255);            // #0A84FF

// Panel
pub const PANEL_HEIGHT: u32 = 24;  // macOS menu bar real
pub const PANEL_PADDING: u32 = 8;

// Notifications
pub const NOTIFICATION_WIDTH: u32 = 320;  // ← 320px, NÃO 400px
pub const NOTIFICATION_MARGIN: u32 = 20;
```
