use crate::error::X11Error;
use macrde_core::Rectangle;
use xcb::x;

pub struct X11Connection {
    conn: xcb::Connection,
    screen_num: i32,
    root: x::Window,
}

impl X11Connection {
    pub fn connect() -> Result<Self, X11Error> {
        let (conn, screen_num) = xcb::Connection::connect(None)?;
        let setup = conn.get_setup();
        let screen = setup
            .roots()
            .nth(screen_num as usize)
            .ok_or(X11Error::ResourceMissing)?;
        let root = screen.root();

        Ok(Self {
            conn,
            screen_num,
            root,
        })
    }

    pub fn root_window(&self) -> x::Window {
        self.root
    }

    pub fn screen_geometry(&self) -> Option<Rectangle> {
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(self.screen_num as usize)?;
        Some(Rectangle::from_xywh(
            0,
            0,
            screen.width_in_pixels() as u32,
            screen.height_in_pixels() as u32,
        )?)
    }

    pub fn create_window(&self, geometry: Rectangle) -> Result<x::Window, X11Error> {
        let window = self.conn.generate_id();
        let screen = self
            .conn
            .get_setup()
            .roots()
            .nth(self.screen_num as usize)
            .ok_or(X11Error::ResourceMissing)?;

        let event_mask = x::EventMask::EXPOSURE
            | x::EventMask::KEY_PRESS
            | x::EventMask::KEY_RELEASE
            | x::EventMask::BUTTON_PRESS
            | x::EventMask::BUTTON_RELEASE
            | x::EventMask::POINTER_MOTION
            | x::EventMask::STRUCTURE_NOTIFY;

        self.conn.send_request(&x::CreateWindow {
            depth: 0,
            wid: window,
            parent: self.root,
            x: geometry.position.x as i16,
            y: geometry.position.y as i16,
            width: geometry.size.width() as u16,
            height: geometry.size.height() as u16,
            border_width: 0,
            class: x::WindowClass::InputOutput,
            visual: screen.root_visual(),
            value_list: &[
                x::Cw::BackPixel(screen.white_pixel()),
                x::Cw::EventMask(event_mask),
            ],
        });

        self.conn.flush()?;
        Ok(window)
    }

    pub fn map_window(&self, window: x::Window) -> Result<(), X11Error> {
        self.conn.send_request(&x::MapWindow { window });
        self.conn.flush()?;
        Ok(())
    }

    pub fn wait_for_event(&self) -> Result<xcb::Event, X11Error> {
        Ok(self.conn.wait_for_event()?)
    }

    pub fn flush(&self) -> Result<(), X11Error> {
        self.conn.flush()?;
        Ok(())
    }
}
