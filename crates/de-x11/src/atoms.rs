use crate::X11Error;
use xcb::{x, Connection};

pub struct AtomCache {
    // EWMH atoms
    pub net_supported: x::Atom,
    pub net_client_list: x::Atom,
    pub net_active_window: x::Atom,
    pub net_wm_name: x::Atom,
    pub net_wm_state: x::Atom,
    pub net_wm_state_fullscreen: x::Atom,
    pub net_number_of_desktops: x::Atom,
    pub net_current_desktop: x::Atom,

    // ICCCM atoms
    pub wm_protocols: x::Atom,
    pub wm_delete_window: x::Atom,
    pub wm_state: x::Atom,
    pub wm_change_state: x::Atom,
}

impl AtomCache {
    pub fn new(conn: &Connection) -> Result<Self, X11Error> {
        Ok(Self {
            // EWMH
            net_supported: intern_atom(conn, "_NET_SUPPORTED")?,
            net_client_list: intern_atom(conn, "_NET_CLIENT_LIST")?,
            net_active_window: intern_atom(conn, "_NET_ACTIVE_WINDOW")?,
            net_wm_name: intern_atom(conn, "_NET_WM_NAME")?,
            net_wm_state: intern_atom(conn, "_NET_WM_STATE")?,
            net_wm_state_fullscreen: intern_atom(conn, "_NET_WM_STATE_FULLSCREEN")?,
            net_number_of_desktops: intern_atom(conn, "_NET_NUMBER_OF_DESKTOPS")?,
            net_current_desktop: intern_atom(conn, "_NET_CURRENT_DESKTOP")?,

            // ICCCM
            wm_protocols: intern_atom(conn, "WM_PROTOCOLS")?,
            wm_delete_window: intern_atom(conn, "WM_DELETE_WINDOW")?,
            wm_state: intern_atom(conn, "WM_STATE")?,
            wm_change_state: intern_atom(conn, "WM_CHANGE_STATE")?,
        })
    }
}

fn intern_atom(conn: &Connection, name: &str) -> Result<x::Atom, X11Error> {
    let cookie = conn.send_request(&x::InternAtom {
        only_if_exists: false,
        name: name.as_bytes(),
    });

    let reply = conn
        .wait_for_reply(cookie)
        .map_err(|e| X11Error::ProtocolError(format!("Failed to intern atom {}: {}", name, e)))?;

    Ok(reply.atom())
}
