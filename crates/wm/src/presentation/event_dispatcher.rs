//! Event Dispatcher — Presentation Layer
//!
//! Recebe eventos X11 e despacha para services apropriados

use de_core::{Position, Rectangle, Size, WindowId};
use de_x11::{EventHandler, X11Connection, X11Error};
use tracing::{debug, error, info, warn};
use xcb::Xid;

use super::X11Adapter;
use crate::application::hotkeys::{HotkeyAction, HotkeyManager};
use crate::application::WindowManagerService;

/// Keycode do ESC no X11
const KEYCODE_ESC: u8 = 9;

/// Modifiers do X11 (usando KeyButMask)
/// POR QUE KeyButMask:
/// - event.state() retorna KeyButMask, não u16
/// - Precisamos comparar tipos compatíveis
const MOD_SUPER: xcb::x::KeyButMask = xcb::x::KeyButMask::MOD4; // Super/Windows key
const MOD_SHIFT: xcb::x::KeyButMask = xcb::x::KeyButMask::SHIFT;

/// Botões do mouse
const BUTTON_LEFT: u8 = 1;

/// Estado de grab do mouse (State Machine)
///
/// # POR QUE STATE MACHINE:
/// - Grab tem 3 estados distintos: None, Moving, Resizing
/// - Cada estado tem comportamento diferente no MotionNotify
/// - Evita if/else hell: match no estado é mais limpo
#[derive(Debug, Clone, Copy)]
enum GrabState {
    /// Nenhum grab ativo
    None,
    /// Movendo janela: armazena posição inicial da janela e do mouse
    Moving {
        window_id: WindowId,
        window_start: Position,
        mouse_start: Position,
    },
    /// Redimensionando janela: armazena geometria inicial e posição do mouse
    Resizing {
        window_id: WindowId,
        geometry_start: Rectangle,
        mouse_start: Position,
    },
}

/// Dispatcher de eventos X11 (Presentation Layer)
///
/// # Responsabilidades
/// - Receber eventos X11
/// - Converter tipos X11 → Domain (xcb::x::Window → WindowId)
/// - Despachar para services apropriados
/// - Orquestrar chamadas entre service e adapter
///
/// # NÃO faz
/// - Lógica de negócio (responsabilidade do WindowManagerService)
pub struct EventDispatcher<'a> {
    service: WindowManagerService,
    adapter: X11Adapter<'a>,
    grab_state: GrabState,
    hotkey_manager: HotkeyManager,
}

impl<'a> EventDispatcher<'a> {
    /// Cria um novo EventDispatcher
    pub fn new(x11: &'a X11Connection, workspace_count: u32) -> Self {
        Self {
            service: WindowManagerService::new(workspace_count),
            adapter: X11Adapter::new(x11),
            grab_state: GrabState::None,
            hotkey_manager: HotkeyManager::new(),
        }
    }

    /// Inicia o event loop do Window Manager
    pub fn register_as_wm(&self) -> Result<(), X11Error> {
        self.adapter.register_as_wm()
    }
}

