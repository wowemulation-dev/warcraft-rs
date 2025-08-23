// Re-export main components
pub mod anim;
pub mod chunks;
pub mod error;
pub mod game_version;
pub mod header;
pub mod model;
pub mod skin;
pub mod version;

// Re-export common types
// pub use anim::AnimFile;
// pub use converter::M2Converter;
pub use error::{M2Error, Result};
pub use model::M2Model;
pub use skin::Skin;
pub use version::M2Version;

// Re-export BLP types from wow-blp crate for backwards compatibility
pub use wow_blp::BlpImage as BlpTexture;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
