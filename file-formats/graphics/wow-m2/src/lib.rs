// Re-export main components
pub mod anim;
pub mod chunks;
pub mod error;
pub mod game_version;
pub mod header;
pub mod md20;
pub mod model;
pub mod phys;
pub mod skin;
pub mod version;

// Re-export common types
// pub use anim::AnimFile;
pub use error::{M2Error, Result};
pub use md20::MD20Model;
pub use model::M2Model;
pub use phys::PhysFile;
pub use skin::Skin;
pub use version::MD20Version;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
