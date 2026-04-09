use macrde_core::Color;

/// Configuração de estilo para renderização de texto.
#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub color: Color,
    pub size: f32,
    // Adicionaremos weight, family, etc. mais tarde.
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::rgb(0, 0, 0),
            size: 13.0,
        }
    }
}

/// Erros específicos do módulo de renderização.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Surface creation failed")]
    SurfaceCreationFailed,
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),
    #[error("Drawing operation failed")]
    DrawFailed,
}
