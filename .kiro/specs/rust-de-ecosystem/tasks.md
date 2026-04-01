# Plano de Implementação: Rust DE/WM Ecosystem

## Visão Geral

Implementação incremental em 12 slices, do MVP até o ecossistema completo.
Linguagem: **Rust** (stable toolchain).
Cada task é estimada em 1–4h de trabalho solo.
Testes são escritos **após** a implementação de cada slice.

---

## Slice 0 — Foundation MVP

- [x] 1. Setup do Cargo Workspace
  - Criar `Cargo.toml` raiz com `[workspace]` listando todos os membros
  - Criar estrutura de pastas: `crates/de-core`, `crates/de-x11`, `crates/de-render`, `crates/de-mvp`
  - Criar `Cargo.toml` mínimo para cada crate membro
  - Adicionar `.gitignore` e `README.md` básico
  - _Requirements: 20.1, 18.3_

- [x] 2. de-core — Domain básico
  - [x] 2.1 Implementar value objects de geometria
    - Criar `crates/de-core/src/domain/geometry.rs` com `Position`, `Size`, `Rectangle`
    - Implementar métodos: `contains_point`, `intersects`, `intersection`, `offset`
    - Derivar `Debug, Clone, Copy, PartialEq, Eq`
    - _Requirements: 1.1, 2.7_
  - [x] 2.2 Implementar `Color` e `WindowId`
    - Criar `crates/de-core/src/domain/color.rs` com `Color { r, g, b, a }`
    - Criar `WindowId(pub u32)` como newtype em `domain/window.rs`
    - _Requirements: 6.1_
  - [x] 2.3 Implementar `Window` entity mínima
    - Struct `Window` com `id`, `geometry`, `state`, `is_focused`, `is_decorated`
    - Métodos: `new`, `move_to`, `resize` (com constraint), `set_focused`
    - _Requirements: 1.1, 1.3, 1.4_
  - [x] 2.4 Implementar `CoreError` e re-exports em `lib.rs`
    - Criar `crates/de-core/src/error.rs` com `CoreError` usando `thiserror`
    - Re-exportar todos os tipos públicos em `lib.rs`
    - _Requirements: 16.3_
  - [ ]* 2.5 Escrever property tests para geometria
    - **Property 1: `rectangle.intersection` é comutativa**
    - **Validates: Requirements 2.7**
    - **Property 2: `window.resize` sempre respeita `min_size`**
    - **Validates: Requirements 1.4**


- [x] 3. de-x11 — Conexão X11 básica
  - [x] 3.1 Implementar `X11Connection`
    - Criar `crates/de-x11/src/connection.rs`
    - Conectar ao X11 server via `xcb::Connection::connect`
    - Expor `root_window()` e `screen_geometry() -> Rectangle`
    - Criar `X11Error` com `thiserror`
    - _Requirements: 13.7_
  - [x] 3.2 Implementar criação de janela e event loop básico
    - Criar janela com `xcb::x::CreateWindow`
    - Mapear janela com `xcb::x::MapWindow`
    - Event loop que trata `KeyPress` (ESC fecha) e `Expose`
    - _Requirements: 13.4_

- [x] 4. de-render — Software Renderer básico
  - [x] 4.1 Implementar `FrameBuffer` e `SoftwareRenderer`
    - Criar `crates/de-render/src/buffer.rs` com gerenciamento de `Pixmap` (tiny-skia)
    - Criar `crates/de-render/src/renderer.rs` com `SoftwareRenderer`
    - Métodos: `new(width, height)`, `clear(color)`, `draw_rectangle(rect, color, radius)`
    - _Requirements: 2.1, 14.2_
  - [x] 4.2 Implementar `PutImage` para X11
    - Método `get_buffer() -> &[u8]` expõe bytes do pixmap
    - Enviar buffer para X11 via `xcb::x::PutImage`
    - _Requirements: 2.1_

- [x] 5. de-mvp — Integração e binário MVP
  - Criar `crates/de-mvp/src/main.rs`
  - Conectar `X11Connection` + `SoftwareRenderer`
  - No evento `Expose`: renderizar retângulo azul 400x300 centralizado e chamar `PutImage`
  - No evento `KeyPress ESC`: fechar janela e sair
  - _Requirements: 2.1, 13.4_

- [x] 6. Checkpoint Slice 0
  - `cargo build --workspace` sem warnings
  - `cargo clippy --workspace -- -D warnings` sem erros
  - `cargo run --bin de-mvp` abre janela 800x600 com retângulo azul, ESC fecha


---

## Slice 1 — Window Manager Básico

