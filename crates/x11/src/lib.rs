pub mod connection;
pub mod error;

pub use connection::X11Connection;
pub use error::X11Error;

// Re-export do xcb para que outros crates possam usar os tipos sem adicionar a dependência diretamente.
pub use xcb;
