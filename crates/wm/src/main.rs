//! Window Manager — Entry Point

use de_x11::X11Connection;
use wm::EventDispatcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Conectar ao X11
    let x11 = X11Connection::connect()?;

    // Criar dispatcher com 4 workspaces
    let mut dispatcher = EventDispatcher::new(&x11, 4);

    println!("Window Manager iniciado com 4 workspaces");

    // Rodar event loop
    dispatcher.run(&x11)?;

    Ok(())
}