- [x] 7. EWMH/ICCCM no de-x11
  - [x] 7.1 Implementar `AtomCache`
    - Criar `crates/de-x11/src/atoms.rs` com todos os atoms necessários
    - Internear atoms no startup via `xcb::x::InternAtom`
    - _Requirements: 13.1, 13.2_
  - [x] 7.2 Implementar `Ewmh`
    - Criar `crates/de-x11/src/ewmh.rs`
    - Métodos: `set_supported`, `set_client_list`, `set_active_window`, `set_number_of_desktops`, `set_current_desktop`, `get_window_name`
    - _Requirements: 13.1_
  - [x] 7.3 Implementar `Icccm`
    - Criar `crates/de-x11/src/icccm.rs`
    - Ler `WM_PROTOCOLS`, `WM_HINTS`, `WM_NORMAL_HINTS`, `WM_CLASS`
    - Enviar `WM_DELETE_WINDOW` ao fechar janela
    - _Requirements: 13.2_

- [x] 8. Event loop completo no de-x11
  - Criar `crates/de-x11/src/events.rs` com trait `EventHandler`
  - Implementar `X11EventLoop::run` tratando: `MapRequest`, `UnmapNotify`, `ConfigureRequest`, `DestroyNotify`, `ButtonPress`, `ButtonRelease`, `MotionNotify`, `KeyPress`, `EnterNotify`, `PropertyNotify`
  - Subclassificar root window para receber `SubstructureRedirect`
  - _Requirements: 13.3, 13.4_

- [x] 9. Domain: `Workspace` e `WorkspaceId`
  - Criar `crates/de-core/src/domain/workspace.rs` completo
  - Struct com `windows: HashMap<WindowId, Window>`, `window_stack: Vec<WindowId>`, `focused_window: Option<WindowId>`
  - Métodos: `add_window`, `remove_window`, `focus_window`, `raise_window`, `lower_window`, `windows_in_stack_order`
  - _Requirements: 11.1, 11.3_
  - [ ]* 9.1 Escrever property tests para `Workspace`
    - **Property 3: `window_stack` e `windows.keys()` sempre têm os mesmos IDs**
    - **Validates: Requirements 11.1**
    - **Property 4: após `focus_window`, exatamente uma janela tem `is_focused == true`**
    - **Validates: Requirements 1.6**

- [ ] 10. Window Manager core (`wm` crate)
  - [ ] 10.1 Criar estrutura do crate `wm`
    - `crates/wm/src/main.rs` com inicialização
    - `crates/wm/src/manager.rs` com struct `WindowManager`
    - Campos: `workspaces`, `current_workspace`, `window_to_workspace`, `x11`, `config`
    - _Requirements: 1.1_
  - [ ] 10.2 Implementar `manage_window` e `unmanage_window`
    - `manage_window`: obtém geometria do X11, cria `Window`, aplica placement, adiciona ao workspace
    - `unmanage_window`: remove do workspace, atualiza EWMH client list
    - _Requirements: 1.2, 13.3_
  - [ ] 10.3 Implementar placement strategies
    - Criar `crates/wm/src/placement.rs`
    - Estratégias: `Center`, `Smart` (minimiza overlap), `Cascade`
    - _Requirements: 1.2_
  - [ ] 10.4 Implementar `move_window` e `resize_window`
    - Atualizar domain + X11 + notificar via log (IPC vem no Slice 2)
    - _Requirements: 1.3, 1.4_
  - [ ] 10.5 Implementar focus management (click-to-focus)
    - `focus_window`: atualiza domain, `SetInputFocus` no X11, atualiza `_NET_ACTIVE_WINDOW`
    - Tratar `EnterNotify` para focus-follows-mouse (opcional, configurável)
    - _Requirements: 1.6_

- [ ] 11. Mouse grab para move e resize
  - Criar `crates/wm/src/grab.rs`
  - `Super+Mouse1` inicia move: gravar posição inicial, tratar `MotionNotify`
  - `Super+Shift+Mouse1` inicia resize: calcular delta, chamar `resize_window`
  - Liberar grab no `ButtonRelease`
  - _Requirements: 1.3, 1.4_

- [ ] 12. Hotkey Manager básico
  - Criar `crates/wm/src/hotkeys.rs` com `HotkeyManager`
  - Structs `Modifiers`, `Hotkey`, enum `Action`
  - `register`: valida conflito, chama `xcb::x::GrabKey`
  - Default bindings: `Super+Q` fecha janela, `Super+F` maximiza
  - _Requirements: 8.1, 8.2, 8.5, 8.6_

- [ ] 13. Checkpoint Slice 1
  - Testar com Xephyr: `Xephyr :1 -screen 1920x1080 & DISPLAY=:1 cargo run --bin wm`
  - Abrir `xterm` no Xephyr — deve ser gerenciado pelo WM
  - `Super+Q` deve fechar a janela
  - `Super+Mouse1` deve mover a janela


