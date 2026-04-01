//! Window Manager — Entry Point

use de_x11::X11Connection;
use tracing::info;
use wm::EventDispatcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar tracing subscriber
    tracing_subscriber::fmt()
        .with_target(false) // Remove o nome do crate dos logs (mais limpo)
        .init();

    info!("Connecting to X11...");

    // Conectar ao X11
    let x11 = X11Connection::connect()?;

    info!("Creating dispatcher with 4 workspaces");

    // Criar dispatcher com 4 workspaces
    let mut dispatcher = EventDispatcher::new(&x11, 4);

    // Registrar como WM (setup separado)
    dispatcher.register_as_wm(&x11)?;

    info!("Window Manager iniciado com 4 workspaces");

    // Rodar event loop
    dispatcher.run(&x11)?;

    Ok(())
}
