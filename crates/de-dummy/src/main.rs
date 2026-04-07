//! Componente Dummy — Validação de IPC Multi-Processo
//!
//! # Objetivo
//! Conectar ao IPC bus, receber mensagens, e logar no stdout.
//! Usado para validar que a comunicação entre processos funciona.
//!
//! # Como usar
//! Terminal 1: cargo run --bin de-ipc-bus (ainda não existe, mas vamos criar)
//! Terminal 2: cargo run --bin wm
//! Terminal 3: cargo run --bin de-dummy
//!
//! O dummy deve logar todas as mensagens que o WM envia via IPC.

use de_ipc::{ComponentType, IpcClient, Message};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configurar logging
    tracing_subscriber::fmt()
        .with_target(false) // Não mostrar o módulo (fica mais limpo)
        .with_level(true) // Mostrar o nível (INFO, DEBUG, etc)
        .init();

    info!("🤖 Dummy component starting...");

    // 2. Conectar ao IPC bus
    // POR QUE Path::new()? Porque IpcClient::connect espera &Path
    let socket_path = Path::new("/tmp/de-ipc.sock");

    // POR QUE ComponentType::ConfigManager?
    // Poderia ser qualquer tipo, mas vamos usar ConfigManager como exemplo
    // (o dummy não é nenhum componente real, então escolhemos um arbitrário)
    let (client, mut rx) = IpcClient::connect(socket_path, ComponentType::ConfigManager)
        .await
        .map_err(|e| {
            error!("Failed to connect to IPC bus: {}", e);
            e
        })?;

    info!("✅ Connected to IPC bus at {:?}", socket_path);

    // 3. Iniciar heartbeat
    // POR QUE Arc? Porque start_heartbeat precisa de Arc<IpcClient>
    // POR QUE? Porque a task de heartbeat precisa compartilhar ownership do client
    let client = Arc::new(client);
    let _heartbeat_handle = client.start_heartbeat();

    info!("💓 Heartbeat started (5s interval)");

    // 4. Loop principal: receber e logar mensagens
    // POR QUE while let? É o padrão idiomático em Rust para consumir um channel
    // Quando o channel fechar (IPC desconectar), o loop termina graciosamente
    info!("👂 Listening for messages...");

    while let Some(msg) = rx.recv().await {
        // POR QUE match? Para tratar cada tipo de mensagem de forma específica
        // Aqui só logamos, mas em componentes reais faríamos ações
        match msg {
            Message::WindowCreated {
                window_id,
                geometry,
            } => {
                info!(
                    "🪟 Window created: id={:?}, geometry={:?}",
                    window_id, geometry
                );
            }
            Message::WindowDestroyed { window_id } => {
                info!("❌ Window destroyed: id={:?}", window_id);
            }
            Message::WindowMoved {
                window_id,
                position,
            } => {
                info!(
                    "↔️  Window moved: id={:?}, position={:?}",
                    window_id, position
                );
            }
            Message::WindowResized { window_id, size } => {
                info!("↕️  Window resized: id={:?}, size={:?}", window_id, size);
            }
            Message::WindowFocused { window_id } => {
                info!("🎯 Window focused: id={:?}", window_id);
            }
            Message::WindowUnfocused { window_id } => {
                info!("😶 Window unfocused: id={:?}", window_id);
            }
            Message::WorkspaceChanged { from, to } => {
                info!("🔄 Workspace changed: {:?} → {:?}", from, to);
            }
            Message::ConfigChanged { key, value } => {
                info!("⚙️  Config changed: {} = {}", key, value);
            }
            Message::ThemeChanged { theme_name } => {
                info!("🎨 Theme changed: {}", theme_name);
            }
            Message::Heartbeat {
                component: _,
                timestamp: _,
            } => {
                // POR QUE não logar heartbeats? Porque são enviados a cada 5s
                // e poluiriam o log. Só logamos se quisermos debug detalhado.
                // Descomente a linha abaixo se quiser ver:
                // info!("💓 Heartbeat from {:?} at {}", component, timestamp);
            }
            Message::ComponentStarted { component } => {
                info!("🚀 Component started: {:?}", component);
            }
            Message::ComponentStopped { component } => {
                info!("🛑 Component stopped: {:?}", component);
            }
            Message::ComponentCrashed { component, error } => {
                error!("💥 Component crashed: {:?} - {}", component, error);
            }
            // POR QUE _ => ? Para capturar todas as outras mensagens
            // Isso garante que se adicionarmos novas mensagens no futuro,
            // o dummy não quebra (fail-safe)
            _ => {
                info!("📨 Other message: {:?}", msg);
            }
        }
    }

    // Se chegamos aqui, o channel foi fechado (IPC desconectou)
    info!("🔌 IPC connection closed, shutting down...");

    Ok(())
}
