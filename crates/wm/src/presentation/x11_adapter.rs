//! X11 Adapter — Presentation Layer
//!
//! Traduz operações de domain para chamadas X11

use de_core::{Position, Rectangle, Size, WindowId};
use de_x11::{Ewmh, X11Connection, X11Error};
use tracing::info;
use xcb::{Xid, XidNew};

use crate::WmError;

/// Adapter que traduz operações de domain para X11 (Presentation Layer)
///
/// # Responsabilidades
/// - Executar operações X11 (ConfigureWindow, MapWindow, SetInputFocus)
/// - Atualizar EWMH/ICCCM
/// - Converter tipos domain → X11 (WindowId → xcb::x::Window)
///
/// # NÃO faz
/// - Lógica de negócio (responsabilidade do WindowManagerService)
///
/// # Design
/// - NÃO guarda referência a X11Connection (evita lifetime complexo)
/// - Recebe &mut X11Connection em cada método (mais flexível)
pub struct X11Adapter;

impl X11Adapter {
    /// Cria um novo X11Adapter
    pub fn new() -> Self {
        Self
    }

    pub fn honor_configure_request(
        &self,
        x11: &mut X11Connection,
        event: &xcb::x::ConfigureRequestEvent,
    ) -> Result<(), WmError> {
        let mut value_list = Vec::new();

        if event.value_mask().contains(xcb::x::ConfigWindowMask::X) {
            value_list.push(xcb::x::ConfigWindow::X(event.x() as i32));
        }
        if event.value_mask().contains(xcb::x::ConfigWindowMask::Y) {
            value_list.push(xcb::x::ConfigWindow::Y(event.y() as i32));
        }
        if event.value_mask().contains(xcb::x::ConfigWindowMask::WIDTH) {
            value_list.push(xcb::x::ConfigWindow::Width(event.width() as u32));
        }
        if event
            .value_mask()
            .contains(xcb::x::ConfigWindowMask::HEIGHT)
        {
            value_list.push(xcb::x::ConfigWindow::Height(event.height() as u32));
        }
        if event
            .value_mask()
            .contains(xcb::x::ConfigWindowMask::BORDER_WIDTH)
        {
            value_list.push(xcb::x::ConfigWindow::BorderWidth(
                event.border_width() as u32
            ));
        }
        if event
            .value_mask()
            .contains(xcb::x::ConfigWindowMask::SIBLING)
        {
            value_list.push(xcb::x::ConfigWindow::Sibling(event.sibling()));
        }
        if event
            .value_mask()
            .contains(xcb::x::ConfigWindowMask::STACK_MODE)
        {
            value_list.push(xcb::x::ConfigWindow::StackMode(event.stack_mode()));
        }

        // Aplicar configuração
        x11.connection().send_request(&xcb::x::ConfigureWindow {
            window: event.window(),
            value_list: &value_list,
        });

        x11.connection()
            .flush()
            .map_err(|e| WmError::X11(X11Error::ProtocolError(e.to_string())))?;

        Ok(())
    }

    /// Registra como Window Manager (SubstructureRedirect)
    pub fn register_as_wm(&self, x11: &mut X11Connection) -> Result<(), X11Error> {
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

        // Registrar hotkeys globais
        self.grab_default_hotkeys(x11)?;

        Ok(())
    }

