use de_core::{Rectangle, Window, WindowId, Workspace, WorkspaceId};
use de_x11::{Ewmh, Icccm, X11Connection, X11Error};
use std::collections::HashMap;
use xcb::XidNew; // Para xcb::x::Window::new()

use crate::placement::PlacementStrategy;

pub struct WindowManager {
    // Workspaces (áreas de trabalho vírtuais)
    workspaces: Vec<Workspace>,
    current_workspace: WorkspaceId,

    // Mapeamento: qual janela está em qual Workspace
    window_to_workspace: HashMap<WindowId, WorkspaceId>,

    // Conexão X11
    x11: X11Connection,

    // Config (por enquanto hardcoded, depois vem do config manager)
    workspace_count: u32,
}

impl WindowManager {
    pub fn new(workspace_count: u32) -> Result<Self, X11Error> {
        let x11 = X11Connection::connect()?;

        // Criar workspaces
        let workspaces: Vec<Workspace> = (0..workspace_count)
            .map(|i| Workspace::new(WorkspaceId(i)))
            .collect();

        Ok(Self {
            workspaces,
            current_workspace: WorkspaceId(0),
            window_to_workspace: HashMap::new(),
            x11,
            workspace_count,
        })
    }

    /// Acesso aos helpers EWMH (criados on-demand)
    fn ewmh(&self) -> Result<Ewmh<'_>, X11Error> {
        let root = self.x11.root_window()?;
        Ok(Ewmh::new(self.x11.connection(), &self.x11.atoms, root))
    }

    /// Acesso aos helpers ICCCM (criados on-demand)
    fn icccm(&self) -> Icccm<'_> {
        Icccm::new(self.x11.connection(), &self.x11.atoms)
    }

    /// Obtém geometria de uma janela do X11
    fn get_window_geometry(&self, _window_id: WindowId) -> Result<Rectangle, X11Error> {
        // TODO: Implementar query ao X11
        // Por enquanto, retorna geometria padrão (800x600 centralizado)
        let screen = self.x11.screen_geometry()?;
        let width = 800;
        let height = 600;
        let x = (screen.size.width - width) / 2;
        let y = (screen.size.height - height) / 2;

        Ok(Rectangle::new(x as i32, y as i32, width, height))
    }

    /// Retorna referência mutável ao workspace atual
    fn get_current_workspace_mut(&mut self) -> Result<&mut Workspace, X11Error> {
        self.workspaces
            .iter_mut()
            .find(|ws| ws.id() == self.current_workspace)
            .ok_or_else(|| {
                X11Error::ProtocolError(format!(
                    "Current workspace {:?} not found",
                    self.current_workspace
                ))
            })
    }

    /// Atualiza list de janelas gerenciadas via EWMH
    fn update_client_list(&self) -> Result<(), X11Error> {
        // Coletar IDs de todas as janelas em todos os workspaces
        let window_ids: Vec<xcb::x::Window> = self
            .workspaces
            .iter()
            .flat_map(|ws| ws.windows())
            .map(|w| xcb::x::Window::new(w.id.0))
            .collect();

        // Atualizar propriedades EWMH
        let ewmh = self.ewmh()?;
        ewmh.set_client_list(&window_ids)?;

        Ok(())
    }

    /// Gerencia uma nova janela (adiciona ao workspace atual)
    pub fn manage_window(&mut self, window_id: WindowId) -> Result<(), X11Error> {
        // 1. Obter geometria da janela do X11
        let geometry = self.get_window_geometry(window_id)?;

        // 2. Criar window entity (domain)
        let window = Window::new(window_id, geometry);

        // 3. Adicionar ao workspace atual
        let workspace = self.get_current_workspace_mut()?;
        workspace.add_window(window);

        // 4. Registrar mapeamento janela + workspace
        self.window_to_workspace
            .insert(window_id, self.current_workspace);

        // 5. Atualizar EWMH client list (notificar outros apps)
        self.update_client_list()?;

        // 6. Mapear a janela no X11 (tornar visível)
        let x11_window = xcb::x::Window::new(window_id.0);
        self.x11.map_window(x11_window)?;

        Ok(())
    }

    /// Remove uma janela do gerenciamento
    pub fn unmanage_window(&mut self, window_id: WindowId) -> Result<(), X11Error> {
        // 1. Encontrar workspace da janela
        let workspace_id = self.window_to_workspace.remove(&window_id).ok_or_else(|| {
            X11Error::ProtocolError(format!("Window {:?} not managed", window_id))
        })?;

        // 2. Remover do workspace
        let workspace = self
            .workspaces
            .iter_mut()
            .find(|ws| ws.id() == workspace_id)
            .ok_or_else(|| {
                X11Error::ProtocolError(format!("Workspace {:?} not found", workspace_id))
            })?;

        workspace.remove_window(window_id);

        // 3. Atualizar EWMH client list
        self.update_client_list()?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), X11Error> {
        Ok(())
    }
}
