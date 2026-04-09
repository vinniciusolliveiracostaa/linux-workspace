use crate::{X11Connection, X11Error};
use xcb::x;

/// Trait que o Window Manager implementa para receber eventos X11
pub trait EventHandler {
    /// Janela quer ser mapeada (exibida na tela)
    fn handle_map_request(&mut self, window: x::Window);

    /// Janela foi desmapeada (escondida)
    fn handle_unmap_notify(&mut self, window: x::Window);

    /// Janela quer mudar tamanho/posição
    fn handle_configure_request(&mut self, event: &x::ConfigureRequestEvent);

    /// Janela foi destruída
    fn handle_destroy_notify(&mut self, window: x::Window);

    /// Botão do mouse pressionado
    fn handle_button_press(&mut self, event: &x::ButtonPressEvent);

    /// Botão do mouse solto
    fn handle_button_release(&mut self, event: &x::ButtonReleaseEvent);

    /// Mouse moveu
    fn handle_motion_notify(&mut self, event: &x::MotionNotifyEvent);

    /// Tecla pressionada
    fn handle_key_press(&mut self, event: &x::KeyPressEvent);

    /// Mouse entrou em uma janela
    fn handle_enter_notify(&mut self, window: x::Window);

    /// Propriedade de janela mudou
    fn handle_property_notify(&mut self, event: &x::PropertyNotifyEvent);
}

pub struct X11EventLoop;

impl X11EventLoop {
    pub fn new() -> Self {
        Self
    }

    pub fn run<H: EventHandler>(
        &self,
        conn: &X11Connection,
        handler: &mut H,
    ) -> Result<(), X11Error> {
        loop {
            let event = conn.wait_for_event()?;

            use tracing::debug;
            debug!("📨 X11 Event received: {:?}", event);

            match event {
                xcb::Event::X(x::Event::MapRequest(e)) => {
                    handler.handle_map_request(e.window());
                }
                xcb::Event::X(x::Event::UnmapNotify(e)) => {
                    handler.handle_unmap_notify(e.window());
                }
                xcb::Event::X(x::Event::ConfigureRequest(e)) => {
                    handler.handle_configure_request(&e);
                }
                xcb::Event::X(x::Event::DestroyNotify(e)) => {
                    handler.handle_destroy_notify(e.window());
                }
                xcb::Event::X(x::Event::ButtonPress(e)) => {
                    handler.handle_button_press(&e);
                }
                xcb::Event::X(x::Event::ButtonRelease(e)) => {
                    handler.handle_button_release(&e);
                }
                xcb::Event::X(x::Event::MotionNotify(e)) => {
                    handler.handle_motion_notify(&e);
                }
                xcb::Event::X(x::Event::KeyPress(e)) => {
                    handler.handle_key_press(&e);
                }
                xcb::Event::X(x::Event::EnterNotify(e)) => {
                    handler.handle_enter_notify(e.event());
                }
                xcb::Event::X(x::Event::PropertyNotify(e)) => {
                    handler.handle_property_notify(&e);
                }
                _ => {
                    // Ignorar eventos não tratados
                }
            }
        }
    }
}
