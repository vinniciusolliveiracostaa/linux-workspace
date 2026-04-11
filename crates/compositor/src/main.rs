use anyhow::Result;
use macrde_core::{Color, Rectangle};
use macrde_ipc::CompositorCommand;
use macrde_render::{MockRenderer, Renderer};
use macrde_x11::xcb::Xid;
use macrde_x11::{X11Connection, xcb};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

struct Compositor {
    x11: Arc<X11Connection>,
    renderer: MockRenderer,
    listener: UnixListener,
    client: Option<UnixStream>,
}

impl Compositor {
    async fn new(x11: Arc<X11Connection>, renderer: MockRenderer) -> Result<Self> {
        let socket_path = "/tmp/macrde-compositor.sock";
        // Remove socket antigo se existir
        let _ = std::fs::remove_file(socket_path);
        let listener = UnixListener::bind(socket_path)?;
        info!("IPC server listening on {}", socket_path);

        Ok(Self {
            x11,
            renderer,
            listener,
            client: None,
        })
    }

    async fn accept_client(&mut self) -> Result<()> {
        info!("Waiting for WM to connect...");
        let (stream, _) = self.listener.accept().await?;
        self.client = Some(stream);
        info!("WM connected");
        Ok(())
    }

    async fn handle_ipc_message(&mut self, cmd: CompositorCommand) -> Result<()> {
        match cmd {
            CompositorCommand::AddWindow {
                window_id,
                geometry,
            } => {
                info!("AddWindow: {:?} at {:?}", window_id, geometry);
                // Aqui futuramente adicionaremos ao scene graph
                // Por enquanto, apenas logamos e forçamos um redraw
                self.renderer.clear(Color::WINDOW_BACKGROUND);
                // Desenha um placeholder para a janela
                self.renderer
                    .draw_rect(geometry, Color::rgb(100, 100, 200), 8.0);
                self.renderer.present()?;
            }
            CompositorCommand::RemoveWindow { window_id } => {
                info!("RemoveWindow: {:?}", window_id);
                // Redesenha sem essa janela
                self.renderer.clear(Color::WINDOW_BACKGROUND);
                self.renderer.present()?;
            }
            CompositorCommand::MoveWindow {
                window_id,
                geometry,
            } => {
                info!("MoveWindow: {:?} to {:?}", window_id, geometry);
                self.renderer.clear(Color::WINDOW_BACKGROUND);
                self.renderer
                    .draw_rect(geometry, Color::rgb(100, 100, 200), 8.0);
                self.renderer.present()?;
            }
            CompositorCommand::FocusWindow { window_id } => {
                info!("FocusWindow: {:?}", window_id);
                // Atualiza borda de foco futuramente
            }
        }
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        // Primeiro aguarda a conexão do WM
        self.accept_client().await?;

        let window_geometry = Rectangle::from_xywh(100, 100, 800, 600).unwrap(); // Remover Prod!
        let window = self.x11.create_window(window_geometry)?;
        self.x11.map_window(window)?;

        self.renderer.init(800, 600)?;
        self.renderer.resize(window_geometry.size)?;
        info!("Compositor started. Window 0x{:x}", window.resource_id());

        loop {
            tokio::select! {
                // Eventos X11
                x11_res = tokio::task::spawn_blocking({
                    let x11 = self.x11.clone();
                    move || x11.wait_for_event()
                }) => {
                    match x11_res {
                        Ok(Ok(event)) => {
                            match event {
                                xcb::Event::X(xcb::x::Event::Expose(_)) => {
                                    self.renderer.clear(Color::WINDOW_BACKGROUND);
                                    self.renderer.present()?;
                                }
                                xcb::Event::X(xcb::x::Event::KeyPress(e)) => {
                                    if e.detail() == 9 { // ESC
                                        info!("ESC pressed, exiting.");
                                        break;
                                    }
                                }
                                _ => {}
                            }
                        }
                        Ok(Err(e)) => error!("X11 event error: {}", e),
                        Err(e) => error!("spawn_blocking error: {}", e),
                    }
                }

                // Mensagens IPC do cliente conectado
                ipc_res = async {
                    if let Some(ref mut stream) = self.client {
                        // Lê tamanho
                        let mut len_buf = [0u8; 4];
                        stream.read_exact(&mut len_buf).await?;
                        let len = u32::from_le_bytes(len_buf) as usize;
                        let mut buf = vec![0u8; len];
                        stream.read_exact(&mut buf).await?;
                        let (cmd, _): (CompositorCommand, _) = bincode::decode_from_slice(&buf, bincode::config::standard())?;
                        Ok(cmd)
                    } else {
                        // Se não há cliente, aguarda um pouco e tenta aceitar novamente (em caso de desconexão)
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        Err::<CompositorCommand, anyhow::Error>(anyhow::anyhow!("no client"))
                    }
                } => {
                    match ipc_res {
                        Ok(cmd) => {
                            if let Err(e) = self.handle_ipc_message(cmd).await {
                                error!("IPC handle error: {}", e);
                            }
                        }
                        Err(e) => {
                            // Se o cliente desconectou, podemos tentar aceitar um novo
                            if self.client.is_none() {
                                if let Err(e) = self.accept_client().await {
                                    error!("Failed to accept new client: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let x11 = Arc::new(X11Connection::connect()?);
    let renderer = MockRenderer::default();

    let mut compositor = Compositor::new(x11, renderer).await?;
    compositor.run().await?;

    Ok(())
}
