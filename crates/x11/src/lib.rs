pub mod connection;
pub mod error;
pub mod events;

pub use connection::X11Connection;
pub use error::X11Error;
pub use events::X11Event;

// Re-export do xcb para que outros crates possam usar os tipos sem adicionar a dependência diretamente.
pub use xcb;
