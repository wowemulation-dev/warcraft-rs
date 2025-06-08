//! Special MPQ files handling: (listfile), (attributes), (signature), etc.

mod attributes;
mod info;
mod listfile;

pub use attributes::{AttributeFlags, Attributes, FileAttributes};
pub use info::{SpecialFileInfo, get_special_file_info};
pub use listfile::parse_listfile;