---

## Slice 2 — IPC Bus + Multi-Process

- [ ] 14. de-ipc — Protocolo e tipos
  - Criar `crates/de-ipc/src/protocol.rs` com enum `Message` completo
  - Todas as variantes: window events, workspace events, config events, theme events, notification events, system events, heartbeat
  - Enum `ComponentType`, `Request`, `Response`
  - Derivar `Serialize, Deserialize` com serde + bincode
  - _Requirements: 7.1, 7.6, 7.7_

- [ ] 15. de-ipc — Server (IpcBus)
  - Criar `crates/de-ipc/src/server.rs` com `IpcBus`
  - `new(socket_path)`: bind `UnixListener`, remover socket antigo se existir
  - `accept_client`: aguarda conexão, lê identificação do componente
  - `send(target, message)`: serializa com bincode (4 bytes len + payload), envia
  - `broadcast(message)`: envia para todos os clientes conectados
  - `receive(from)`: lê len + payload, deserializa
  - _Requirements: 7.2, 7.3, 7.4_

- [ ] 16. de-ipc — Client (IpcClient)
  - Criar `crates/de-ipc/src/client.rs` com `IpcClient`
  - `connect(socket_path, component_type)`: conecta, envia identificação
  - `send(message)`, `receive()`, `try_receive()` (non-blocking)
  - `request(req) -> Response`: envia `Message::Request`, aguarda `Message::Response` com mesmo ID
  - Geração de request IDs com `AtomicU64`
  - _Requirements: 7.2, 7.3_

- [ ] 17. Heartbeat e detecção de desconexão
  - Criar `crates/de-ipc/src/heartbeat.rs`
  - `IpcClient` envia `Message::Heartbeat` a cada 5s em background task
  - `IpcBus` detecta timeout (>15s sem heartbeat) e notifica com `ComponentCrashed`
  - _Requirements: 7.5_
  - [ ]* 17.1 Escrever testes de integração para IPC
    - Testar send/receive round-trip com dois processos
    - Testar detecção de desconexão
    - _Requirements: 7.3, 7.5_

- [ ] 18. Integrar IPC no Window Manager
  - Adicionar `IpcClient` ao `WindowManager`
  - Enviar `Message::WindowCreated/Destroyed/Moved/Resized/Focused` nos respectivos métodos
  - Enviar `Message::WorkspaceChanged` no `switch_workspace`
  - _Requirements: 7.1_

- [ ] 19. Componente dummy para validar multi-process
  - Criar `crates/de-dummy/src/main.rs`
  - Conecta ao IPC bus, recebe mensagens, loga no stdout
  - Valida que WM e dummy rodam como processos separados e se comunicam
  - _Requirements: 7.1, 7.4_

- [ ] 20. Checkpoint Slice 2
  - Rodar IPC bus + WM + dummy em terminais separados
  - Mover janela no Xephyr → dummy deve logar `WindowMoved`
  - Desconectar dummy → WM não deve crashar


---

## Slice 3 — Compositor CPU

- [ ] 21. Scene graph
  - Criar `crates/compositor/src/scene.rs` com `SceneGraph`
  - `WindowNode { window_id, geometry, is_decorated, is_visible }`
  - Métodos: `add_window`, `remove_window`, `move_window`, `resize_window`, `raise_window`
  - `layers()`: retorna janelas em stacking order (bottom to top)
  - _Requirements: 2.1, 2.6_

- [ ] 22. Damage tracking
  - Criar `crates/compositor/src/damage.rs` com `DamageTracker`
  - `mark_dirty(rect)`: tenta mesclar com regiões existentes via `merge_rectangles`
  - `mark_all_dirty()`: full damage flag
  - `has_damage()`, `dirty_regions()`, `intersects_dirty(rect)`, `clear()`
  - _Requirements: 2.7, 14.2_
  - [ ]* 22.1 Escrever property tests para `DamageTracker`
    - **Property 5: após `mark_all_dirty`, `intersects_dirty` retorna `true` para qualquer rect**
    - **Validates: Requirements 2.7**

- [ ] 23. VSync timer-based
  - Criar `crates/compositor/src/vsync.rs` com `VSync`
  - `wait()`: dorme até próximo frame (target 16.667ms / 60 FPS)
  - `set_refresh_rate(hz)`: configura target frame time
  - _Requirements: 2.4_

- [ ] 24. Compositor core
  - Criar `crates/compositor/src/compositor.rs` com `Compositor`
  - `run()`: loop de vsync → processar mensagens IPC → renderizar se há damage
  - `handle_message`: atualiza scene graph e damage tracker para cada `Message`
  - `render_frame()`: limpa dirty regions, renderiza scene em ordem, apresenta ao X11
  - _Requirements: 2.1, 2.3_

