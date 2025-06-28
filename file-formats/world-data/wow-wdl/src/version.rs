//! Defines version information for WDL files

use std::fmt;

/// Represents the different versions of WDL files
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum WdlVersion {
    /// Vanilla version (Classic, TBC, pre-WotLK)
    Vanilla,

    /// Wrath of the Lich King version (3.3.5.12340)
    Wotlk,

    /// Cataclysm version
    Cataclysm,

    /// Mists of Pandaria version
    Mop,

    /// Warlords of Draenor version
    Wod,

    /// Legion version
    Legion,

    /// Battle for Azeroth version
    Bfa,

    /// Shadowlands version
    Shadowlands,

    /// Dragonflight version
    Dragonflight,

    /// Latest (most recent) version
    #[default]
    Latest,
}

impl WdlVersion {
    /// Returns the magic value for this WDL version if applicable
    pub fn magic_value(&self) -> Option<&'static str> {
        // WDL files use the generic chunk-based file format with MVER chunk
        // The actual magic is in the MVER chunk as a version number
        None
    }

    /// Returns the version number for this WDL version
    pub fn version_number(&self) -> u32 {
        match self {
            WdlVersion::Vanilla => 18,
            WdlVersion::Wotlk => 18,
            WdlVersion::Cataclysm => 18,
            WdlVersion::Mop => 18,
            WdlVersion::Wod => 18,
            WdlVersion::Legion => 18,
            WdlVersion::Bfa => 18,
            WdlVersion::Shadowlands => 18,
            WdlVersion::Dragonflight => 18,
            WdlVersion::Latest => 18,
        }
    }

    /// Detects the WDL version from the given version number
    pub fn from_version_number(version: u32) -> Result<Self, String> {
        match version {
            18 => Ok(WdlVersion::Latest), // All versions use 18 currently
            _ => Err(format!("Unknown WDL version number: {version}")),
        }
    }

    /// Returns true if this version supports MWMO, MWID, and MODF chunks
    pub fn has_wmo_chunks(&self) -> bool {
        match self {
            WdlVersion::Vanilla => false,
            WdlVersion::Wotlk => true,
            WdlVersion::Cataclysm => true,
            WdlVersion::Mop => true,
            WdlVersion::Wod => true,
            WdlVersion::Legion => false, // Legion removed these chunks
            WdlVersion::Bfa => false,
            WdlVersion::Shadowlands => false,
            WdlVersion::Dragonflight => false,
            WdlVersion::Latest => false,
        }
    }

    /// Returns true if this version supports MLDD, MLDX, MLMD, MLMX chunks
    pub fn has_ml_chunks(&self) -> bool {
        match self {
            WdlVersion::Vanilla => false,
            WdlVersion::Wotlk => false,
            WdlVersion::Cataclysm => false,
            WdlVersion::Mop => false,
            WdlVersion::Wod => false,
            WdlVersion::Legion => true, // Legion added these chunks
            WdlVersion::Bfa => true,
            WdlVersion::Shadowlands => true,
            WdlVersion::Dragonflight => true,
            WdlVersion::Latest => true,
        }
    }

    /// Returns true if the MAHO chunk is supported
    pub fn has_maho_chunk(&self) -> bool {
        match self {
            WdlVersion::Vanilla => false,
            WdlVersion::Wotlk => true, // WotLK added this chunk
            WdlVersion::Cataclysm => true,
            WdlVersion::Mop => true,
            WdlVersion::Wod => true,
            WdlVersion::Legion => true,
            WdlVersion::Bfa => true,
            WdlVersion::Shadowlands => true,
            WdlVersion::Dragonflight => true,
            WdlVersion::Latest => true,
        }
    }

    /// Converts from one version to another
    pub fn convert_to(&self, target: WdlVersion) -> WdlVersion {
        // Currently all versions are compatible, we just need to add/remove the appropriate chunks
        target
    }
}

impl fmt::Display for WdlVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WdlVersion::Vanilla => write!(f, "Vanilla"),
            WdlVersion::Wotlk => write!(f, "Wrath of the Lich King"),
            WdlVersion::Cataclysm => write!(f, "Cataclysm"),
            WdlVersion::Mop => write!(f, "Mists of Pandaria"),
            WdlVersion::Wod => write!(f, "Warlords of Draenor"),
            WdlVersion::Legion => write!(f, "Legion"),
            WdlVersion::Bfa => write!(f, "Battle for Azeroth"),
            WdlVersion::Shadowlands => write!(f, "Shadowlands"),
            WdlVersion::Dragonflight => write!(f, "Dragonflight"),
            WdlVersion::Latest => write!(f, "Latest"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_number() {
        assert_eq!(WdlVersion::Vanilla.version_number(), 18);
        assert_eq!(WdlVersion::Wotlk.version_number(), 18);
        assert_eq!(WdlVersion::Latest.version_number(), 18);
    }

    #[test]
    fn test_from_version_number() {
        assert!(WdlVersion::from_version_number(18).is_ok());
        assert!(WdlVersion::from_version_number(0).is_err());
    }

    #[test]
    fn test_wmo_chunks_support() {
        assert!(!WdlVersion::Vanilla.has_wmo_chunks());
        assert!(WdlVersion::Wotlk.has_wmo_chunks());
        assert!(!WdlVersion::Legion.has_wmo_chunks());
    }

    #[test]
    fn test_ml_chunks_support() {
        assert!(!WdlVersion::Vanilla.has_ml_chunks());
        assert!(!WdlVersion::Wotlk.has_ml_chunks());
        assert!(WdlVersion::Legion.has_ml_chunks());
    }
}
