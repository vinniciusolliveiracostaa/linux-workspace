//! Window Manager — Entry Point

use de_x11::{X11Connection, X11EventLoop};
use tracing::info;
use wm::EventDispatcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar tracing subscriber
    tracing_subscriber::fmt().with_target(false).init();

    info!("Connecting to X11...");
    let x11 = X11Connection::connect()?;

    // Criar dispatcher com 4 workspaces
    let mut dispatcher = EventDispatcher::new(&x11, 4);

    // Setup: registrar como WM antes de iniciar o loop
    dispatcher.register_as_wm()?;

    info!("Window Manager iniciado com 4 workspaces");

    let event_loop = X11EventLoop::new(&x11);

    event_loop.run(&mut dispatcher)?;

    Ok(())
}
