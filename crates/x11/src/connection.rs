use crate::error::X11Error;
use macrde_core::Rectangle;
use xcb::x::{ConfigureWindow, GetGeometry, GetWindowAttributes, KillClient};
use xcb::{Xid, x};

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

    pub fn get_window_geometry(&self, window: x::Window) -> Result<Rectangle, X11Error> {
        let cookie = self.conn.send_request(&GetGeometry {
            drawable: x::Drawable::Window(window),
        });
        let reply = self.conn.wait_for_reply(cookie)?;
        Rectangle::from_xywh(
            reply.x() as i32,
            reply.y() as i32,
            reply.width() as u32,
            reply.height() as u32,
        )
        .ok_or(X11Error::InvalidGeometry)
    }

    pub fn get_window_attributes(
        &self,
        window: x::Window,
    ) -> Result<x::GetWindowAttributesReply, X11Error> {
        let cookie = self.conn.send_request(&GetWindowAttributes { window });
        Ok(self.conn.wait_for_reply(cookie)?)
    }

    pub fn configure_window(&self, window: x::Window, geom: Rectangle) -> Result<(), X11Error> {
        let values = [
            x::ConfigWindow::X(geom.position.x),
            x::ConfigWindow::Y(geom.position.y),
            x::ConfigWindow::Width(geom.size.width()),
            x::ConfigWindow::Height(geom.size.height()),
            x::ConfigWindow::BorderWidth(0),
        ];
        self.conn.send_request(&ConfigureWindow {
            window,
            value_list: &values,
        });
        self.conn.flush()?;
        Ok(())
    }

    pub fn kill_client(&self, window: x::Window) -> Result<(), X11Error> {
        self.conn.send_request(&KillClient {
            resource: window.resource_id(),
        });
        self.conn.flush()?;
        Ok(())
    }
}
