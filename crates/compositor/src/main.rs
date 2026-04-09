use anyhow::Result;
use macrde_core::{Color, Position, Rectangle, Size};
use macrde_render::MockRenderer;
use macrde_render::Renderer; // Import the trait to use its methods
use macrde_x11::X11Connection;
use xcb::Event;
use xcb::Xid;
use xcb::x;

fn main() -> Result<()> {
    // Inicialize logging
    tracing_subscriber::fmt::init();

    // 1. Conecta ao X11
    let x11 = X11Connection::connect()?;
    let screen_geom = x11
        .screen_geometry()
        .expect("Failed to get screen geometry");

    // 2. Cria uma janela de compositor (janela de overlay/dock, para testes)
    let window_geometry =
        Rectangle::from_xywh(100, 100, 800, 600).expect("Invalid window geometry");
    let window = x11.create_window(window_geometry)?;
    x11.map_window(window)?;

    // 3. Inicializa o renderizador mock
    // Em um cenário real, precisaríamos de um `HasWindowHandle` para o window do X11.
    // Como estamos usando MockRenderer, isso não é necessário.
    let mut renderer = MockRenderer::default();
    // Nota: o MockRenderer ignora o handle, então passamos um dummy.
    // (Na implementação real com wgpu, passaríamos um RawWindowHandle extraído do x11::Window)
    renderer.init(&())?;
    renderer.resize(window_geometry.size)?;

    tracing::info!("Compositor started. Window 0x{:x}", window.resource_id());

    // 4. Loop de eventos
    loop {
        // Processa eventos X11
        let event = x11.wait_for_event()?;
        match event {
            Event::X(x::Event::Expose(_)) => {
                // A cada Expose, desenhamos um quadro
                renderer.clean(Color::WINDOW_BACKGROUND);
                // Desenha um retângulo vermelho de exemplo
                let rect = Rectangle::from_xywh(50, 50, 200, 100).unwrap();
                renderer.draw_rect(rect, Color::rgb(255, 0, 0), 8.0);
                renderer.present()?;
                tracing::debug!("Frame rendered");
            }
            Event::X(x::Event::KeyPress(e)) => {
                // Sai com ESC (keycode 9)
                if e.detail() == 9 {
                    tracing::info!("ESC pressed, exiting.");
                    break;
                }
            }
            _ => {}
        }

        // Aqui também processaríamos mensagens IPC com tokio::select!
        // Por enquanto, apenas o loop de eventos X11.
    }

    Ok(())
}
