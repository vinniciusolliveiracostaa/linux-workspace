// de-render — Software Renderer Infrastructure

pub mod buffer;
pub mod error;
pub mod renderer;

pub use buffer::FrameBuffer;
pub use error::RenderError;
pub use renderer::SoftwareRenderer;