- [ ] 25. Window decorations (macOS Sonoma style)
  - Criar `crates/compositor/src/decorations.rs`
  - `render_decoration(window_node, renderer, theme)`:
    - Title bar com `corner_radius = 10px`
    - Traffic lights: vermelho (close) 12px, amarelo (minimize) 12px, verde (maximize) 12px, espaçamento 8px, margem esquerda 20px
    - Título da janela centralizado na title bar
  - _Requirements: 12.1, 12.2, 12.6_

- [ ] 26. Sombras sem GPU
  - Implementar `draw_shadow` no `SoftwareRenderer`
  - Múltiplas camadas de retângulos com alpha decrescente (5 camadas)
  - Offset de 1px por camada
  - _Requirements: 2.2_

- [ ] 27. Checkpoint Slice 3
  - Compositor rodando como processo separado
  - Janelas abertas no Xephyr devem ter decorações (title bar + traffic lights)
  - Mover janela deve atualizar compositor via IPC sem tearing visível


---

## Slice 4 — Panel + System Tray

- [ ] 28. Panel core
  - Criar `crates/panel/src/panel.rs` com `Panel` e `PanelPosition`
  - `calculate_geometry(position, screen, theme)`: calcula rect do panel
  - `create_panel_window()`: cria janela X11 com `_NET_WM_WINDOW_TYPE_DOCK`, configura strut, `_NET_WM_STATE_ABOVE`
  - `run()`: loop com `tokio::select!` entre IPC e timer de 100ms
  - _Requirements: 3.1, 3.5_

- [ ] 29. Widget system
  - Criar `crates/panel/src/widgets/mod.rs` com trait `Widget`
  - Métodos: `alignment()`, `preferred_width(theme)`, `update()`, `render(renderer, rect, theme)`, `handle_click(pos)`
  - Enum `Alignment { Left, Center, Right }`
  - _Requirements: 3.1_

- [ ] 30. Widgets essenciais
  - [ ] 30.1 `ClockWidget`: formata hora com `chrono`, alinhamento direito
    - _Requirements: 3.1, 3.7_
  - [ ] 30.2 `WorkspaceWidget`: indicadores circulares, clique troca workspace via IPC
    - _Requirements: 3.4, 11.5_
  - [ ] 30.3 `WindowTitleWidget`: exibe título da janela focada, atualiza via IPC
    - _Requirements: 3.4_
  - [ ] 30.4 `BatteryWidget`: lê `/sys/class/power_supply/BAT0/capacity`
    - _Requirements: 3.1_
  - [ ] 30.5 `NetworkWidget`: lê estado de `/sys/class/net/<iface>/operstate`
    - _Requirements: 3.1_

- [ ] 31. Layout engine do panel
  - Implementar `calculate_layout()` em `Panel`
  - Widgets `Left` alinhados à esquerda, `Center` centralizados, `Right` à direita
  - Respeitar `panel.padding` e `panel.widget_spacing` do tema
  - _Requirements: 3.3_

- [ ] 32. Integração IPC no Panel
  - Tratar `Message::WindowFocused` → atualiza `WindowTitleWidget`
  - Tratar `Message::WorkspaceChanged` → atualiza `WorkspaceWidget`
  - Tratar `Message::ThemeChanged` → recarrega tema e força redraw
  - _Requirements: 3.6, 3.7_

- [ ] 33. Checkpoint Slice 4
  - Panel visível no topo da tela no Xephyr
  - Clock atualiza a cada minuto
  - Workspace indicator muda ao trocar workspace com `Super+1..4`
  - Título da janela atualiza ao focar janelas diferentes


---

## Slice 5 — Launcher

- [ ] 34. Indexador de aplicações
  - Criar `crates/launcher/src/indexer.rs`
  - Buscar arquivos `.desktop` em `/usr/share/applications`, `/usr/local/share/applications`, `~/.local/share/applications`
  - Parser de `.desktop`: extrair `Name`, `Exec`, `Icon`, `Comment`, `Categories`, `NoDisplay`
  - Ignorar entradas com `NoDisplay=true`
  - _Requirements: 4.2, 4.7_

- [ ] 35. Cache de aplicações
  - Serializar lista de `Application` para `~/.cache/rust-de/launcher.cache` com bincode
  - Carregar cache no startup se mais novo que 1h
  - Invalidar cache quando diretórios de `.desktop` mudam (via `mtime`)
  - _Requirements: 4.7_

