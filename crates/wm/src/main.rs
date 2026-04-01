// wm — Window Manager
mod manager;

use manager::WindowManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Criar um WM com 4 workspaces
    let mut wm = WindowManager::new(4)?;

    println!("Window Manager iniciado com {} workspaces", 4);

    // Rodar event loop
    wm.run()?;

    Ok(())
}
