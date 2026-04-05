//! IPC Server — Gerencia conexões de clientes via Unix Domain Sockets
//!
//! # Arquitetura
//! - IpcBus: servidor central que aceita conexões
//! - Cada cliente se identifica com ComponentType ao conectar
//! - Mensagens são serializadas com bincode (formato: [4 bytes len][payload])
//!
//! # POR QUE Unix Domain Sockets?
//! - Mais rápido que TCP (~1ms vs ~5ms latência)
//! - Não passa pela stack de rede (sem overhead de IP/TCP)
//! - Permissões de filesystem (segurança)
//! - Suporte a passing de file descriptors (futuro)

use crate::protocol::{ComponentType, Message};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// Erro do IPC Bus
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Client {0:?} not found")]
    ClientNotFound(ComponentType),

    #[error("Invalid message format")]
    InvalidMessage,
}

/// IPC Bus — Servidor central de mensagens
///
/// # Responsabilidades
/// - Aceitar conexões de clientes
/// - Rotear mensagens entre componentes
/// - Detectar desconexões
///
/// # NÃO faz
/// - Lógica de negócio (isso é responsabilidade dos componentes)
/// - Validação de mensagens (componentes validam)
pub struct IpcBus {
    /// Caminho do socket Unix
    socket_path: PathBuf,

    /// Listener para aceitar novas conexões
    listener: UnixListener,

    /// Clientes conectados: ComponentType -> canal de envio
    /// POR QUE mpsc::Sender?
    /// - Permite enviar mensagens de forma assíncrona
    /// - Cada cliente tem sua própria task de envio
    /// - Evita bloqueio se um cliente estiver lento
    clients: Arc<RwLock<HashMap<ComponentType, mpsc::Sender<Message>>>>,
}

