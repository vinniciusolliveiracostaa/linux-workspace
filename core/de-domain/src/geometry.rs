//! Geometry types - Value objects para posições e dimensões
//! 
//! Usa euclid para tipos fortemente tipados com unidades.

use euclid::{default, Point2D, Rect as EuclidRect, Size2D, Vector2D};

/// Posição 2D em pixels (coordenadas de tela)
pub type Position = Point2D<i32>;

/// Tamanho 2D em pixels
pub type Size = Size2D<i32>;

/// Vetor 2D em pixels (para deltas, offsets)
pub type Vector = Vector2D<i32>;

/// Retângulo em coordenadas de tela
/// 
/// Invariantes:
/// - width e height devem ser >= 0
/// - Criado apenas através de construtores validados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rect {
    pub origin: Position,
    pub size: Size,
}

impl Rect {
    /// Cria um novo Rect validando invariantes
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        // Invariante: dimensões não negativas
        debug_assert!(width >= 0, "Width must be non-negative");
        debug_assert!(height >= 0, "Height must be non-negative");
        
        Self {
            origin: Position::new(x, y),
            size: Size::new(width, height),
        }
    }
    
    /// Cria Rect a partir de Position e Size
    pub fn from_origin_size(origin: Position, size: Size) -> Self {
        Self::new(origin.x, origin.y, size.width, size.height)
    }
    
    /// Retorna o lado esquerdo (x)
    #[inline]
    pub fn x(&self) -> i32 {
        self.origin.x
    }
    
    /// Retorna o topo (y)
    #[inline]
    pub fn y(&self) -> i32 {
        self.origin.y
    }
    
    /// Retorna a largura
    #[inline]
    pub fn width(&self) -> i32 {
        self.size.width
    }
    
    /// Retorna a altura
    #[inline]
    pub fn height(&self) -> i32 {
        self.size.height
    }
    
    /// Retorna o lado direito (x + width)
    #[inline]
    pub fn right(&self) -> i32 {
        self.x() + self.width()
    }
    
    /// Retorna o lado inferior (y + height)
    #[inline]
    pub fn bottom(&self) -> i32 {
        self.y() + self.height()
    }
    
    /// Verifica se um ponto está dentro do rect
    #[inline]
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x() && x < self.right() && y >= self.y() && y < self.bottom()
    }
    
    /// Verifica se dois rects se interceptam
    #[inline]
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x() < other.right()
            && self.right() > other.x()
            && self.y() < other.bottom()
            && self.bottom() > other.y()
    }
    
    /// Retorna a interseção entre dois rects (se existir)
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let left = self.x().max(other.x());
        let top = self.y().max(other.y());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        
        if left < right && top < bottom {
            Some(Rect::new(left, top, right - left, bottom - top))
        } else {
            None
        }
    }
    
    /// União de dois rects (menor rect que contém ambos)
    pub fn union(&self, other: &Rect) -> Rect {
        let left = self.x().min(other.x());
        let top = self.y().min(other.y());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        
        Rect::new(left, top, right - left, bottom - top)
    }
    
    /// Move o rect por um vetor
    pub fn translate(&self, vector: Vector) -> Rect {
        Rect::new(
            self.x() + vector.x,
            self.y() + vector.y,
            self.width(),
            self.height(),
        )
    }
    
    /// Infla o rect por uma quantidade em cada lado
    pub fn inflate(&self, dx: i32, dy: i32) -> Rect {
        Rect::new(
            self.x() - dx,
            self.y() - dy,
            self.width() + dx * 2,
            self.height() + dy * 2,
        )
    }
    
    /// Converte para euclid::Rect
    pub fn to_euclid(&self) -> EuclidRect<i32> {
        EuclidRect::new(self.origin, self.size)
    }
    
    /// Cria a partir de euclid::Rect
    pub fn from_euclid(rect: EuclidRect<i32>) -> Self {
        Self::from_origin_size(rect.origin, rect.size)
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10, 10, 100, 100);
        assert!(rect.contains(50, 50));
        assert!(!rect.contains(5, 5));
        assert!(!rect.contains(115, 115));
    }
    
    #[test]
    fn test_rect_intersects() {
        let a = Rect::new(0, 0, 100, 100);
        let b = Rect::new(50, 50, 100, 100);
        let c = Rect::new(200, 200, 50, 50);
        
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }
    
    #[test]
    fn test_rect_translate() {
        let rect = Rect::new(0, 0, 100, 100);
        let moved = rect.translate(Vector::new(10, 20));
        assert_eq!(moved.x(), 10);
        assert_eq!(moved.y(), 20);
        assert_eq!(moved.width(), 100);
        assert_eq!(moved.height(), 100);
    }
}
