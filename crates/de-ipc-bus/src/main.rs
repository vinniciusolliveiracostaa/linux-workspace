//! IPC Bus — Servidor Central de Mensagens
//!
//! # Responsabilidade
//! - Aceitar conexões de componentes (WM, Compositor, Panel, etc)
//! - Rotear mensagens entre componentes
//! - Detectar componentes mortos via heartbeat
//!
//! # Arquitetura
//! Este é um processo standalone que deve ser iniciado ANTES dos outros componentes.
//! Todos os componentes conectam a ele via Unix Domain Socket.
//!
//! # Lifecycle
//! 1. Bind no socket `/tmp/de-ipc.sock`
//! 2. Loop infinito aceitando conexões
//! 3. Ao receber SIGTERM/SIGINT: graceful shutdown
//!
//! # Como usar
//! Terminal 1: cargo run --bin de-ipc-bus
//! Terminal 2: cargo run --bin wm
//! Terminal 3: cargo run --bin de-dummy

use de_ipc::IpcBus;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::path::PathBuf;
use tokio::select;
use tokio_stream::StreamExt;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configurar logging
    // POR QUE with_env_filter? Para permitir controlar log level via variável de ambiente
    // Exemplo: RUST_LOG=debug cargo run --bin de-ipc-bus
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("🚀 IPC Bus starting...");

    // 2. Definir caminho do socket
    // POR QUE /tmp? É o diretório padrão para sockets Unix temporários
    // POR QUE .sock? Convenção para identificar Unix Domain Sockets
    let socket_path = PathBuf::from("/tmp/de-ipc.sock");

    // 3. Criar IPC Bus
    // POR QUE Arc<Mutex<>>? Porque vamos compartilhar entre tasks
    // - Loop principal precisa chamar accept_client()
    // - Heartbeat monitor precisa chamar check_timeouts()
    // Arc = compartilhamento thread-safe, Mutex = acesso exclusivo
    let bus = IpcBus::new(socket_path.clone()).await?;
    let bus = std::sync::Arc::new(tokio::sync::Mutex::new(bus));

    info!("✅ IPC Bus listening on {:?}", socket_path);
    info!("💡 Press Ctrl+C to shutdown gracefully");

    // 4. Configurar signal handlers (Ctrl+C, SIGTERM)
    // POR QUE? Para fazer shutdown gracioso ao invés de matar o processo abruptamente
    let mut signals = Signals::new(&[SIGTERM, SIGINT])?;

    // 5. Spawnar task de monitoramento de heartbeat
    // POR QUE clone()? Porque Arc permite múltiplos owners
    // Cada clone incrementa o reference count
    let bus_for_heartbeat = std::sync::Arc::clone(&bus);
    tokio::spawn(async move {
        heartbeat_monitor(bus_for_heartbeat).await;
    });

    // 6. Loop principal: aceitar conexões OU aguardar signal
    // POR QUE select!? Para reagir ao primeiro evento que acontecer:
    // - Nova conexão de cliente
    // - Signal de shutdown (Ctrl+C)
    loop {
        select! {
            // Branch 1: Nova conexão de cliente
            // POR QUE lock().await? Para obter acesso exclusivo ao bus
            result = async {
                let mut bus_guard = bus.lock().await;
                bus_guard.accept_client().await
            } => {
                match result {
                    Ok(()) => {
                        // Cliente conectado com sucesso
                        // O log já foi feito dentro de accept_client()
                    }
                    Err(e) => {
                        error!("Failed to accept client: {}", e);
                        // POR QUE não break? Porque um erro em um cliente
                        // não deve derrubar o bus inteiro
                    }
                }
            }

            // Branch 2: Signal recebido (Ctrl+C ou SIGTERM)
            Some(signal) = signals.next() => {
                match signal {
                    SIGTERM | SIGINT => {
                        info!("🛑 Received shutdown signal, stopping...");
                        break;
                    }
                    _ => {
                        warn!("Received unexpected signal: {}", signal);
                    }
                }
            }
        }
    }

    // 7. Graceful shutdown
    info!("👋 IPC Bus shutting down...");

    // POR QUE não precisamos fazer nada aqui?
    // - O Drop trait de IpcBus já remove o socket file
    // - As tasks de clientes terminam quando o bus é dropped
    // - Rust RAII (Resource Acquisition Is Initialization) cuida disso

    info!("✅ IPC Bus stopped cleanly");

    Ok(())
}

/// Task de monitoramento de heartbeat
///
/// # Responsabilidade
/// - A cada 10s, verificar quais componentes não enviaram heartbeat
/// - Logar componentes mortos
///
/// # POR QUE 10s?
/// - Heartbeat é enviado a cada 5s
/// - Timeout é 15s
/// - Verificar a cada 10s é um meio-termo razoável
///
/// # POR QUE Arc<Mutex<IpcBus>>?
/// - Arc: permite compartilhar ownership entre tasks
/// - Mutex: garante acesso exclusivo (só uma task por vez)
async fn heartbeat_monitor(bus: std::sync::Arc<tokio::sync::Mutex<IpcBus>>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

    loop {
        interval.tick().await;

        // POR QUE lock()? Porque bus é compartilhado entre tasks
        // POR QUE await? Porque lock() é async (pode bloquear)
        let bus_guard = bus.lock().await;

        // Verificar timeouts
        let dead_components = bus_guard.check_timeouts().await;

        if !dead_components.is_empty() {
            warn!("💀 Dead components detected: {:?}", dead_components);
            // TODO (Slice 9): Aqui o Session Manager seria notificado
            // para reiniciar os componentes mortos
        }
    }
}
