use macrde_x11::xcb;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub modifiers: u16,
    pub keycode: u8,
}

#[derive(Debug, Clone)]
pub enum WmAction {
    CloseWindow,
    FocusNext,
    // ... outros
}

pub struct hotkeyManager {
    bindings: HashMap<KeyBinding, WmAction>,
}

impl hotkeyManager {
    pub fn new() -> Self {
        let mut bindings = HashMap::new();

        // Super + Q = fechar janela (keycode 24 = 0, Mod4 = Super)
        bindings.insert(
            KeyBinding {
                modifiers: xcb::x::KeyButMask::MOD4.bits() as u16,
                keycode: 24,
            },
            WmAction::CloseWindow,
        );
        Self { bindings }
    }

    pub fn looklop(&self, modifiers: u16, keycode: u8) -> Option<WmAction> {
        // Limpa bits de botoes do mouse
        const KEYBOARD_MODIFIERS: u16 = 0x0f; // Shift, Lock, Control, Mod1..Mod5
        let clean_mods = modifiers & KEYBOARD_MODIFIERS;
        self.bindings
            .get(&KeyBinding {
                modifiers: clean_mods,
                keycode,
            })
            .cloned()
    }
}
