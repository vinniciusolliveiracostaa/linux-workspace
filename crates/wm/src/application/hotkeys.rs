//! Hotkey Manager — Application Layer
//!
//! Gerencia atalhos de teclado globais do Window Manager
use std::collections::HashMap;
use tracing::{debug, info};
use xcb::x::KeyButMask;

/// Ação que um hotkey pode executar (Command Pattern)
///
/// # POR QUE ENUM:
/// - Type-safe: não podemos executar ação inválida
/// - Extensível: adicionar nova ação = adicionar variante
/// - Pattern matching: compilador garante que tratamos todos os casos
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    /// Fecha a janela focada
    CloseWindow,
    /// Maximiza/restaura a janela focada
    ToggleMaximize,
    /// Sai do Window Manager
    Quit,
}

/// Representa uma combinação de tecla + modificadores
///
/// # POR QUE STRUCT:
/// - Encapsula keysym + modifiers como uma unidade
/// - Implementa Hash/Eq para usar como chave no HashMap
/// - Type-safe: não confundimos keysym com modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey {
    /// Keysym da tecla (ex: XK_q, XK_f)
    pub keysym: u32,
    /// Modificadores (Super, Shift, Ctrl, Alt)
    pub modifiers: KeyButMask,
}

impl Hotkey {
    /// Cria um novo Hotkey
    pub fn new(keysym: u32, modifiers: KeyButMask) -> Self {
        Self { keysym, modifiers }
    }
}

/// Gerenciador de hotkeys globais (Registry Pattern)
///
/// # Responsabilidades
/// - Registrar hotkeys → ações
/// - Lookup de ação baseado em tecla pressionada
/// - Fornecer bindings padrão
///
/// # NÃO faz
/// - Executar ações (responsabilidade do EventDispatcher)
/// - Interagir com X11 diretamente (responsabilidade do X11Adapter)
pub struct HotkeyManager {
    /// Mapa de hotkey → ação
    /// POR QUE HashMap:
    /// - Lookup O(1) — crítico para event loop
    /// - Permite adicionar/remover hotkeys dinamicamente
    bindings: HashMap<Hotkey, HotkeyAction>,
}

impl HotkeyManager {
    /// Cria um novo HotkeyManager com bindings padrão
    ///
    /// # Bindings Padrão (inspirado no macOS)
    /// - Super+Q: fechar janela focada
    /// - Super+F: maximizar/restaurar janela focada
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
        };

        // Registrar bindings padrão
        manager.register_defaults();
        manager
    }

    /// Registra os hotkeys padrão
    fn register_defaults(&mut self) {
        // Super+Q: fechar janela (igual macOS Cmd+Q)
        self.register(
            Hotkey::new(0x71, KeyButMask::MOD4), // 'q' + Super
            HotkeyAction::CloseWindow,
        );

        // Super+F: maximizar janela
        self.register(
            Hotkey::new(0x66, KeyButMask::MOD4), // 'f' + Super
            HotkeyAction::ToggleMaximize,
        );

        info!("Registered default hotkeys");
    }

    /// Registra um novo hotkey
    pub fn register(&mut self, hotkey: Hotkey, action: HotkeyAction) {
        debug!(hotkey = ?hotkey, action = ?action, "Registering hotkey");
        self.bindings.insert(hotkey, action);
    }

    /// Busca ação associada a um hotkey
    pub fn lookup(&self, keysym: u32, modifiers: KeyButMask) -> Option<HotkeyAction> {
        let hotkey = Hotkey::new(keysym, modifiers);
        self.bindings.get(&hotkey).copied()
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}
