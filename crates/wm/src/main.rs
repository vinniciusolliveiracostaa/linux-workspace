//! Window Manager — Entry Point

use de_x11::{X11Connection, X11EventLoop};
use tracing::info;
use wm::EventDispatcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_target(false).init();

    info!("Connecting to X11...");
    let x11 = X11Connection::connect()?;

    // ? propaga o erro se workspace_count == 0
    // Na prática, 4 nunca vai falhar — mas respeitamos o contrato do tipo
    let mut dispatcher = EventDispatcher::new(&x11, 4)?;

    dispatcher.register_as_wm()?;

    info!("Window Manager iniciado com 4 workspaces");

    let event_loop = X11EventLoop::new(&x11);
    event_loop.run(&mut dispatcher)?;

    Ok(())
}
