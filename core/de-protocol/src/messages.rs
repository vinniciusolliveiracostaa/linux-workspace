//! Mensagens IPC - Request/Response e Commands

use de_domain::{geometry::Rect, window::WindowId, MaximizeState, ResizeEdge};
use serde::{Deserialize, Serialize};

/// Identificador único de mensagem
pub type MessageId = u64;

// ============================================
// WM -> COMPOSITOR
// ============================================

/// Comandos enviados do WM para o Compositor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WmToCompositorCommand {
    /// Adicionar uma nova janela ao compositor
    AddWindow {
        window_id: WindowId,
        initial_geometry: Rect,
        title: String,
    },
    
    /// Remover janela do compositor
    RemoveWindow {
        window_id: WindowId,
    },
    
    /// Atualizar geometria da janela
    UpdateGeometry {
        window_id: WindowId,
        new_geometry: Rect,
    },
    
    /// Atualizar estado de foco
    SetFocus {
        window_id: Option<WindowId>,
    },
    
    /// Atualizar estado de maximização
    SetMaximized {
        window_id: WindowId,
        state: MaximizeState,
    },
    
    /// Iniciar drag/move
    StartMove {
        window_id: WindowId,
        grab_x: i32,
        grab_y: i32,
    },
    
    /// Atualizar posição durante drag
    UpdateMove {
        window_id: WindowId,
        delta_x: i32,
        delta_y: i32,
    },
    
    /// Finalizar drag/move
    EndMove {
        window_id: WindowId,
    },
    
    /// Iniciar resize
    StartResize {
        window_id: WindowId,
        edge: ResizeEdge,
        grab_x: i32,
        grab_y: i32,
    },
    
    /// Atualizar tamanho durante resize
    UpdateResize {
        window_id: WindowId,
        delta_x: i32,
        delta_y: i32,
        edge: ResizeEdge,
    },
    
    /// Finalizar resize
    EndResize {
        window_id: WindowId,
    },
    
    /// Elevar janela (z-order)
    RaiseWindow {
        window_id: WindowId,
    },
    
    /// Minimizar janela
    MinimizeWindow {
        window_id: WindowId,
    },
    
    /// Fechar janela
    CloseWindow {
        window_id: WindowId,
    },
}

// ============================================
// PANEL -> WM
// ============================================

/// Comandos enviados do Panel para o WM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelToWmCommand {
    /// Listar todas as janelas gerenciadas
    ListWindows,
    
    /// Obter janela ativa
    GetActiveWindow,
    
    /// Ativar janela (foco + raise)
    ActivateWindow {
        window_id: WindowId,
    },
    
    /// Minimizar janela
    MinimizeWindow {
        window_id: WindowId,
    },
    
    /// Maximizar/restaurar janela
    ToggleMaximize {
        window_id: WindowId,
    },
    
    /// Fechar janela
    CloseWindow {
        window_id: WindowId,
    },
    
    /// Mostrar preview de janela (hover no taskbar)
    ShowPreview {
        window_id: WindowId,
        thumbnail_rect: Rect,
    },
    
    /// Esconder preview
    HidePreview,
}

// ============================================
// RESPONSES
// ============================================

/// Respostas para comandos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub message_id: MessageId,
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> Response<T> {
    pub fn ok(message_id: MessageId, data: T) -> Self {
        Self {
            message_id,
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn err(message_id: MessageId, error: String) -> Self {
        Self {
            message_id,
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Dados de resposta para ListWindows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub window_id: WindowId,
    pub title: String,
    pub geometry: Rect,
    pub is_active: bool,
    pub is_minimized: bool,
    pub app_id: Option<String>,
}

/// Tipo de resposta genérico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResponse {
    ListWindows { windows: Vec<WindowInfo> },
    GetActiveWindow { window_id: Option<WindowId> },
    Ack,
}

// ============================================
// MESSAGE ENVELOPE
// ============================================

/// Envelope para qualquer mensagem IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: MessageId,
    pub timestamp: u64,
    pub source: ComponentId,
    pub target: ComponentId,
    pub payload: IpcPayload,
}

/// Identificador de componente
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentId {
    WindowManager,
    Compositor,
    Panel,
    Launcher,
    Notifications,
    Settings,
}

/// Payload de mensagem IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcPayload {
    // WM <-> Compositor
    WmToCompositor(WmToCompositorCommand),
    
    // Panel <-> WM
    PanelToWm(PanelToWmCommand),
    
    // Responses
    Response(CommandResponse),
}

impl IpcMessage {
    pub fn new(source: ComponentId, target: ComponentId, payload: IpcPayload) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        Self {
            id,
            timestamp,
            source,
            target,
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ipc_message_creation() {
        let msg = IpcMessage::new(
            ComponentId::WindowManager,
            ComponentId::Compositor,
            IpcPayload::WmToCompositor(WmToCompositorCommand::AddWindow {
                window_id: WindowId::new(123),
                initial_geometry: Rect::new(0, 0, 800, 600),
                title: "Test Window".to_string(),
            }),
        );
        
        assert!(msg.id > 0);
        assert_eq!(msg.source, ComponentId::WindowManager);
        assert_eq!(msg.target, ComponentId::Compositor);
    }
    
    #[test]
    fn test_response_ok() {
        let resp: Response<CommandResponse> = Response::ok(
            1,
            CommandResponse::Ack,
        );
        
        assert!(resp.success);
        assert!(resp.data.is_some());
        assert!(resp.error.is_none());
    }
    
    #[test]
    fn test_response_err() {
        let resp: Response<()> = Response::err(1, "Error occurred".to_string());
        
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("Error occurred".to_string()));
    }
}
