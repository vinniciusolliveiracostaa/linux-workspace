//! X11 Adapter — Presentation Layer
//!
//! Traduz operações de domain para chamadas X11

use de_core::{Position, Rectangle, Size, WindowId};
use de_x11::{Ewmh, X11Connection, X11Error};
use tracing::info;
use xcb::{Xid, XidNew};

/// Adapter que traduz operações de domain para X11 (Presentation Layer)
///
/// # Responsabilidades
/// - Executar operações X11 (ConfigureWindow, MapWindow, SetInputFocus)
/// - Atualizar EWMH/ICCCM
/// - Converter tipos domain → X11 (WindowId → xcb::x::Window)
///
/// # NÃO faz
/// - Lógica de negócio (responsabilidade do WindowManagerService)
pub struct X11Adapter<'a> {
    x11: &'a X11Connection,
}

impl<'a> X11Adapter<'a> {
    /// Cria um novo X11Adapter
    pub fn new(x11: &'a X11Connection) -> Self {
        Self { x11 }
    }

    /// Query geometria atual de uma janela
    pub fn query_window_geometry(&self, window_id: WindowId) -> Result<Rectangle, X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);

        let cookie = self.x11.connection().send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(x11_window),
        });

        let reply = self
            .x11
            .connection()
            .wait_for_reply(cookie)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(Rectangle::new(
            reply.x() as i32,
            reply.y() as i32,
            reply.width() as u32,
            reply.height() as u32,
        ))
    }

    /// Retorna geometria da tela
    pub fn screen_geometry(&self) -> Result<Rectangle, X11Error> {
        self.x11.screen_geometry()
    }

    /// Configura geometria de uma janela no X11
    pub fn configure_window(
        &self,
        window_id: WindowId,
        geometry: Rectangle,
    ) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);
        self.x11
            .connection()
            .send_request(&xcb::x::ConfigureWindow {
                window: x11_window,
                value_list: &[
                    xcb::x::ConfigWindow::X(geometry.position.x),
                    xcb::x::ConfigWindow::Y(geometry.position.y),
                    xcb::x::ConfigWindow::Width(geometry.size.width),
                    xcb::x::ConfigWindow::Height(geometry.size.height),
                ],
            });
        Ok(())
    }

    /// Mapeia janela (torna visível)
    pub fn map_window(&self, window_id: WindowId) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);
        self.x11.map_window(x11_window)
    }

    /// Move janela no X11
    pub fn move_window_x11(&self, window_id: WindowId, position: Position) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);
        self.x11
            .connection()
            .send_request(&xcb::x::ConfigureWindow {
                window: x11_window,
                value_list: &[
                    xcb::x::ConfigWindow::X(position.x),
                    xcb::x::ConfigWindow::Y(position.y),
                ],
            });
        self.x11
            .connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, position = ?position, "Window moved");
        Ok(())
    }

    /// Redimensiona janela no X11
    pub fn resize_window_x11(&self, window_id: WindowId, size: Size) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);
        self.x11
            .connection()
            .send_request(&xcb::x::ConfigureWindow {
                window: x11_window,
                value_list: &[
                    xcb::x::ConfigWindow::Width(size.width),
                    xcb::x::ConfigWindow::Height(size.height),
                ],
            });
        self.x11
            .connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, size = ?size, "Window resized");
        Ok(())
    }

    /// Foca janela no X11
    pub fn focus_window_x11(&self, window_id: WindowId) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);

        // SetInputFocus
        self.x11.connection().send_request(&xcb::x::SetInputFocus {
            revert_to: xcb::x::InputFocus::PointerRoot,
            focus: x11_window,
            time: xcb::x::CURRENT_TIME,
        });

        // Atualizar EWMH _NET_ACTIVE_WINDOW
        let root = self.x11.root_window()?;
        let ewmh = Ewmh::new(self.x11.connection(), &self.x11.atoms, root);
        ewmh.set_active_window(Some(x11_window))?;

        self.x11
            .connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, "Window focused");
        Ok(())
    }

    /// Atualiza EWMH client list
    pub fn update_client_list(&self, window_ids: &[WindowId]) -> Result<(), X11Error> {
        let x11_windows: Vec<xcb::x::Window> = window_ids
            .iter()
            .map(|id| xcb::x::Window::new(id.0))
            .collect();

        let root = self.x11.root_window()?;
        let ewmh = Ewmh::new(self.x11.connection(), &self.x11.atoms, root);
        ewmh.set_client_list(&x11_windows)
    }

    /// Fecha janela enviando WM_DELETE_WINDOW (ICCCM)
    pub fn close_window(&self, window_id: WindowId) -> Result<(), X11Error> {
        let x11_window = xcb::x::Window::new(window_id.0);

        // Enviar ClientMessage com WM_DELETE_WINDOW
        let wm_delete = self.x11.atoms.wm_delete_window;
        let wm_protocols = self.x11.atoms.wm_protocols;

        self.x11.connection().send_request(&xcb::x::SendEvent {
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

        self.x11
            .connection()
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        info!(window_id = ?window_id, "Sent WM_DELETE_WINDOW");
        Ok(())
    }
}