- [ ] 36. Fuzzy search
  - Criar `crates/launcher/src/search.rs`
  - Usar `fuzzy-matcher` crate com `SkimMatcherV2`
  - `search(query, apps) -> Vec<(Application, i64)>`: retorna apps ordenados por score
  - Priorizar apps usados recentemente (boost no score)
  - _Requirements: 4.3, 4.5_

- [ ] 37. Launcher UI
  - Criar `crates/launcher/src/ui.rs`
  - Janela popup centralizada 600x400px com `corner_radius = 10px`
  - Campo de busca no topo com cursor de texto
  - Lista de resultados com scroll (máx 8 visíveis)
  - Item selecionado destacado com cor de accent do tema
  - _Requirements: 4.1, 4.3_

- [ ] 38. Launcher core e hotkey global
  - Criar `crates/launcher/src/launcher.rs` com `Launcher`
  - `show()` / `hide()`: mapear/desmapar janela X11
  - Tratar `KeyPress`: caracteres adicionam ao query, `Backspace` remove, `Enter` lança app, `Escape` fecha, `Up/Down` navega lista
  - Registrar `Super+Space` como hotkey global no WM via IPC
  - _Requirements: 4.1, 4.4, 4.6_

- [ ] 39. Lançamento de aplicações
  - `launch(app)`: processar `Exec` field (remover `%f`, `%u`, `%F`, `%U`), `Command::new().spawn()`
  - Atualizar `last_used` timestamp no cache
  - Fechar launcher após lançar
  - _Requirements: 4.4, 4.6_

- [ ] 40. Checkpoint Slice 5
  - `Super+Space` abre launcher em <100ms
  - Digitar "fire" deve mostrar Firefox (se instalado) ou similar
  - `Enter` lança a aplicação selecionada
  - `Escape` fecha o launcher


---

## Slice 6 — Config Manager + Hot-reload

- [ ] 41. de-config — Parser TOML
  - Criar `crates/de-config/src/parser.rs`
  - Usar `toml` crate para deserializar em `toml::Value`
  - Suporte a `include = ["other.toml"]` para configs modulares
  - Reportar número de linha em erros de sintaxe
  - _Requirements: 15.1, 15.4, 15.6_

- [ ] 42. Schema e validação
  - Criar `crates/de-config/src/schema.rs`
  - Validar campos obrigatórios com mensagens descritivas
  - Suporte a expansão de variáveis de ambiente (`${HOME}`, `${XDG_CONFIG_HOME}`)
  - _Requirements: 5.3, 5.4, 15.3, 15.7_

- [ ] 43. Defaults sensatos
  - Criar `crates/de-config/src/defaults.rs`
  - Gerar config padrão completa se arquivo não existir
  - Incluir comentários inline explicativos nos arquivos gerados
  - _Requirements: 15.2, 15.5_

- [ ] 44. Config Manager core
  - Criar `crates/de-config/src/manager.rs` com `ConfigManager`
  - `get<T: DeserializeOwned>(key: &str) -> Result<T>`: acesso por chave dotted (`window.border_width`)
  - `set<T: Serialize>(key, value)`: atualiza valor em memória
  - Manter última config válida em caso de erro de parse
  - _Requirements: 5.1, 5.3, 5.4_

- [ ] 45. File watcher e hot-reload
  - Criar `crates/de-config/src/watcher.rs`
  - Usar `notify` crate para monitorar mudanças nos arquivos de config
  - Ao detectar mudança: re-parsear, validar, e se válido aplicar + broadcast via IPC `Message::ConfigReloaded`
  - Se inválido: logar erro, manter config anterior
  - _Requirements: 5.2, 5.5_

- [ ] 46. Integrar Config Manager nos componentes
  - WM lê `wm.toml`: workspace_count, placement_strategy, hotkeys
  - Panel lê `panel.toml`: position, widgets habilitados
  - Cada componente recarrega config ao receber `Message::ConfigReloaded`
  - _Requirements: 5.5, 5.6, 5.7, 8.3, 8.7_

- [ ] 47. Checkpoint Slice 6
  - Editar `wm.toml` mudando `workspace_count` → WM deve recarregar sem restart
  - Config inválida (TOML quebrado) → componente mantém config anterior e loga erro
  - `${HOME}` em paths deve ser expandido corretamente


---

## Slice 7 — Theme Engine + macOS Sonoma Theme

- [ ] 48. de-theme — Structs de tema
  - Criar `crates/de-theme/src/theme.rs` com `Theme`, `ColorScheme`, `Typography`, `Spacing`, `WindowDecorationStyle`, `PanelStyle`
  - Todos os campos com `Serialize, Deserialize`
  - Trait `ThemeProvider` com `current_theme() -> &Theme`
  - _Requirements: 6.1, 6.5_

