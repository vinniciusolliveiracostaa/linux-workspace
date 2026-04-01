#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Failed to create pixmap with size {width}x{height}")]
    PixmapCreationFailed { width: u32, height: u32 },

    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),
}
