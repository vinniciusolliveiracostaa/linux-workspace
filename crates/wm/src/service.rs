use macrde_core::{Rectangle, WindowId, WorkspaceId};
use macrde_ipc::CompositorCommand;
use std::collections::HashMap;

pub struct WindowManagerService {
    windows: HashMap<WindowId, ManagedWindow>,
    workspace_windows: HashMap<WorkspaceId, Vec<WindowId>>,
    current_workspace: WorkspaceId,
    focused_window: Option<WindowId>,
}

struct ManagedWindow {
    id: WindowId,
    geometry: Rectangle,
    workspace: WorkspaceId,
    is_focused: bool,
    // Outros atributos virao depois (title, class, etc.)
}

impl WindowManagerService {
    pub fn new() -> Self {
        let mut workspace_windows = HashMap::new();
        workspace_windows.insert(WorkspaceId(0), Vec::new());
        Self {
            windows: HashMap::new(),
            workspace_windows,
            current_workspace: WorkspaceId(0),
            focused_window: None,
        }
    }

    pub fn add_window(&mut self, id: WindowId, geometry: Rectangle) -> CompositorCommand {
        let window = ManagedWindow {
            id,
            geometry,
            workspace: self.current_workspace,
            is_focused: false,
        };

        self.windows.insert(id, window);
        self.workspace_windows
            .entry(self.current_workspace)
            .or_default()
            .push(id);

        // Decide a geometria final (usando placement strategy depois)
        CompositorCommand::AddWindow {
            window_id: id,
            geometry,
        }
    }

    pub fn remove_window(&mut self, id: WindowId) -> Option<CompositorCommand> {
        if self.windows.remove(&id).is_some() {
            for windows in self.workspace_windows.values_mut() {
                windows.retain(|&wid| wid != id);
            }

            if self.focused_window == Some(id) {
                self.focused_window = None;
                // precisa focar na proxima janela
            }
            Some(CompositorCommand::RemoveWindow { window_id: id })
        } else {
            None
        }
    }

    pub fn focus_window(&mut self, id: WindowId) -> Option<CompositorCommand> {
        if self.windows.contains_key(&id) {
            if let Some(prev) = self.focused_window {
                if let Some(win) = self.windows.get_mut(&prev) {
                    win.is_focused = false;
                }
            }
            self.focused_window = Some(id);
            if let Some(win) = self.windows.get_mut(&id) {
                win.is_focused = true;
            }
            Some(CompositorCommand::FocusWindow { window_id: id })
        } else {
            None
        }
    }

    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }
}
