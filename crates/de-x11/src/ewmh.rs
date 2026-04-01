use crate::{AtomCache, X11Error};
use xcb::{x, Connection, Xid};

pub struct Ewmh<'a> {
    conn: &'a Connection,
    atoms: &'a AtomCache,
    root: x::Window,
}

impl<'a> Ewmh<'a> {
    pub fn new(conn: &'a Connection, atoms: &'a AtomCache, root: x::Window) -> Self {
        Self { conn, atoms, root }
    }

    // Anunciar suporte EWMH
    pub fn set_supported(&self, atoms: &[x::Atom]) -> Result<(), X11Error> {
        self.conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_supported,
            r#type: x::ATOM_ATOM,
            data: atoms,
        });

        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    // Lista de janelas gerenciadas
    pub fn set_client_list(&self, windows: &[x::Window]) -> Result<(), X11Error> {
        self.conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_client_list,
            r#type: x::ATOM_WINDOW,
            data: windows,
        });

        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    // Janelas com foco
    pub fn set_active_window(&self, window: Option<x::Window>) -> Result<(), X11Error> {
        let data = match window {
            Some(w) => vec![w],
            None => vec![x::Window::none()],
        };

        self.conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_active_window,
            r#type: x::ATOM_WINDOW,
            data: &data,
        });

        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    // Workspaces
    pub fn set_number_of_desktops(&self, count: u32) -> Result<(), X11Error> {
        self.conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_number_of_desktops,
            r#type: x::ATOM_CARDINAL,
            data: &[count],
        });

        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }
    pub fn set_current_desktop(&self, index: u32) -> Result<(), X11Error> {
        self.conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.root,
            property: self.atoms.net_current_desktop,
            r#type: x::ATOM_CARDINAL,
            data: &[index],
        });

        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    // Ler nome da janela (UTF-8)
    pub fn get_window_name(&self, window: x::Window) -> Result<String, X11Error> {
        // Tentar _NET_WM_NAME primeiro (UTF-8)
        let cookie = self.conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property: self.atoms.net_wm_name,
            r#type: x::ATOM_ANY,
            long_offset: 0,
            long_length: 1024,
        });

        if let Ok(reply) = self.conn.wait_for_reply(cookie) {
            if reply.r#type() != x::ATOM_NONE {
                let bytes = reply.value::<u8>();
                if let Ok(name) = String::from_utf8(bytes.to_vec()) {
                    return Ok(name);
                }
            }
        }

        // Failback WM_NAME (Latin-1)
        Err(X11Error::ProtocolError(
            "Failed to get window name".to_string(),
        ))
    }
}
