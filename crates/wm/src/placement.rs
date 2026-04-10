use crate::service::WindowManagerService;
use macrde_core::{Position, Rectangle, Size};

#[derive(Debug, Clone, Copy)]
pub enum PlacementStrategy {
    Center,
    Cascade,
    // Smart será implementado depois
}

impl Default for PlacementStrategy {
    fn default() -> Self {
        Self::Center
    }
}

pub fn compute_placement(
    _service: &WindowManagerService,
    strategy: PlacementStrategy,
    screen_geom: Rectangle,
    window_size: Size,
) -> Position {
    match strategy {
        PlacementStrategy::Center => {
            let x = screen_geom.position.x
                + (screen_geom.size.width() as i32 - window_size.width() as i32) / 2;
            let y = screen_geom.position.y
                + (screen_geom.size.height() as i32 - window_size.height() as i32) / 2;
            Position::new(x.max(0), y.max(0))
        }
        PlacementStrategy::Cascade => {
            // Simplicada: offset fixo. Futuramente usaremos o numero de janelas.
            let offset = 30;
            Position::new(offset, offset)
        }
    }
}
