//! Protocolo IPC — Definição de todas as mensagens trocadas entre componentes
//!
//! # POR QUE enum Message?
//! - Type-safe: compilador garante que só enviamos mensagens válidas
//! - Extensível: adicionar nova mensagem = adicionar variante
//! - Serialização automática: serde + bincode fazem o trabalho pesado
//!
//! # POR QUE bincode?
//! - Serialização binária compacta (~10x menor que JSON)
//! - Rápido (~5x mais rápido que JSON)
//! - Zero-copy deserialization quando possível

use de_core::{Position, Rectangle, Size, WindowId, WorkspaceId};
use serde::{Deserialize, Serialize};

/// Tipo de componente no sistema
///
/// # POR QUE enum?
/// - Identificação única de cada componente
/// - Usado para routing de mensagens
/// - Facilita logging e debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    SessionManager,
    WindowManager,
    Compositor,
    Panel,
    Launcher,
    NotificationManager,
    ConfigManager,
}

/// Urgência de notificação (freedesktop.org spec)
///
/// # Spec: org.freedesktop.Notifications
/// Valores definidos pela especificação freedesktop.org
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Urgency {
    Low = 0,
    Normal = 1,
    Critical = 2,
}

/// Razão do fechamento de notificação (freedesktop.org spec)
///
/// # Spec: org.freedesktop.Notifications
/// Valores OBRIGATÓRIOS pela especificação
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseReason {
    /// Notificação expirou (timeout)
    Expired = 1,
    /// Usuário fechou manualmente
    DismissedByUser = 2,
    /// Aplicação chamou CloseNotification
    ClosedByCall = 3,
    /// Razão indefinida
    Undefined = 4,
}

/// Mensagem IPC
///
/// # Design Pattern: Command Pattern
/// Cada variante é um comando que pode ser executado por um componente
///
/// # Serialização
/// - Formato: [4 bytes length][payload bincode]
/// - Length em little-endian u32
/// - Payload: Message serializado com bincode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // ===== Window Manager Messages =====
    /// Janela foi criada e está sendo gerenciada
    WindowCreated {
        window_id: WindowId,
        geometry: Rectangle,
    },

    /// Janela foi destruída
    WindowDestroyed { window_id: WindowId },

    /// Janela foi movida
    WindowMoved {
        window_id: WindowId,
        position: Position,
    },

    /// Janela foi redimensionada
    WindowResized { window_id: WindowId, size: Size },

    /// Janela recebeu foco
    WindowFocused { window_id: WindowId },

    /// Janela perdeu foco
    WindowUnfocused { window_id: WindowId },

    // ===== Workspace Messages =====
    /// Workspace foi trocado
    WorkspaceChanged { from: WorkspaceId, to: WorkspaceId },

    /// Janela foi movida para outro workspace
    WindowMovedToWorkspace {
        window_id: WindowId,
        workspace_id: WorkspaceId,
    },

    // ===== Configuration Messages =====
    /// Configuração foi alterada
    ConfigChanged { key: String, value: String },

    /// Tema foi alterado
    ThemeChanged { theme_name: String },

    /// Config foi recarregada (hot-reload)
    ConfigReloaded,

    // ===== Notification Messages =====
    /// Notificação recebida (freedesktop.org spec compliant)
    ///
    /// # Spec: org.freedesktop.Notifications.Notify
    /// Todos os campos seguem a especificação freedesktop.org
    /// Referência: https://specifications.freedesktop.org/notification-spec/notification-spec-latest.html
    NotificationReceived {
        id: u32,
        app_name: String,     // Nome da aplicação que enviou
        summary: String,      // Título da notificação
        body: String,         // Corpo da notificação
        urgency: Urgency,     // Low, Normal, Critical
        icon: Option<String>, // Nome do ícone ou caminho
        actions: Vec<String>, // Ações disponíveis (pares: id, label)
        timeout: i32,         // -1 = default, 0 = never, >0 = ms
    },

    /// Notificação foi fechada (freedesktop.org spec compliant)
    ///
    /// # Spec: org.freedesktop.Notifications.NotificationClosed
    NotificationClosed { id: u32, reason: CloseReason },

    /// Ação de notificação foi invocada (freedesktop.org spec compliant)
    ///
    /// # Spec: org.freedesktop.Notifications.ActionInvoked
    NotificationActionInvoked { id: u32, action_key: String },

    // ===== System Messages =====
    /// Componente iniciou
    ComponentStarted { component: ComponentType },

    /// Componente parou graciosamente
    ComponentStopped { component: ComponentType },

    /// Componente crashou
    ComponentCrashed {
        component: ComponentType,
        error: String,
    },

    /// Heartbeat (enviado a cada 5s)
    ///
    /// # POR QUE heartbeat?
    /// - Detectar componentes mortos (timeout >15s)
    /// - Session Manager pode reiniciar componentes crashados
    /// - Evita deadlocks (componente travado é detectado)
    Heartbeat {
        component: ComponentType,
        timestamp: u64, // Unix timestamp em milissegundos
    },

    // ===== Request-Response Pattern =====
    /// Request síncrono (aguarda Response)
    ///
    /// # POR QUE Request/Response?
    /// - Algumas operações precisam de resposta (ex: GetWindowList)
    /// - ID único permite correlacionar request com response
    /// - Timeout pode ser implementado no client
    Request { id: u64, request: Request },

    /// Response para um Request
    Response { id: u64, response: Response },
}

/// Request síncrono
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    /// Obter lista de janelas gerenciadas
    GetWindowList,

    /// Obter lista de workspaces
    GetWorkspaceList,

    /// Obter valor de configuração
    GetConfig { key: String },

    /// Lançar aplicação
    LaunchApplication { command: String },

    /// Trocar workspace
    SwitchWorkspace { workspace_id: WorkspaceId },

    /// Obter histórico de notificações
    GetNotificationHistory,
}

/// Response para um Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// Lista de janelas
    WindowList { windows: Vec<WindowInfo> },

    /// Lista de workspaces
    WorkspaceList { workspaces: Vec<WorkspaceInfo> },

    /// Valor de configuração
    Config { value: String },

    /// Aplicação lançada
    ApplicationLaunched { pid: u32 },

    /// Workspace trocado
    WorkspaceSwitched,

    /// Histórico de notificações
    NotificationHistory {
        notifications: Vec<NotificationInfo>,
    },

    /// Erro ao processar request
    Error { message: String },
}

/// Informação sobre uma janela (para Response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: WindowId,
    pub title: String,
    pub geometry: Rectangle,
    pub workspace_id: WorkspaceId,
    pub is_focused: bool,
}

/// Informação sobre um workspace (para Response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: WorkspaceId,
    pub name: String,
    pub window_count: usize,
    pub is_active: bool,
}

/// Informação sobre uma notificação (para Response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationInfo {
    pub id: u32,
    pub summary: String,
    pub body: String,
    pub urgency: Urgency,
    pub timestamp: u64,
}
