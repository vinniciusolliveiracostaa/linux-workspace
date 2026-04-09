#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    // Constantes para as cores do macOS Sonoma (luz)
    pub const WINDOW_BACKGROUND: Self = Self::rgb(236, 236, 236); // #ECECEC
    pub const ACCENT_BLUE: Self = Self::rgb(0, 122, 255); // #007AFF
}