impl<'a> EventHandler for EventDispatcher<'a> {
    fn handle_map_request(&mut self, window: xcb::x::Window) {
        let window_id = WindowId(window.resource_id());
        info!(window_id = ?window_id, "MapRequest received");

        // 1. Service: lógica de negócio (retorna geometria)
        let screen = match self.adapter.screen_geometry() {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "Failed to get screen geometry");
                return;
            }
        };

        let geometry = match self.service.manage_window(window_id, None, screen) {
            Ok(g) => g,
            Err(e) => {
                error!(window_id = ?window_id, error = %e, "Failed to manage window");
                return;
            }
        };

        // 2. Adapter: aplicar no X11
        if let Err(e) = self.adapter.configure_window(window_id, geometry) {
            error!(window_id = ?window_id, error = %e, "Failed to configure window");
            return;
        }

        if let Err(e) = self.adapter.map_window(window_id) {
            error!(window_id = ?window_id, error = %e, "Failed to map window");
            return;
        }

        let all_windows = self.service.all_window_ids();
        if let Err(e) = self.adapter.update_client_list(&all_windows) {
            warn!(error = %e, "Failed to update client list");
        }

        info!(window_id = ?window_id, geometry = ?geometry, "Window managed successfully");
    }

    fn handle_destroy_notify(&mut self, window: xcb::x::Window) {
        let window_id = WindowId(window.resource_id());

        if let Err(e) = self.service.unmanage_window(window_id) {
            warn!(window_id = ?window_id, error = %e, "Failed to unmanage window");
        }

        let all_windows = self.service.all_window_ids();
        let _ = self.adapter.update_client_list(&all_windows);
    }

    fn handle_button_press(&mut self, event: &xcb::x::ButtonPressEvent) {
        let window_id = WindowId(event.child().resource_id());
        let modifiers = event.state();
        let button = event.detail();

        // Detectar Super+Mouse1 (move) ou Super+Shift+Mouse1 (resize)
        if button == BUTTON_LEFT && modifiers.contains(MOD_SUPER) {
            if !self.service.is_managed(window_id) {
                return;
            }

            let geometry = match self.adapter.query_window_geometry(window_id) {
                Ok(g) => g,
                Err(e) => {
                    error!(window_id = ?window_id, error = %e, "Failed to query window geometry");
                    return;
                }
            };

            let mouse_pos = Position {
                x: event.root_x() as i32,
                y: event.root_y() as i32,
            };

            if modifiers.contains(MOD_SHIFT) {
                // Super+Shift+Mouse1 = Resize
                self.grab_state = GrabState::Resizing {
                    window_id,
                    geometry_start: geometry,
                    mouse_start: mouse_pos,
                };
                info!(window_id = ?window_id, "Started resizing window");
            } else {
                // Super+Mouse1 = Move
                self.grab_state = GrabState::Moving {
                    window_id,
                    window_start: geometry.position,
                    mouse_start: mouse_pos,
                };
                info!(window_id = ?window_id, "Started moving window");
            }

            return;
        }

        // Comportamento normal: click-to-focus
        if !self.service.is_managed(window_id) {
            return;
        }

        // Service: atualizar domain
        if let Err(e) = self.service.focus_window(window_id) {
            error!(window_id = ?window_id, error = %e, "Failed to focus window");
            return;
        }

        if let Err(e) = self.adapter.focus_window_x11(window_id) {
            error!(window_id = ?window_id, error = %e, "Failed to focus window in X11");
        }
    }

    fn handle_key_press(&mut self, event: &xcb::x::KeyPressEvent) {
        let keycode = event.detail();
        let modifiers = event.state();

        debug!(keycode = keycode, modifiers = ?modifiers, "KeyPress received");

        // ESC sempre sai (hardcoded)
        if event.detail() == KEYCODE_ESC {
            info!("ESC pressed, exiting...");
            std::process::exit(0);
        }

        // TODO(Slice 10): substituir por xkbcommon::State::key_get_one_sym()
        // para suporte correto a layouts de teclado diferentes de US QWERTY.
        // Referência: task 7.4 no tasks.md
        // Por enquanto, vamos usar keycodes diretos para testar
        // Keycode 24 = 'q', Keycode 41 = 'f' no Layout US
        let keysym = match keycode {
            24 => 0x71,  // 'q' (US layout)
            41 => 0x66,  // 'f' (US layout)
            _ => return, // ignorar outras teclas por enquanto
        };

        // Lookup da ação
        let action = match self.hotkey_manager.lookup(keysym, modifiers) {
            Some(a) => a,
            None => {
                debug!(keysym = ?keysym, modifiers = ?modifiers, "No hotkey binding found");
                return;
            }
        };

        info!(action = ?action, "Hotkey triggered");

        // Executar ação
        self.execute_hotkey_action(action);
    }

    fn handle_unmap_notify(&mut self, window: xcb::x::Window) {
        let window_id = WindowId(window.resource_id());

        if let Err(e) = self.service.unmanage_window(window_id) {
            warn!(window_id = ?window_id, error = %e, "Failed to unmanage window on unmap");
        }

        let all_windows = self.service.all_window_ids();
        let _ = self.adapter.update_client_list(&all_windows);
    }

    // Stubs (implementar futuramente)
    fn handle_configure_request(&mut self, _event: &xcb::x::ConfigureRequestEvent) {
        // TODO: Implementar para ICCCM compliance
    }

    fn handle_button_release(&mut self, event: &xcb::x::ButtonReleaseEvent) {
        let button = event.detail();

        if button != BUTTON_LEFT {
            return;
        }

        match self.grab_state {
            GrabState::Moving { window_id, .. } => {
                info!(window_id = ?window_id, "Finished moving window");
            }
            GrabState::Resizing { window_id, .. } => {
                info!(window_id = ?window_id, "Finished resizing window");
            }
            GrabState::None => {
                return;
            }
        }

        self.grab_state = GrabState::None;
    }

    fn handle_motion_notify(&mut self, event: &xcb::x::MotionNotifyEvent) {
        let mouse_pos = Position {
            x: event.root_x() as i32,
            y: event.root_y() as i32,
        };

        match self.grab_state {
            GrabState::Moving {
                window_id,
                window_start,
                mouse_start,
            } => {
                let dx = mouse_pos.x - mouse_start.x;
                let dy = mouse_pos.y - mouse_start.y;

                let new_position = Position {
                    x: window_start.x + dx,
                    y: window_start.y + dy,
                };

                if let Err(e) = self.service.move_window(window_id, new_position) {
                    error!(window_id = ?window_id, error = %e, "Failed to move window");
                    return;
                }

                if let Err(e) = self.adapter.move_window_x11(window_id, new_position) {
                    error!(window_id = ?window_id, error = %e, "Failed to move window in X11");
                }
            }

            GrabState::Resizing {
                window_id,
                geometry_start,
                mouse_start,
            } => {
                let dx = mouse_pos.x - mouse_start.x;
                let dy = mouse_pos.y - mouse_start.y;

                let new_width = (geometry_start.size.width as i32 + dx).max(100) as u32;
                let new_height = (geometry_start.size.height as i32 + dy).max(50) as u32;

                let new_size = Size {
                    width: new_width,
                    height: new_height,
                };

                if let Err(_e) = self.service.resize_window(window_id, new_size) {
                    // Não logar: MotionNotify é muito frequente
                    return;
                }

                if let Err(e) = self.adapter.resize_window_x11(window_id, new_size) {
                    error!(window_id = ?window_id, error = %e, "Failed to resize window in X11");
                }
            }

            GrabState::None => {
                // Nenhum grab ativo, ignorar
            }
        }
    }

    fn handle_enter_notify(&mut self, _window: xcb::x::Window) {
        // TODO: Implementar para focus-follows-mouse
    }

    fn handle_property_notify(&mut self, _event: &xcb::x::PropertyNotifyEvent) {
        // TODO: Implementar para reagir a mudanças de propriedades (WM_NAME, etc)
    }
}

