//! Window Manager — Entry Point

use de_ipc::{IpcClient, Message};
use de_x11::{X11Connection, X11EventLoop};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use wm::EventDispatcher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_target(false).init();

    info!("Connecting to X11...");
    let x11 = X11Connection::connect()?;

    // Tentar conectar ao IPC Bus (opcional — WM funciona sem IPC)
    let ipc_tx = match setup_ipc().await {
        Ok(tx) => {
            info!("IPC connected");
            Some(tx)
        }
        Err(e) => {
            warn!(error = %e, "IPC not available, running standalone");
            None
        }
    };

    let mut dispatcher = EventDispatcher::new(&x11, 4, ipc_tx)?;
    dispatcher.register_as_wm()?;

    info!("Window Manager iniciado com 4 workspaces");

    let event_loop = X11EventLoop::new(&x11);
    event_loop.run(&mut dispatcher)?;

    Ok(())
}

/// Configura IPC Client e spawna task de envio em background
///
/// POR QUE função separada?
/// - Separação de responsabilidades: main() orquestra, setup_ipc() configura
/// - Permite retornar Result (main trata como opcional)
async fn setup_ipc() -> Result<mpsc::UnboundedSender<Message>, Box<dyn std::error::Error>> {
    let socket_path = Path::new("/tmp/de-ipc.sock");

    let (client, _rx) = IpcClient::connect(socket_path, de_ipc::ComponentType::WindowManager).await?;
    let client = Arc::new(client);

    // Iniciar heartbeat
    client.start_heartbeat();

    // Criar canal unbounded para bridge sync→async
    // POR QUE unbounded?
    // - send() é sync (pode ser chamado sem .await)
    // - O event loop X11 é sync (EventHandler trait)
    // - Unbounded não bloqueia — mensagens ficam em memória até serem consumidas
    let (tx, mut ipc_rx) = mpsc::unbounded_channel::<Message>();

    // Task que consome o canal e envia ao IPC Bus
    tokio::spawn(async move {
        while let Some(msg) = ipc_rx.recv().await {
            if let Err(e) = client.send(msg).await {
                error!(error = %e, "Failed to send IPC message");
            }
        }
        info!("IPC sender task ended");
    });

    Ok(tx)
}