impl IpcBus {
    /// Cria um novo IPC Bus
    ///
    /// # POR QUE async?
    /// - UnixListener::bind é uma operação de I/O
    /// - Pode falhar (socket já existe, permissões, etc.)
    ///
    /// # POR QUE remover socket existente?
    /// - Se o processo anterior crashou, o socket fica órfão
    /// - Remover antes de bind evita erro "Address already in use"
    pub async fn new(socket_path: PathBuf) -> Result<Self, IpcError> {
        // Remove socket existente (se houver)
        let _ = tokio::fs::remove_file(&socket_path).await;

        let listener = UnixListener::bind(&socket_path)?;
        info!("IPC Bus listening on {:?}", socket_path);

        Ok(Self {
            socket_path,
            listener,
            clients: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Aceita uma nova conexao de Cliente
    ///
    /// Protocolo de Handshake
    /// 1. Cliente conecta
    /// 2. Cliente envia ComponentType (1 byte)
    /// 3. Servidor registra cliente no HashMap
    /// 4. Servidor spawna task de leitura para esse cliente
    pub async fn accept_client(&mut self) -> Result<(), IpcError> {
        let (stream, _addr) = self.listener.accept().await?;
        debug!("New client connected");

        // Handshake: ler ComponentType (1 byte)
        let mut stream = stream;
        let component_type = Self::read_component_type(&mut stream).await?;
        info!("Client identified as {:?}", component_type);

        // Criar canal mpsc para enviar mensagens para esse cliente
        let (tx, rx) = mpsc::channel::<Message>(100);

        // Registrar cliente
        {
            let mut clients = self.clients.write().await;
            clients.insert(component_type, tx);
        } // RwLock é liberado aqui (RAII)

        // Dividir stream em read half e write half
        let (read_half, write_half) = stream.into_split();

        // Swawnar task de leitura (recebe mensagens do cliente)
        let clients_clone = Arc::clone(&self.clients);
        tokio::spawn(async move {
            Self::handle_client_read(read_half, component_type, clients_clone).await;
        });

        // Spawnar task de escrita (envia mensagens para o cliente)
        let clients_for_cleanup = Arc::clone(&self.clients);
        tokio::spawn(async move {
            Self::handle_client_write(write_half, component_type, rx, clients_for_cleanup).await;
        });

        Ok(())
    }

    /// Lê o ComponentType do cliente (handshake)
    ///
    /// # Formato
    /// - 1 byte: ComponentType como u8
    async fn read_component_type(stream: &mut UnixStream) -> Result<ComponentType, IpcError> {
        let mut buf = [0u8; 1];
        stream.read_exact(&mut buf).await?;

        // Deserialiazar ComponentType
        bincode::deserialize(&buf).map_err(|_| IpcError::InvalidMessage)
    }

    /// Task de leitura de um cliente
    ///
    /// # Responsabilidades
    /// - Ler mensagens do cliente
    /// - Rotear mensagens para outros clientes (broadcast ou unicast)
    /// - Detectar desconexão
    ///
    /// # Formato de Mensagem
    /// [4 bytes length (little-endian u32)][payload bincode]
    async fn handle_client_read(
        mut read_half: OwnedReadHalf,
        component: ComponentType,
        clients: Arc<RwLock<HashMap<ComponentType, mpsc::Sender<Message>>>>,
    ) {
        loop {
            match Self::read_message(&mut read_half).await {
                Ok(msg) => {
                    debug!("Received message from {:?}: {:?}", component, msg);

                    // Rotear mensagem (broadcast para todos os clientes)
                    let clients_guard = clients.read().await;
                    for (client_type, tx) in clients_guard.iter() {
                        // Não enviar de volta para o remetente
                        if *client_type == component {
                            continue;
                        }

                        if let Err(e) = tx.send(msg.clone()).await {
                            warn!("Failed to route message to {:?}: {}", client_type, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from {:?}: {}", component, e);
                    break;
                }
            }
        }
        info!("Client {:?} read task ended", component);
    }

    async fn handle_client_write(
        mut write_half: OwnedWriteHalf,
        component: ComponentType,
        mut rx: mpsc::Receiver<Message>,
        clients: Arc<RwLock<HashMap<ComponentType, mpsc::Sender<Message>>>>,
    ) {
        // Loop de escrita
        while let Some(msg) = rx.recv().await {
            if let Err(e) = Self::write_message(&mut write_half, &msg).await {
                error!("Failed to send message to {:?}: {}", component, e);
                break;
            }
        }

        // Cliente desconectou — remover do HashMap
        // POR QUE write() aqui?
        // - Estamos modificando o HashMap (removendo entrada)
        // - write() garante acesso exclusivo
        {
            let mut clients_guard = clients.write().await;
            clients_guard.remove(&component);
            info!("Client {:?} removed from registry", component);
        }

        info!("Client {:?} write task ended", component);
    }

    /// Lê uma mensagem do stream
    ///
    /// # Formato
    /// [4 bytes length][payload]
    async fn read_message(read_half: &mut OwnedReadHalf) -> Result<Message, IpcError> {
        // Ler length (4 bytes little-endian)
        let mut len_buf = [0u8; 4];
        read_half.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;

        // Ler payload
        let mut payload = vec![0u8; len];
        read_half.read_exact(&mut payload).await?;

        // Deserializar
        bincode::deserialize(&payload).map_err(|e| e.into())
    }

    /// Escreve uma mensagem no stream
    ///
    /// # Formato
    /// [4 bytes length][payload]
    async fn write_message(write_half: &mut OwnedWriteHalf, msg: &Message) -> Result<(), IpcError> {
        // Serializar mensagem
        let payload = bincode::serialize(msg)?;
        let len = payload.len() as u32;

        // Escrever length (4 bytes little-endian)
        write_half.write_all(&len.to_le_bytes()).await?;

        // Escrever payload
        write_half.write_all(&payload).await?;

        Ok(())
    }

    /// Envia mensagem para um cliente específico
    ///
    /// # POR QUE async?
    /// - mpsc::Sender::send é async
    /// - Pode bloquear se o buffer do canal estiver cheio
    pub async fn send(&self, target: ComponentType, msg: Message) -> Result<(), IpcError> {
        let clients = self.clients.read().await;
        let tx = clients
            .get(&target)
            .ok_or(IpcError::ClientNotFound(target))?;

        tx.send(msg)
            .await
            .map_err(|_| IpcError::ClientNotFound(target))
    }

    /// Broadcast: envia mensagem para todos os clientes
    ///
    /// # POR QUE broadcast?
    /// - Eventos de sistema (WindowCreated, ConfigChanged) interessam a todos
    /// - Evita lógica de routing complexa
    pub async fn broadcast(&self, msg: Message) {
        let clients = self.clients.read().await;
        for (client_type, tx) in clients.iter() {
            if let Err(e) = tx.send(msg.clone()).await {
                warn!("Failed to broadcast to {:?}: {}", client_type, e);
            }
        }
    }
}

impl Drop for IpcBus {
    fn drop(&mut self) {
        // Remover socket ao desligar
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
