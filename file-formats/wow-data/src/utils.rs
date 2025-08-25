use crate::types::MagicStr;

pub fn chunk_magic_to_type(magic: &MagicStr) -> String {
    String::from_utf8_lossy(magic).into()
}
