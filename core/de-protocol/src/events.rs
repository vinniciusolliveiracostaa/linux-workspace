//! Events IPC - Notificações assíncronas broadcast

use de_domain::{geometry::Rect, window::WindowId};
use serde::{Deserialize, Serialize};

// ============================================
// SYSTEM EVENTS
// ============================================

/// Eventos de sistema broadcast para todos os componentes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    /// Sistema está iniciando
    Startup,
    
    /// Sistema está pronto (todos os componentes carregados)
    Ready,
    
    /// Sistema está desligando
    Shutdown,
    
    /// Componente específico está sendo reiniciado
    ComponentRestart { component: String },
}

// ============================================
// DISPLAY EVENTS
// ============================================

/// Eventos relacionados a displays/resolução
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayEvent {
    /// Resolução mudou
    ResolutionChanged {
        display_id: u32,
        new_resolution: Rect,
    },
    
    /// Display conectado/desconectado
    DisplayAdded {
        display_id: u32,
        geometry: Rect,
    },
    
    DisplayRemoved {
        display_id: u32,
    },
    
    /// Scale factor mudou (HiDPI)
    ScaleFactorChanged {
        display_id: u32,
        new_scale: f32,
    },
}

// ============================================
// THEME EVENTS
// ============================================

/// Eventos de tema/aparência
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeEvent {
    /// Modo escuro/claro mudou
    DarkModeChanged { enabled: bool },
    
    /// Tema mudou
    ThemeChanged { theme_name: String },
    
    /// Font mudou
    FontChanged { font_family: String, font_size: f32 },
    
    /// Accent color mudou
    AccentColorChanged { color: [u8; 4] }, // RGBA
}

// ============================================
// CONFIG EVENTS
// ============================================

/// Eventos de configuração
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigEvent {
    /// Configuração foi recarregada
    Reloaded,
    
    /// Uma chave específica mudou
    KeyChanged {
        key: String,
        value: String,
    },
}

// ============================================
// INPUT EVENTS (para componentes UI)
// ============================================

/// Eventos de input globais (apenas para componentes autorizados)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlobalInputEvent {
    /// Hotkey foi acionada
    HotkeyTriggered {
        hotkey: String,
        timestamp: u64,
    },
    
    /// Mouse move global (apenas durante grab)
    MouseMoved {
        x: i32,
        y: i32,
        delta_x: i32,
        delta_y: i32,
    },
    
    /// Mouse button global
    MouseButton {
        button: u8, // 1=left, 2=right, 3=middle
        pressed: bool,
        x: i32,
        y: i32,
    },
}

// ============================================
// NOTIFICATION EVENTS
// ============================================

/// Eventos do sistema de notificações
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
    /// Nova notificação
    NotificationPosted {
        id: u64,
        title: String,
        body: String,
        app_name: Option<String>,
        icon: Option<String>,
        urgency: Urgency,
    },
    
    /// Notificação foi fechada
    NotificationClosed {
        id: u64,
        reason: CloseReason,
    },
    
    /// Ação de notificação foi ativada
    NotificationActionInvoked {
        id: u64,
        action_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloseReason {
    Expired,
    DismissedByUser,
    ClosedByApp,
    Unknown,
}

// ============================================
// EVENT ENVELOPE
// ============================================

/// Envelope para eventos broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastEvent {
    pub event: EventPayload,
    pub timestamp: u64,
    pub source: String,
}

/// Payload de evento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    System(SystemEvent),
    Display(DisplayEvent),
    Theme(ThemeEvent),
    Config(ConfigEvent),
    GlobalInput(GlobalInputEvent),
    Notification(NotificationEvent),
}

impl BroadcastEvent {
    pub fn new<S: Into<String>>(source: S, event: EventPayload) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            event,
            timestamp,
            source: source.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_broadcast_event_creation() {
        let event = BroadcastEvent::new(
            "theme-service",
            EventPayload::Theme(ThemeEvent::DarkModeChanged { enabled: true }),
        );
        
        assert_eq!(event.source, "theme-service");
        assert!(event.timestamp > 0);
        
        if let EventPayload::Theme(ThemeEvent::DarkModeChanged { enabled }) = event.event {
            assert!(enabled);
        } else {
            panic!("Wrong event type");
        }
    }
    
    #[test]
    fn test_notification_event() {
        let notif = NotificationEvent::NotificationPosted {
            id: 1,
            title: "Test".to_string(),
            body: "Body".to_string(),
            app_name: Some("TestApp".to_string()),
            icon: None,
            urgency: Urgency::Normal,
        };
        
        let event = BroadcastEvent::new(
            "notifications",
            EventPayload::Notification(notif),
        );
        
        if let EventPayload::Notification(NotificationEvent::NotificationPosted { title, .. }) = event.event {
            assert_eq!(title, "Test");
        } else {
            panic!("Wrong event type");
        }
    }
}
