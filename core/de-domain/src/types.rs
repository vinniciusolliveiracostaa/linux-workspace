//! Types comuns do domínio

use crate::geometry::Rect;
use crate::window::WindowId;

/// Edge da tela para snapping/resizing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenEdge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Direção de resize
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Tipo de superfície (para o compositor)
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceType {
    Window(WindowId),
    Panel,
    Dock,
    Notification,
    Desktop,
}

/// Damage region para repaint eficiente
#[derive(Debug, Clone)]
pub struct DamageRegion {
    pub rects: Vec<Rect>,
}

impl DamageRegion {
    pub fn new() -> Self {
        Self { rects: Vec::new() }
    }
    
    pub fn from_rect(rect: Rect) -> Self {
        Self { rects: vec![rect] }
    }
    
    pub fn add(&mut self, rect: Rect) {
        self.rects.push(rect);
    }
    
    pub fn merge(&mut self, other: DamageRegion) {
        self.rects.extend(other.rects);
    }
    
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.rects.clear();
    }
    
    /// Otimiza a região fundindo rects sobrepostos
    pub fn optimize(&mut self) {
        // TODO: Implementar algoritmo de merge de rects sobrepostos
        // Por enquanto, apenas remove duplicatas exatas
        self.rects.sort_by(|a, b| {
            a.x().cmp(&b.x()).then(a.y().cmp(&b.y()))
        });
        self.rects.dedup();
    }
}

impl Default for DamageRegion {
    fn default() -> Self {
        Self::new()
    }
}

/// Configurações globais do sistema
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub primary_display: Rect,
    pub displays: Vec<Rect>,
    pub font_family: String,
    pub font_size: f32,
    pub scale_factor: f32,
    pub dark_mode: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            primary_display: Rect::new(0, 0, 1920, 1080),
            displays: vec![Rect::new(0, 0, 1920, 1080)],
            font_family: "Inter".to_string(),
            font_size: 13.0,
            scale_factor: 1.0,
            dark_mode: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_damage_region() {
        let mut region = DamageRegion::new();
        assert!(region.is_empty());
        
        region.add(Rect::new(0, 0, 100, 100));
        assert!(!region.is_empty());
        
        region.clear();
        assert!(region.is_empty());
    }
    
    #[test]
    fn test_damage_region_from_rect() {
        let region = DamageRegion::from_rect(Rect::new(10, 10, 50, 50));
        assert_eq!(region.rects.len(), 1);
    }
}
