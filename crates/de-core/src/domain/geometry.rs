use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Cria uma nova posição
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Value Object — sempre válido (width > 0, height > 0)
///
/// # POR QUE campos privados?
/// Se os campos fossem pub, alguém poderia fazer Size { width: 0, height: 0 }
/// e burlar nossa invariante. Campos privados + construtor validado = invariante garantida.
///
/// # Invariante
/// - width > 0
/// - height > 0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Size {
    width: u32,  // privado — invariante: width > 0
    height: u32, // privado — invariante: height > 0
}

impl Size {
    /// Construtor validado — retorna None se width ou height == 0
    ///
    /// # POR QUE Option?
    /// - Rust idiomático: Option representa "pode não existir"
    /// - Força o caller a tratar o caso de tamanho inválido
    /// - Evita panic em runtime
    ///
    /// # Exemplo
    /// ```
    /// let size = Size::new(800, 600).expect("tamanho válido");
    /// let invalid = Size::new(0, 600); // None
    /// ```
    pub const fn new(width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            None
        } else {
            Some(Self { width, height })
        }
    }

    /// Construtor para valores conhecidos em tempo de compilação
    ///
    /// # Safety
    /// Só use quando width e height são literalmente > 0.
    /// Ex: Size::new_unchecked(1920, 1080)
    ///
    /// # POR QUE unchecked?
    /// - Para constantes conhecidas (WINDOW_MIN_WIDTH, SCREEN_SIZE)
    /// - Evita Option desnecessário quando sabemos que é válido
    /// - debug_assert garante em dev, zero-cost em release
    ///
    /// # Exemplo
    /// ```
    /// const SCREEN_SIZE: Size = Size::new_unchecked(1920, 1080);
    /// ```
    pub const fn new_unchecked(width: u32, height: u32) -> Self {
        // debug_assert em dev, zero-cost em release
        debug_assert!(width > 0, "width must be > 0");
        debug_assert!(height > 0, "height must be > 0");
        Self { width, height }
    }

    /// Getter para width
    ///
    /// # POR QUE getter?
    /// - Campos privados = precisamos de getters para leitura
    /// - Rust idiomático: getters sem prefixo "get_"
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Getter para height
    pub const fn height(&self) -> u32 {
        self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rectangle {
    pub position: Position,
    pub size: Size,
}

impl Rectangle {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Option<Self> {
        Some(Self {
            position: Position { x, y },
            size: Size::new(width, height)?,
        })
    }

    pub fn new_unchecked(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            position: Position { x, y },
            size: Size::new_unchecked(width, height),
        }
    }

    pub fn contains_point(&self, point: Position) -> bool {
        let right = self.position.x + self.size.width() as i32;
        let bottom = self.position.y + self.size.height() as i32;

        point.x >= self.position.x
            && point.x < right
            && point.y >= self.position.y
            && point.y < bottom
    }

    pub fn intersects(&self, other: Rectangle) -> bool {
        let self_right = self.position.x + self.size.width() as i32;
        let self_bottom = self.position.y + self.size.height() as i32;
        let other_right = other.position.x + other.size.width() as i32;
        let other_bottom = other.position.y + other.size.height() as i32;

        self.position.x < other_right
            && self_right > other.position.x
            && self.position.y < other_bottom
            && self_bottom > other.position.y
    }

    pub fn intersection(&self, other: Rectangle) -> Option<Rectangle> {
        if !self.intersects(other) {
            return None;
        }

        let self_right = self.position.x + self.size.width() as i32;
        let self_bottom = self.position.y + self.size.height() as i32;
        let other_right = other.position.x + other.size.width() as i32;
        let other_bottom = other.position.y + other.size.height() as i32;

        let x = self.position.x.max(other.position.x);
        let y = self.position.y.max(other.position.y);
        let right = self_right.min(other_right);
        let bottom = self_bottom.min(other_bottom);

        let width = (right - x) as u32;
        let height = (bottom - y) as u32;

        let size = Size::new(width, height)?;

        Some(Rectangle {
            position: Position { x, y },
            size,
        })
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Rectangle {
        Rectangle {
            position: Position {
                x: self.position.x + dx,
                y: self.position.y + dy,
            },
            size: self.size,
        }
    }
}