- [ ] 49. macOS Sonoma Light theme
  - Criar `crates/de-theme/src/macos_sonoma.rs`
  - Valores exatos do Sonoma Light:
    - Background: `#ECECEC`, Foreground: `#1C1C1E`, Accent: `#007AFF`
    - Traffic lights: close `#FF5F57`, minimize `#FEBC2E`, maximize `#28C840`
    - Title bar height: 28px, corner radius: 10px, button size: 12px, spacing: 8px, margem: 20px
    - Font: "SF Pro" (fallback "Inter"), size: 13px
  - _Requirements: 6.1, 6.4, 12.6_

- [ ] 50. macOS Sonoma Dark theme
  - Valores do Sonoma Dark:
    - Background: `#1C1C1E`, Foreground: `#F2F2F7`, Accent: `#0A84FF`
    - Mesmos traffic lights (não mudam no dark mode)
  - _Requirements: 6.1, 6.4_

- [ ] 51. Carregamento de tema via TOML
  - `ThemeManager::load_from_file(path)`: deserializa `Theme` de arquivo TOML
  - Validar campos obrigatórios antes de ativar
  - Suporte a herança: `extends = "sonoma-light"` sobrescreve apenas campos especificados
  - _Requirements: 6.3, 6.6, 6.7_

- [ ] 52. Propagação de tema via IPC
  - `ThemeManager::apply(theme_name)`: salva tema ativo, envia `Message::ThemeChanged`
  - Cada componente ao receber `ThemeChanged`: recarrega tema e força redraw completo
  - _Requirements: 6.2_

- [ ] 53. Checkpoint Slice 7
  - Trocar tema via config → todos os componentes atualizam visualmente
  - Decorações de janela devem usar valores exatos do Sonoma (traffic lights 12px, margem 20px)
  - Tema customizado em TOML com `extends` deve funcionar


---

## Slice 8 — Notifications + D-Bus

- [ ] 54. D-Bus interface (freedesktop.org spec)
  - Criar `crates/notifications/src/dbus.rs`
  - Usar `zbus` crate para implementar `org.freedesktop.Notifications`
  - Métodos: `Notify`, `CloseNotification`, `GetCapabilities`, `GetServerInformation`
  - Registrar nome `org.freedesktop.Notifications` no session bus
  - _Requirements: 9.1_

- [ ] 55. Notification Manager core
  - Criar `crates/notifications/src/manager.rs` com `NotificationManager`
  - Struct `Notification { id, summary, body, urgency, timeout, actions, timestamp }`
  - `receive(notification)`: atribuir ID único, adicionar à fila, enviar `Message::NotificationReceived` via IPC
  - Auto-dismiss: `tokio::time::sleep(timeout)` → `close(id, Expired)`
  - _Requirements: 9.2, 9.3, 9.4_

- [ ] 56. Histórico de notificações
  - Criar `crates/notifications/src/history.rs`
  - Manter últimas 50 notificações em memória
  - Responder a `Request::GetNotificationHistory` via IPC
  - _Requirements: 9.6_

- [ ] 57. Notification UI
  - Criar `crates/notifications/src/ui.rs`
  - Popup no canto superior direito (estilo macOS): 320px largura, padding 16px
  - Stack de notificações com espaçamento 8px entre elas
  - Urgency `Critical`: borda vermelha, sem auto-dismiss
  - Urgency `Low`: alpha reduzido
  - Animação de entrada: slide-in da direita (sem GPU — interpolação linear em 150ms)
  - _Requirements: 9.2, 9.3, 9.7_

- [ ] 58. Interação com notificações
  - Clique na notificação: invocar action padrão, enviar `NotificationActionInvoked` via IPC, fechar
  - Botão X: fechar notificação (`CloseReason::DismissedByUser`)
  - _Requirements: 9.5_

- [ ] 59. Checkpoint Slice 8
  - `notify-send "Teste" "Mensagem"` deve exibir popup no canto superior direito
  - Notificação deve desaparecer após timeout configurado
  - Notificação `critical` não deve desaparecer automaticamente


---

## Slice 9 — Session Manager + Crash Recovery

- [ ] 60. Definir ordem de startup
  - Criar `crates/session/src/startup.rs`
  - Ordem: `de-ipc-bus` → `de-config` → `de-theme` → `de-x11` → `wm` → `compositor` → `panel` → `launcher` → `notifications`
  - Cada componente aguarda confirmação via `Message::ComponentStarted` antes de iniciar o próximo
  - Timeout de 5s por componente
  - _Requirements: 10.1, 10.2_

- [ ] 61. Health monitoring
  - Criar `crates/session/src/monitor.rs`
  - Monitorar heartbeats de todos os componentes via IPC
  - Detectar ausência de heartbeat por >15s como crash
  - _Requirements: 10.3, 17.1_

