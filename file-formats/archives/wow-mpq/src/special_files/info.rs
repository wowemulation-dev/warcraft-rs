//! Information about special MPQ files

/// Information about a special file
#[derive(Debug, Clone)]
pub struct SpecialFileInfo {
    /// The filename (e.g., "(listfile)")
    pub name: &'static str,
    /// Whether this file is encrypted by default
    pub encrypted: bool,
    /// Whether this file is compressed by default
    pub compressed: bool,
}

/// Get information about known special files
pub fn get_special_file_info(filename: &str) -> Option<SpecialFileInfo> {
    match filename {
        "(listfile)" => Some(SpecialFileInfo {
            name: "(listfile)",
            encrypted: false,
            compressed: true,
        }),
        "(attributes)" => Some(SpecialFileInfo {
            name: "(attributes)",
            encrypted: false,
            compressed: true,
        }),
        "(signature)" => Some(SpecialFileInfo {
            name: "(signature)",
            encrypted: false,
            compressed: false,
        }),
        "(user data)" => Some(SpecialFileInfo {
            name: "(user data)",
            encrypted: false,
            compressed: false,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_file_info() {
        assert!(get_special_file_info("(listfile)").is_some());
        assert!(get_special_file_info("(attributes)").is_some());
        assert!(get_special_file_info("(signature)").is_some());
        assert!(get_special_file_info("(user data)").is_some());
        assert!(get_special_file_info("regular_file.txt").is_none());

        let info = get_special_file_info("(attributes)").unwrap();
        assert!(!info.encrypted);
        assert!(info.compressed);
    }
}
