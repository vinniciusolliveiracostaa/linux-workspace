use crate::{AtomCache, X11Error};
use de_core::Rectangle;
use std::cell::RefCell;
use std::collections::HashMap;
use xcb::{x, Connection, Xid};

pub struct X11Connection {
    conn: Connection,
    screen_num: i32,
    gc_cache: RefCell<HashMap<u32, x::Gcontext>>,
    pub atoms: AtomCache,
}

impl X11Connection {
    pub fn connect() -> Result<Self, X11Error> {
        let (conn, screen_num) =
            Connection::connect(None).map_err(|e| X11Error::ConnectionFailed(e.to_string()))?;

        let atoms = AtomCache::new(&conn)?;

        Ok(Self {
            conn,
            screen_num,
            gc_cache: RefCell::new(HashMap::new()),
            atoms,
        })
    }

    pub fn root_window(&self) -> Result<x::Window, X11Error> {
        let setup = self.conn.get_setup();
        let screen = setup
            .roots()
            .nth(self.screen_num as usize)
            .ok_or(X11Error::InvalidScreen(self.screen_num))?;
        Ok(screen.root())
    }

    pub fn screen_geometry(&self) -> Result<Rectangle, X11Error> {
        let setup = self.conn.get_setup();
        let screen = setup
            .roots()
            .nth(self.screen_num as usize)
            .ok_or(X11Error::InvalidScreen(self.screen_num))?;

        Ok(Rectangle::new_unchecked(
            0,
            0,
            screen.width_in_pixels() as u32,
            screen.height_in_pixels() as u32,
        ))
    }

    pub fn create_window(&self, geometry: Rectangle) -> Result<x::Window, X11Error> {
        let window_id = self.conn.generate_id();
        let root = self.root_window()?;

        let values = [
            x::Cw::BackPixel(0xECECEC),
            x::Cw::EventMask(
                x::EventMask::EXPOSURE
                    | x::EventMask::KEY_PRESS
                    | x::EventMask::BUTTON_PRESS
                    | x::EventMask::BUTTON_RELEASE
                    | x::EventMask::BUTTON_MOTION,
            ),
        ];

        self.conn.send_request(&x::CreateWindow {
            depth: x::COPY_FROM_PARENT as u8,
            wid: window_id,
            parent: root,
            x: geometry.position.x as i16,
            y: geometry.position.y as i16,
            width: geometry.size.width() as u16,
            height: geometry.size.height() as u16,
            border_width: 0,
            class: x::WindowClass::InputOutput,
            visual: x::COPY_FROM_PARENT,
            value_list: &values,
        });

        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        Ok(window_id)
    }

    pub fn map_window(&self, window: x::Window) -> Result<(), X11Error> {
        self.conn.send_request(&x::MapWindow { window });
        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;
        Ok(())
    }

    pub fn wait_for_event(&self) -> Result<xcb::Event, X11Error> {
        self.conn
            .wait_for_event()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))
    }

    pub fn put_image(
        &self,
        window: x::Window,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), X11Error> {
        self.conn.send_request(&x::PutImage {
            format: x::ImageFormat::ZPixmap,
            drawable: x::Drawable::Window(window),
            gc: self.get_or_create_gc(window)?,
            width: width as u16,
            height: height as u16,
            dst_x: 0,
            dst_y: 0,
            left_pad: 0,
            depth: 24,
            data,
        });

        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;
        Ok(())
    }

    pub fn destroy_window(&self, window: x::Window) -> Result<(), X11Error> {
        self.conn.send_request(&x::DestroyWindow { window });
        self.conn
            .flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;
        Ok(())
    }

    fn get_or_create_gc(&self, window: x::Window) -> Result<x::Gcontext, X11Error> {
        let id = window.resource_id();

        // Tenta pegar do cache
        if let Some(&gc) = self.gc_cache.borrow().get(&id) {
            return Ok(gc);
        }

        // Não existe, cria novo
        let gc = self.conn.generate_id();
        self.conn.send_request(&x::CreateGc {
            cid: gc,
            drawable: x::Drawable::Window(window),
            value_list: &[],
        });

        // Adiciona ao cache
        self.gc_cache.borrow_mut().insert(id, gc);

        Ok(gc)
    }

    /// Acesso à conexão X11 interna (para event loop e operações avançadas)
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}
