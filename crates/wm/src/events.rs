use anyhow::Result;
use macrde_core::{Position, Rectangle, Size, WindowId};
use macrde_ipc::CompositorCommand;
use macrde_x11::{X11Connection, xcb};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::WindowManager;
use crate::placement::{PlacementStrategy, compute_placement};
use crate::service::WindowManagerService;

impl WindowManager {
    pub async fn handle_x11_event(&mut self, event: xcb::Event) -> Result<()> {
        match event {
            xcb::Event::X(xcb::x::Event::MapRequest(ev)) => {
                let window = ev.window();
                let attrs = self.x11.get_window_attributes(window)?;
                if attrs.override_redirect() {
                    // Janelas override-redirect (popups, menus) nao sao gerenciadas
                    self.x11.map_window(window)?;
                    return Ok(());
                }
            }
        }
    }
}
