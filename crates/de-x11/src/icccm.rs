use crate::{AtomCache, X11Error};
use xcb::{x, Connection, Xid, XidNew};

pub struct Icccm<'a> {
    conn: &'a Connection,
    atoms: &'a AtomCache,
}

// Constantes para WM_SIZE_HINTS
const WM_SIZE_HINTS_LENGTH: u32 = 18;

// Bit flags
const P_MIN_SIZE: u32 = 1 << 4;
const P_MAX_SIZE: u32 = 1 << 5;
const P_BASE_SIZE: u32 = 1 << 8;

// Índices da struct WM_SIZE_HINTS
const MIN_WIDTH_INDEX: usize = 5;
const MIN_HEIGHT_INDEX: usize = 6;
const MAX_WIDTH_INDEX: usize = 7;
const MAX_HEIGHT_INDEX: usize = 8;
const BASE_WIDTH_INDEX: usize = 15;
const BASE_HEIGHT_INDEX: usize = 16;

impl<'a> Icccm<'a> {
    pub fn new(conn: &'a Connection, atoms: &'a AtomCache) -> Self {
        Self { conn, atoms }
    }

    fn get_property_bytes(
        &self,
        window: x::Window,
        property: x::Atom,
        prop_type: x::Atom,
        length: u32,
    ) -> Result<Vec<u8>, X11Error> {
        let cookie = self.conn.send_request(&x::GetProperty {
            delete: false,
            window,
            property,
            r#type: prop_type,
            long_offset: 0,
            long_length: length,
        });

        let reply = self
            .conn
            .wait_for_reply(cookie)
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        if reply.r#type() == x::ATOM_NONE {
            return Err(X11Error::ProtocolError(format!(
                "Property {:?} not found",
                property
            )));
        }

        Ok(reply.value::<u8>().to_vec())
    }

    pub fn get_protocols(&self, window: x::Window) -> Result<Vec<x::Atom>, X11Error> {
        match self.get_property_bytes(window, self.atoms.wm_protocols, x::ATOM_ATOM, 1024) {
            Ok(bytes) => {
                let atoms: Vec<x::Atom> = bytes
                    .chunks_exact(4)
                    .map(|chunk| {
                        let id = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        x::Atom::new(id)
                    })
                    .collect();
                Ok(atoms)
            }
            Err(_) => Ok(vec![]), // Janelas antigas podem não ter WM_PROTOCOLS
        }
    }

    pub fn get_wm_class(&self, window: x::Window) -> Result<(String, String), X11Error> {
        let bytes = self.get_property_bytes(window, x::ATOM_WM_CLASS, x::ATOM_STRING, 1024)?;

        let parts: Vec<&[u8]> = bytes.split(|&b| b == 0).filter(|s| !s.is_empty()).collect();

        match parts.as_slice() {
            [instance, class, ..] => Ok((
                String::from_utf8_lossy(instance).to_string(),
                String::from_utf8_lossy(class).to_string(),
            )),
            _ => Err(X11Error::ProtocolError(
                "Invalid WM_CLASS format: expected at least 2 null-terminated strings".to_string(),
            )),
        }
    }

    pub fn supports_delete_window(&self, window: x::Window) -> bool {
        if let Ok(protocols) = self.get_protocols(window) {
            protocols.contains(&self.atoms.wm_delete_window)
        } else {
            false
        }
    }

    pub fn send_delete_window(&self, window: x::Window) -> Result<(), X11Error> {
        let event = x::ClientMessageEvent::new(
            window,
            self.atoms.wm_protocols,
            x::ClientMessageData::Data32([self.atoms.wm_delete_window.resource_id(), 0, 0, 0, 0]),
        );

        self.conn.send_request(&x::SendEvent {
            propagate: false,
            destination: x::SendEventDest::Window(window),
            event_mask: x::EventMask::empty(),
            event: &event,
        });

        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(())
    }

    pub fn get_normal_hints(&self, window: x::Window) -> Result<NormalHints, X11Error> {
        match self.get_property_bytes(
            window,
            x::ATOM_WM_NORMAL_HINTS,
            x::ATOM_WM_SIZE_HINTS,
            WM_SIZE_HINTS_LENGTH,
        ) {
            Ok(bytes) => {
                let data: Vec<u32> = bytes
                    .chunks_exact(4)
                    .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                NormalHints::from_raw_data(&data)
            }
            Err(_) => Ok(NormalHints {
                min_size: None,
                max_size: None,
                base_size: None,
            }),
        }
    }
}

pub struct NormalHints {
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub base_size: Option<(u32, u32)>,
}

impl NormalHints {
    fn from_raw_data(data: &[u32]) -> Result<Self, X11Error> {
        if data.len() < WM_SIZE_HINTS_LENGTH as usize {
            return Err(X11Error::ProtocolError(format!(
                "Invalid WM_NORMAL_HINTS size: expected {}, got {}",
                WM_SIZE_HINTS_LENGTH,
                data.len()
            )));
        }

        let flags = data[0];

        Ok(Self {
            min_size: Self::extract_size_explicit(
                flags,
                P_MIN_SIZE,
                data,
                MIN_WIDTH_INDEX,
                MIN_HEIGHT_INDEX,
            ),
            max_size: Self::extract_size_explicit(
                flags,
                P_MAX_SIZE,
                data,
                MAX_WIDTH_INDEX,
                MAX_HEIGHT_INDEX,
            ),
            base_size: Self::extract_size_explicit(
                flags,
                P_BASE_SIZE,
                data,
                BASE_WIDTH_INDEX,
                BASE_HEIGHT_INDEX,
            ),
        })
    }

    fn extract_size_explicit(
        flags: u32,
        flag_bit: u32,
        data: &[u32],
        width_index: usize,
        height_index: usize,
    ) -> Option<(u32, u32)> {
        if flags & flag_bit != 0 {
            Some((data[width_index], data[height_index]))
        } else {
            None
        }
    }
}
