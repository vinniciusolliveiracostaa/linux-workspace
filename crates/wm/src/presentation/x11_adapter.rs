//! X11 Adapter — Presentation Layer
//!
//! Traduz operações de domain para chamadas X11

use de_core::{Position, Rectangle, Size, WindowId};
use de_x11::{Ewmh, X11Connection, X11Error};
use tracing::info;
use xcb::XidNew;

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

        println!("Window {:?} resized to {:?}", window_id, size);
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

        println!("Window {:?} focused", window_id);
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
}
