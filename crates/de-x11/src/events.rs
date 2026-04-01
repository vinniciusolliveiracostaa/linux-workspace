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

pub struct X11EventLoop<'a> {
    conn: &'a X11Connection,
}

impl<'a> X11EventLoop<'a> {
    /// Registra o WM para interceptar requests de janelas
    pub fn subclassify_root(&self) -> Result<(), X11Error> {
        let root = self.conn.root_window()?;
        let conn = self.conn.connection();

        // Tenta mudar event mask do root window
        conn.send_request(&x::ChangeWindowAttributes {
            window: root,
            value_list: &[x::Cw::EventMask(
                x::EventMask::SUBSTRUCTURE_REDIRECT
                    | x::EventMask::SUBSTRUCTURE_NOTIFY
                    | x::EventMask::BUTTON_PRESS
                    | x::EventMask::STRUCTURE_NOTIFY
                    | x::EventMask::PROPERTY_CHANGE,
            )],
        });

        // flush e verificar erro
        conn.flush()
            .map_err(|e| X11Error::ProtocolError(e.to_string()))?;

        // Verificar se outro WM já está rodando
        if let Err(e) = conn.has_error() {
            return Err(X11Error::ProtocolError(format!(
                "Another window manager is already running: {}",
                e
            )));
        }

        Ok(())
    }

    pub fn run<H: EventHandler>(&self, handler: &mut H) -> Result<(), X11Error> {
        loop {
            let event = self.conn.wait_for_event()?;

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

    pub fn new(conn: &'a X11Connection) -> Self {
        Self { conn }
    }
}