// 6. Impl EventHandler — TODOS os handlers + métodos privados auxiliares
impl<'a> EventDispatcher<'a> {
    /// Executa uma ação de hotkey
    fn execute_hotkey_action(&mut self, action: HotkeyAction) {
        match action {
            HotkeyAction::CloseWindow => {
                // Pegar janela focada
                let focused = match self.service.focused_window() {
                    Some(id) => id,
                    None => {
                        debug!("No focused window to close");
                        return;
                    }
                };

                // Enviar WM_DELETE_WINDOW (ICCCM)
                if let Err(e) = self.adapter.close_window(focused) {
                    error!(window_id = ?focused, error = %e, "Failed to close window");
                }
            }

            HotkeyAction::ToggleMaximize => {
                let focused = match self.service.focused_window() {
                    Some(id) => id,
                    None => {
                        debug!("No focused window to maximize");
                        return;
                    }
                };

                // Service: toggle maximize (domain logic)
                if let Err(e) = self.service.toggle_maximize(focused) {
                    error!(window_id = ?focused, error = %e, "Failed to toggle maximize");
                    return;
                }

                // Adapter: aplicar geometria no X11
                let geometry = match self.service.window_geometry(focused) {
                    Some(g) => g,
                    None => {
                        error!(window_id = ?focused, "Window geometry not found after maximize");
                        return;
                    }
                };

                if let Err(e) = self.adapter.configure_window(focused, geometry) {
                    error!(window_id = ?focused, error = %e, "Failed to apply maximized geometry");
                }
            }

            HotkeyAction::Quit => {
                info!("Quit hotkey triggered, exiting...");
                std::process::exit(0);
            }
        }
    }
}
