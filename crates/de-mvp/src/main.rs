// de-mvp — MVP de integração (Slice 0)
use de_core::{Color, Rectangle};
use de_render::SoftwareRenderer;
use de_x11::X11Connection;
use xcb::{x, Event};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Conectar ao X11
    let x11 = X11Connection::connect()?;
    let screen = x11.screen_geometry()?;

    println!(
        "Conectado ao X11. Screen: {}x{}",
        screen.size.width(),
        screen.size.height()
    );

    // 2. Criar janela 800x600 centralizada
    let window_width = 800;
    let window_height = 600;
    let window_x = (screen.size.width() as i32 - window_width as i32) / 2;
    let window_y = (screen.size.height() as i32 - window_height as i32) / 2;

    let window_geometry = Rectangle::new(window_x, window_y, window_width, window_height);
    let window = x11.create_window(window_geometry)?;
    x11.map_window(window)?;

    println!("Janela criada: {:?}", window);

    // 3. Criar renderer
    let mut renderer =
        SoftwareRenderer::new(window_width, window_height).ok_or("Failed to create renderer")?;

    // 4. Event loop
    loop {
        let event = x11.wait_for_event()?;

        match event {
            Event::X(x::Event::Expose(_)) => {
                // Renderizar retângulo azul centralizado 400x300
                renderer.clear(Color::rgb(236, 236, 236)); // #ECECEC (background macOS)

                let rect = Rectangle::new(200, 150, 400, 300);
                let blue = Color::rgb(0, 122, 255); // #007AFF (accent macOS)
                renderer.draw_rectangle(rect, blue, 0.0);

                // Enviar para X11
                let buffer = renderer.get_buffer_bgra();
                x11.put_image(window, &buffer, window_width, window_height)?;

                println!("Frame renderizado");
            }

            Event::X(x::Event::KeyPress(e)) => {
                // ESC fecha a janela
                if e.detail() == 9 {
                    // keycode 9 = ESC
                    println!("ESC pressionado, fechando...");
                    break;
                }
            }
            _ => {}
        }
    }

    x11.destroy_window(window)?;
    println!("Janela fechada");

    Ok(())
}