- [ ] 62. Crash recovery com circuit breaker
  - Criar `crates/session/src/recovery.rs`
  - `restart_component(component)`: re-lança processo com mesma config
  - Circuit breaker: contar crashes por componente nos últimos 60s
  - Se >3 crashes em 60s: desabilitar componente, logar, enviar notificação ao usuário
  - _Requirements: 10.4, 17.2, 17.3_

- [ ] 63. Salvar e restaurar estado de sessão
  - Criar `crates/session/src/session.rs`
  - Ao shutdown: serializar `Vec<WindowState>` (posição, workspace, título) para `~/.local/share/rust-de/session.json`
  - Ao startup: restaurar posições de janelas se arquivo existir
  - _Requirements: 10.6, 17.4, 17.6_

- [ ] 64. Graceful shutdown
  - `SIGTERM` → enviar `Message::ComponentStopped` para todos → aguardar confirmação (timeout 3s) → `SIGKILL` restantes
  - Salvar estado antes de encerrar
  - _Requirements: 10.5_

- [ ] 65. Notificação de crash ao usuário
  - Ao detectar crash: enviar notificação via `NotificationManager` com urgency `Critical`
  - Mensagem: "Componente X falhou e foi reiniciado" ou "Componente X desabilitado após 3 falhas"
  - _Requirements: 17.7_

- [ ] 66. Checkpoint Slice 9
  - Matar processo `compositor` com `kill -9` → Session Manager deve reiniciá-lo em <1s
  - Matar compositor 3x em 60s → deve ser desabilitado com notificação
  - `loginctl terminate-session` deve encerrar tudo graciosamente


---

## Slice 10 — Workspaces + Hotkeys Avançados

- [ ] 67. Múltiplos workspaces no WM
  - Expandir `WindowManager` para suportar N workspaces (configurável, default 4)
  - `switch_workspace(id)`: unmap janelas do workspace atual, map janelas do novo
  - Atualizar `_NET_CURRENT_DESKTOP` e `_NET_NUMBER_OF_DESKTOPS` via EWMH
  - _Requirements: 11.1, 11.2_

- [ ] 68. Mover janela entre workspaces
  - `move_window_to_workspace(window_id, workspace_id)`: remove do workspace atual, adiciona ao destino
  - Se destino não é o workspace ativo: unmap a janela
  - Enviar `Message::WindowMovedToWorkspace` via IPC
  - _Requirements: 11.3_

- [ ] 69. Persistência de workspace assignments
  - Salvar `HashMap<WindowId, WorkspaceId>` no estado de sessão
  - Restaurar assignments ao reabrir sessão
  - _Requirements: 11.4_

- [ ] 70. Hotkeys de workspace
  - Registrar `Super+1..4`: `switch_workspace(N)`
  - Registrar `Super+Shift+1..4`: `move_window_to_workspace(focused, N)`
  - Registrar `Super+Tab`: ciclar janelas no workspace atual
  - _Requirements: 11.6, 8.1, 8.5_

- [ ] 71. Hotkeys configuráveis via TOML
  - Expandir `HotkeyManager::load_from_config` para ler `[hotkeys]` do `wm.toml`
  - Formato: `"Super+Q" = "close_window"`, `"Super+1" = "switch_workspace:1"`
  - Validar conflitos ao carregar, logar warnings para bindings duplicados
  - Hot-reload: ao receber `ConfigReloaded`, desregistrar todos e re-registrar
  - _Requirements: 8.3, 8.4, 8.7_

- [ ] 72. Workspace indicator no Panel
  - `WorkspaceWidget` deve mostrar N indicadores (N = workspace_count da config)
  - Clique em indicador envia `Request::SwitchWorkspace(N)` via IPC
  - _Requirements: 11.5_

- [ ] 73. Checkpoint Slice 10
  - `Super+2` muda para workspace 2, janelas do workspace 1 somem
  - `Super+Shift+2` move janela focada para workspace 2
  - Workspace indicator no panel reflete workspace atual
  - Hotkey customizada em `wm.toml` funciona após hot-reload


---

## Slice 11 — Polish + Performance + Testes

- [ ] 74. Testes de integração completos
  - [ ] 74.1 Setup de infraestrutura de testes
    - Helper `TestEnvironment`: inicia Xvfb, IPC bus, WM em background
    - Cleanup automático no `Drop`
    - _Requirements: 20.2_
  - [ ]* 74.2 Testes de integração do Window Manager
    - Testar `manage_window`, `move_window`, `resize_window`, `focus_window`
    - Testar `switch_workspace` com múltiplas janelas
    - _Requirements: 1.1–1.7_
  - [ ]* 74.3 Testes de integração do IPC
    - Round-trip de todas as variantes de `Message`
    - Testar request-response com timeout
    - _Requirements: 7.1–7.7_
  - [ ]* 74.4 Testes de integração do Config Manager
    - Testar hot-reload com arquivo modificado em disco
    - Testar fallback para config anterior em caso de erro
    - _Requirements: 5.1–5.7_

