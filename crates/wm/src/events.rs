use anyhow::Result;
use macrde_core::{Rectangle, WindowId};
use macrde_x11::xcb;
use macrde_x11::xcb::Xid;
use tracing::info;

use crate::WindowManager;
use crate::placement::{PlacementStrategy, compute_placement};

impl WindowManager {
    pub async fn handle_x11_event(&mut self, event: xcb::Event) -> Result<()> {
        match event {
            xcb::Event::X(xcb::x::Event::MapRequest(ev)) => {
                let window = ev.window();
                let attrs = self.x11.get_window_attributes(window)?;
                if attrs.override_redirect() {
                    // Janelas override-redirect (popups, menus) não são gerenciadas
                    self.x11.map_window(window)?;
                    return Ok(());
                }

                let geom = self.x11.get_window_geometry(window)?;
                let window_id = WindowId(window.resource_id());

                let placement = compute_placement(
                    &self.service,
                    PlacementStrategy::default(),
                    self.x11.screen_geometry().unwrap(),
                    geom.size,
                );
                let final_geom = Rectangle::new(placement, geom.size);

                let cmd = self.service.add_window(window_id, final_geom);
                self.send_command(cmd).await?;

                // Configura a janela para ser redirecionada e mapeada
                self.x11.configure_window(window, final_geom)?;
                self.x11.map_window(window)?;
                info!("Managed new window {:?}", window_id);
            }
            xcb::Event::X(xcb::x::Event::UnmapNotify(ev)) => {
                let window_id = WindowId(ev.window().resource_id());
                if let Some(cmd) = self.service.remove_window(window_id) {
                    self.send_command(cmd).await?;
                    info!("Unmanaged window {:?}", window_id);
                }
            }
            xcb::Event::X(xcb::x::Event::DestroyNotify(ev)) => {
                let window_id = WindowId(ev.window().resource_id());
                if let Some(cmd) = self.service.remove_window(window_id) {
                    self.send_command(cmd).await?;
                    info!("Window destroyed {:?}", window_id);
                }
            }
            xcb::Event::X(xcb::x::Event::ConfigureRequest(ev)) => {
                // Honramos pedidos de redimensionamento, mas mantemos controle
                let window = ev.window();
                let geom = Rectangle::from_xywh(
                    ev.x() as i32,
                    ev.y() as i32,
                    ev.width() as u32,
                    ev.height() as u32,
                )
                .unwrap();
                self.x11.configure_window(window, geom)?;
            }
            xcb::Event::X(xcb::x::Event::EnterNotify(ev)) => {
                let window_id = WindowId(ev.event().resource_id());
                if let Some(cmd) = self.service.focus_window(window_id) {
                    self.send_command(cmd).await?;
                }
            }
            xcb::Event::X(xcb::x::Event::KeyPress(ev)) => {
                use crate::hotkeys::HotkeyManager;
                use macrde_x11::xcb::XidNew;
                let hotkeys = HotkeyManager::new();

                if let Some(action) = hotkeys.lookup(ev.state().bits() as u16, ev.detail()) {
                    match action {
                        crate::hotkeys::WmAction::CloseWindow => {
                            if let Some(focused) = self.service.focused_window() {
                                let xwin = xcb::x::Window::new(focused.0);
                                self.x11.kill_client(xwin)?;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
