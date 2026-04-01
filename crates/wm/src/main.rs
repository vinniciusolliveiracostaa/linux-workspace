// wm — Window Manager
mod manager;
mod placement;

use manager::WindowManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Criar um WM com 4 workspaces
    let mut wm = WindowManager::new(4)?;

    println!("Window Manager iniciado com {} workspaces", 4);

    // Rodar event loop
    wm.run()?;

    Ok(())
}