- [ ] 75. Property-based tests com proptest
  - [ ]* 75.1 Property tests para `Rectangle`
    - **Property 1: `intersection` é comutativa**
    - **Property 6: `intersection` de rect consigo mesmo é o próprio rect**
    - **Validates: Requirements 2.7, 14.2**
  - [ ]* 75.2 Property tests para `Window`
    - **Property 2: `resize` sempre respeita `min_size`**
    - **Property 7: `resize` sempre respeita `max_size`**
    - **Validates: Requirements 1.4**
  - [ ]* 75.3 Property tests para `Workspace`
    - **Property 3: `window_stack` e `windows.keys()` sempre têm os mesmos IDs**
    - **Property 4: após `focus_window`, exatamente uma janela tem `is_focused == true`**
    - **Property 8: `window_stack` nunca tem duplicatas**
    - **Validates: Requirements 1.6, 11.1**
  - [ ]* 75.4 Property tests para `DamageTracker`
    - **Property 5: após `mark_all_dirty`, `intersects_dirty` retorna `true` para qualquer rect**
    - **Property 9: `merge_rectangles(a, b)` contém todos os pontos de `a` e `b`**
    - **Validates: Requirements 2.7**
  - [ ]* 75.5 Property tests para IPC serialization
    - **Property 10: `serialize → deserialize` é identidade para qualquer `Message`**
    - **Validates: Requirements 7.6**

- [ ] 76. Benchmarks com criterion
  - [ ]* 76.1 Benchmark de frame rendering
    - Medir tempo de `render_frame()` para 0, 5, 10, 20 janelas
    - Meta: <16ms para 10 janelas em hardware moderno
    - _Requirements: 14.4_
  - [ ]* 76.2 Benchmark de IPC latency
    - Medir round-trip de `send → receive` para mensagens de diferentes tamanhos
    - Meta: <10ms
    - _Requirements: 7.3_
  - [ ]* 76.3 Benchmark de fuzzy search
    - Medir tempo de busca com 100, 500, 1000 aplicações indexadas
    - Meta: <10ms para 1000 apps
    - _Requirements: 4.3_

- [ ] 77. Otimizações de hot paths
  - Perfilar com `cargo flamegraph` ou `perf`
  - Otimizar `render_frame`: evitar alocações no loop principal
  - Otimizar `DamageTracker::merge_rectangles`: usar array fixo ao invés de Vec quando possível
  - Verificar uso de SIMD pelo tiny-skia (deve ser automático)
  - _Requirements: 14.1, 14.3, 14.6, 14.7_

- [ ] 78. Logging estruturado com tracing
  - Substituir todos os `println!`/`eprintln!` por `tracing::{info, debug, warn, error}`
  - Configurar `tracing-subscriber` com formato JSON para logs em arquivo
  - Rotação de logs com `tracing-appender` (daily rotation)
  - _Requirements: 16.1, 16.2, 16.4, 16.5_

- [ ] 79. Script de instalação
  - Criar `scripts/install.sh`: copia binários para `/usr/local/bin`, configs para `/etc/rust-de`, cria `.desktop` session file
  - Criar `scripts/uninstall.sh`: remove tudo instalado
  - Criar `scripts/build.sh`: `cargo build --release --workspace`
  - _Requirements: 20.3, 20.4, 20.5, 20.6, 20.7_

- [ ] 80. Checkpoint Final — Slice 11
  - `cargo test --workspace` — todos os testes passam
  - `cargo bench --workspace` — benchmarks dentro das metas
  - `cargo clippy --workspace -- -D warnings` — zero warnings
  - `cargo fmt --all --check` — código formatado
  - Sistema completo rodando no Xephyr por 30min sem crashes

---

## Slice 12 — Display Manager (FUTURO)

> **Placeholder apenas — não implementar agora.**
> O Display Manager será um projeto separado após o DE/WM estar estável.
> Referência: `requirements.md` seção futura, integração via PAM e X11 session.

---

## Notas

- Tasks marcadas com `*` são opcionais e podem ser puladas para MVP mais rápido
- Cada task referencia requirements específicos para rastreabilidade
- Checkpoints garantem validação incremental a cada slice
- Property tests validam invariantes universais do domain model
- Testes de integração requerem Xvfb instalado (`apt install xvfb`)
