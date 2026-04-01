//! Event Dispatcher — Presentation Layer
//!
//! Recebe eventos X11 e despacha para services apropriados

use de_core::WindowId;
use de_x11::{EventHandler, X11Connection, X11Error};
use tracing::{error, info, warn};
use xcb::Xid;

use super::X11Adapter;
use crate::application::WindowManagerService;

/// Keycode do ESC no X11
const KEYCODE_ESC: u8 = 9;

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
}

impl<'a> EventDispatcher<'a> {
    /// Cria um novo EventDispatcher
    pub fn new(x11: &'a X11Connection, workspace_count: u32) -> Self {
        Self {
            service: WindowManagerService::new(workspace_count),
            adapter: X11Adapter::new(x11),
        }
    }

    /// Inicia o event loop do Window Manager
    pub fn register_as_wm(&self, x11: &X11Connection) -> Result<(), X11Error> {
        let root = x11.root_window()?;
        let conn = x11.connection();

        let cookie = conn.send_request_checked(&xcb::x::ChangeWindowAttributes {
            window: root,
            value_list: &[xcb::x::Cw::EventMask(
                xcb::x::EventMask::SUBSTRUCTURE_REDIRECT
                    | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY
                    | xcb::x::EventMask::BUTTON_PRESS
                    | xcb::x::EventMask::KEY_PRESS
                    | xcb::x::EventMask::STRUCTURE_NOTIFY
                    | xcb::x::EventMask::PROPERTY_CHANGE,
            )],
        });

        conn.check_request(cookie).map_err(|_| {
            X11Error::ProtocolError("Another window manager is already running".to_string())
        })?;

        info!("Registered as window manager");
        // ↑ MUDANÇA: tracing::info em vez de println!
        // POR QUE: logging estruturado, pode filtrar por nível

        Ok(())
    }

    pub fn run(&mut self, x11: &X11Connection) -> Result<(), X11Error> {
        info!("Window Manager running. Press ESC to exit.");
        // ↑ MUDANÇA: tracing::info

        loop {
            let event = x11.wait_for_event()?;

            match event {
                xcb::Event::X(xcb::x::Event::MapRequest(e)) => {
                    self.handle_map_request(e.window());
                }
                xcb::Event::X(xcb::x::Event::DestroyNotify(e)) => {
                    self.handle_destroy_notify(e.window());
                }
                xcb::Event::X(xcb::x::Event::ButtonPress(e)) => {
                    self.handle_button_press(&e);
                }
                xcb::Event::X(xcb::x::Event::KeyPress(e)) => {
                    self.handle_key_press(&e);
                }
                xcb::Event::X(xcb::x::Event::UnmapNotify(e)) => {
                    self.handle_unmap_notify(e.window());
                }
                xcb::Event::X(xcb::x::Event::ConfigureRequest(e)) => {
                    self.handle_configure_request(&e);
                }
                xcb::Event::X(xcb::x::Event::ButtonRelease(e)) => {
                    self.handle_button_release(&e);
                }
                xcb::Event::X(xcb::x::Event::MotionNotify(e)) => {
                    self.handle_motion_notify(&e);
                }
                xcb::Event::X(xcb::x::Event::EnterNotify(e)) => {
                    self.handle_enter_notify(e.event());
                }
                xcb::Event::X(xcb::x::Event::PropertyNotify(e)) => {
                    self.handle_property_notify(&e);
                }
                _ => {}
            }
        }
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

        // CORREÇÃO: passar None como requested_size (3 argumentos)
        // POR QUE: manage_window agora recebe requested_size
        // TODO: implementar query do tamanho real da janela
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

        // 3. Adapter: atualizar EWMH
        let all_windows = self.service.all_window_ids();
        if let Err(e) = self.adapter.update_client_list(&all_windows) {
            warn!(error = %e, "Failed to update client list");
        }

        info!(window_id = ?window_id, geometry = ?geometry, "Window managed successfully");
    }

    fn handle_destroy_notify(&mut self, window: xcb::x::Window) {
        let window_id = WindowId(window.resource_id());

        if let Err(e) = self.service.unmanage_window(window_id) {
            // Janela pode já ter sido removida, não é erro crítico
            warn!(window_id = ?window_id, error = %e, "Failed to unmanage window");
        }

        // Atualizar EWMH
        let all_windows = self.service.all_window_ids();
        let _ = self.adapter.update_client_list(&all_windows);
    }

    fn handle_button_press(&mut self, event: &xcb::x::ButtonPressEvent) {
        let window_id = WindowId(event.event().resource_id());

        // Guard: só focar se a janela está gerenciada
        if !self.service.is_managed(window_id) {
            return;
        }

        // Service: atualizar domain
        if let Err(e) = self.service.focus_window(window_id) {
            error!(window_id = ?window_id, error = %e, "Failed to focus window");
            // ↑ MUDANÇA: tracing::error
            return;
        }

        // Adapter: aplicar no X11
        if let Err(e) = self.adapter.focus_window_x11(window_id) {
            error!(window_id = ?window_id, error = %e, "Failed to focus window in X11");
        }
    }

    fn handle_key_press(&mut self, event: &xcb::x::KeyPressEvent) {
        info!(keycode = event.detail(), "KeyPress received");

        if event.detail() == KEYCODE_ESC {
            info!("ESC pressed, exiting...");
            std::process::exit(0);
        }
    }

    /// IMPLEMENTAÇÃO CRÍTICA: Remove janela quando usuário fecha
    ///
    /// # POR QUE IMPLEMENTAR:
    /// - UnmapNotify é enviado quando janela é fechada
    /// - Se não remover, janela fica "fantasma" no workspace
    /// - Causa memory leak e bugs
    fn handle_unmap_notify(&mut self, window: xcb::x::Window) {
        let window_id = WindowId(window.resource_id());

        // Remover do gerenciamento
        if let Err(e) = self.service.unmanage_window(window_id) {
            warn!(window_id = ?window_id, error = %e, "Failed to unmanage window on unmap");
        }

        // Atualizar EWMH
        let all_windows = self.service.all_window_ids();
        let _ = self.adapter.update_client_list(&all_windows);
    }

    // Stubs (implementar futuramente)
    fn handle_configure_request(&mut self, _event: &xcb::x::ConfigureRequestEvent) {
        // TODO: Implementar para ICCCM compliance
        // Aplicações pedem resize via ConfigureRequest
    }

    fn handle_button_release(&mut self, _event: &xcb::x::ButtonReleaseEvent) {
        // TODO: Implementar para drag-and-drop de janelas
    }

    fn handle_motion_notify(&mut self, _event: &xcb::x::MotionNotifyEvent) {
        // TODO: Implementar para move/resize interativo
    }

    fn handle_enter_notify(&mut self, _window: xcb::x::Window) {
        // TODO: Implementar para focus-follows-mouse
    }

    fn handle_property_notify(&mut self, _event: &xcb::x::PropertyNotifyEvent) {
        // TODO: Implementar para reagir a mudanças de propriedades (WM_NAME, etc)
    }
}