    /// Query geometria atual de uma janela
    pub fn query_window_geometry(
        &self,
        x11: &X11Connection,
        window_id: WindowId,
    ) -> Result<Rectangle, X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);

        let cookie = x11.connection().send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(x11_window),
        });

        let reply = x11
            .connection()
            .wait_for_reply(cookie)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(Rectangle::new_unchecked(
            reply.x() as i32,
            reply.y() as i32,
            reply.width() as u32,
            reply.height() as u32,
        ))
    }

    /// Retorna geometria da tela
    pub fn screen_geometry(&self, x11: &X11Connection) -> Result<Rectangle, X11Error> {
        x11.screen_geometry()
    }

    /// Configura geometria de uma janela no X11
    pub fn configure_window(
        &self,
        x11: &mut X11Connection,
        window_id: WindowId,
        geometry: Rectangle,
    ) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);
        x11.connection().send_request(&xcb::x::ConfigureWindow {
            window: x11_window,
            value_list: &[
                xcb::x::ConfigWindow::X(geometry.position.x),
                xcb::x::ConfigWindow::Y(geometry.position.y),
                xcb::x::ConfigWindow::Width(geometry.size.width()),
                xcb::x::ConfigWindow::Height(geometry.size.height()),
            ],
        });

        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    /// Mapeia janela (torna visível)
    pub fn map_window(&self, x11: &mut X11Connection, window_id: WindowId) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);
        x11.map_window(x11_window)
    }

    /// Move janela no X11
    pub fn move_window_x11(
        &self,
        x11: &mut X11Connection,
        window_id: WindowId,
        position: Position,
    ) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);
        x11.connection().send_request(&xcb::x::ConfigureWindow {
            window: x11_window,
            value_list: &[
                xcb::x::ConfigWindow::X(position.x),
                xcb::x::ConfigWindow::Y(position.y),
            ],
        });
        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, position = ?position, "Window moved");
        Ok(())
    }

    /// Redimensiona janela no X11
    pub fn resize_window_x11(
        &self,
        x11: &mut X11Connection,
        window_id: WindowId,
        size: Size,
    ) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);
        x11.connection().send_request(&xcb::x::ConfigureWindow {
            window: x11_window,
            value_list: &[
                xcb::x::ConfigWindow::Width(size.width()),
                xcb::x::ConfigWindow::Height(size.height()),
            ],
        });
        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, size = ?size, "Window resized");
        Ok(())
    }

    /// Foca janela no X11
    pub fn focus_window_x11(
        &self,
        x11: &mut X11Connection,
        window_id: WindowId,
    ) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);

        // SetInputFocus
        x11.connection().send_request(&xcb::x::SetInputFocus {
            revert_to: xcb::x::InputFocus::PointerRoot,
            focus: x11_window,
            time: xcb::x::CURRENT_TIME,
        });

        // Atualizar EWMH _NET_ACTIVE_WINDOW
        let root = x11.root_window()?;
        let ewmh = Ewmh::new(x11.connection(), &x11.atoms, root);
        ewmh.set_active_window(Some(x11_window))?;

        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, "Window focused");
        Ok(())
    }

    /// Atualiza EWMH client list
    pub fn update_client_list(
        &self,
        x11: &mut X11Connection,
        window_ids: &[WindowId],
    ) -> Result<(), X11Error> {
        let x11_windows: Vec<xcb::x::Window> = window_ids
            .iter()
            .map(|id| xcb::x::Window::new(id.0))
            .collect();

        let root = x11.root_window()?;
        let ewmh = Ewmh::new(x11.connection(), &x11.atoms, root);
        ewmh.set_client_list(&x11_windows)
    }

    /// Fecha janela enviando WM_DELETE_WINDOW (ICCCM)
    pub fn close_window(
        &self,
        x11: &mut X11Connection,
        window_id: WindowId,
    ) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);

        // Enviar ClientMessage com WM_DELETE_WINDOW
        let wm_delete = x11.atoms.wm_delete_window;
        let wm_protocols = x11.atoms.wm_protocols;

        x11.connection().send_request(&xcb::x::SendEvent {
            propagate: false,
            destination: xcb::x::SendEventDest::Window(x11_window),
            event_mask: xcb::x::EventMask::NO_EVENT,
            event: &xcb::x::ClientMessageEvent::new(
                x11_window,
                wm_protocols,
                xcb::x::ClientMessageData::Data32([
                    wm_delete.resource_id(),
                    xcb::x::CURRENT_TIME,
                    0,
                    0,
                    0,
                ]),
            ),
        });

        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, "Sent WM_DELETE_WINDOW");
        Ok(())
    }

    /// Registra hotkey global via GrabKey
    pub fn grab_key(
        &self,
        x11: &mut X11Connection,
        keycode: xcb::x::Keycode,
        modifiers: xcb::x::ModMask,
    ) -> Result<(), X11Error> {
        let root = x11.root_window()?;

        x11.connection().send_request(&xcb::x::GrabKey {
            owner_events: false, // só o WM recebe, não o app focado
            grab_window: root,
            modifiers,
            key: keycode,
            pointer_mode: xcb::x::GrabMode::Async,
            keyboard_mode: xcb::x::GrabMode::Async,
        });

        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    /// Registra hotkeys padrão no X11
    pub fn grab_default_hotkeys(&self, x11: &mut X11Connection) -> Result<(), X11Error> {
        // Super+Q (keycode 24 no US layout)
        self.grab_key(x11, 24, xcb::x::ModMask::N4)?;

        // Super+F (keycode 41 no US layout)
        self.grab_key(x11, 41, xcb::x::ModMask::N4)?;

        // ESC (keycode 9, sem modifier) — para sair
        self.grab_key(x11, 9, xcb::x::ModMask::empty())?;

        info!("Grabbed default hotkeys");
        Ok(())
    }

    // Faz grab de ButtonPress em uma janela para receber eventos de clique
    ///
    /// POR QUE isso é necessário?
    /// - Por padrão, ButtonPress vai para a janela, não para o WM
    /// - Precisamos fazer grab para interceptar cliques e implementar click-to-focus
    /// - owner_events=true: a janela também recebe o evento (não bloqueamos)
    pub fn grab_button_on_window(
        &self,
        x11: &mut X11Connection,
        window_id: WindowId,
    ) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);

        // Grab de todos os botões do mouse (ANY)
        // com qualquer modifier (ANY) para click-to-focus
        x11.connection().send_request(&xcb::x::GrabButton {
            owner_events: true, // ← IMPORTANTE: janela também recebe o evento
            grab_window: x11_window,
            event_mask: xcb::x::EventMask::BUTTON_PRESS,
            pointer_mode: xcb::x::GrabMode::Async, // ← MUDANÇA: Async para não congelar
            keyboard_mode: xcb::x::GrabMode::Async,
            confine_to: xcb::x::Window::none(),
            cursor: xcb::x::Cursor::none(),
            button: xcb::x::ButtonIndex::Any, // ← Qualquer botão
            modifiers: xcb::x::ModMask::ANY,  // ← Qualquer modifier
        });

        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    /// Permite que o evento continue para a janela (replay)
    ///
    /// POR QUE isso é necessário?
    /// - GrabMode::Sync congela o pointer até chamarmos AllowEvents
    /// - Replay: o evento é reenviado para a janela como se não tivesse sido grabbed
    pub fn allow_events_replay(&self, x11: &mut X11Connection) -> Result<(), X11Error> {
        x11.connection().send_request(&xcb::x::AllowEvents {
            mode: xcb::x::Allow::ReplayPointer,
            time: xcb::x::CURRENT_TIME,
        });

        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    /// Descongela o pointer para permitir novos eventos (MotionNotify)
    ///
    /// POR QUE isso é necessário?
    /// - GrabMode::Sync congela o pointer até chamarmos AllowEvents
    /// - AsyncPointer: descongela e permite que novos eventos sejam gerados
    /// - Usado quando iniciamos move/resize com Super+Mouse
    pub fn allow_events_async(&self, x11: &mut X11Connection) -> Result<(), X11Error> {
        x11.connection().send_request(&xcb::x::AllowEvents {
            mode: xcb::x::Allow::AsyncPointer,
            time: xcb::x::CURRENT_TIME,
        });

        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    /// Eleva janela para o topo (raise) no X11
    pub fn raise_window_x11(
        &self,
        x11: &mut X11Connection,
        window_id: WindowId,
    ) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);

        x11.connection().send_request(&xcb::x::ConfigureWindow {
            window: x11_window,
            value_list: &[xcb::x::ConfigWindow::StackMode(xcb::x::StackMode::Above)],
        });

        x11.connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, "Window raised to top");
        Ok(())
    }
}
