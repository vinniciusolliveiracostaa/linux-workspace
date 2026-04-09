use serde::{Deserialize, Serialize};

/// Representa uma posição 2D.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Representa um tamanho 2D com invariante estrita: largura e altura > 0.
/// Os campos são privados para garantir que nenhum Size inválido seja criado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    /// Construtor validado. Retorna `None` se a invariante for violada
    pub const fn new(width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        Some(Self { width, height })
    }

    /// Construtor "inseguro" para uso interno quando a invariante 'e garantida.
    /// # Safety
    /// O chamador deve garantir que width > 0 e height > 0.
    pub const unsafe fn new_unchecked(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }
}

/// Representa um retângulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rectangle {
    pub position: Position,
    pub size: Size,
}

impl Rectangle {
    pub const fn new(position: Position, size: Size) -> Self {
        Self { position, size }
    }

    /// M'etodo helper para criar a partir de coordenadas.
    pub fn from_xywh(x: i32, y: i32, width: u32, height: u32) -> Option<Self> {
        Some(Self {
            position: Position::new(x, y),
            size: Size::new(width, height)?,
        })
    }

    pub fn contains(&self, point: Position) -> bool {
        point.x >= self.position.x
            && point.x < self.position.x + self.size.width() as i32
            && point.y >= self.position.y
            && point.y < self.position.y + self.size.height() as i32
    }
}
