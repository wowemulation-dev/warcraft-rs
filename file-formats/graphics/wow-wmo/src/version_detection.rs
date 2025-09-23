use crate::chunk_discovery::ChunkDiscovery;

/// WMO version based on expansion and chunk patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmoVersion {
    /// Classic/Vanilla (1.12.1) and TBC (2.4.3)
    Classic,
    /// Wrath of the Lich King (3.3.5)
    WotLK,
    /// Cataclysm (4.3.4) - introduced MCVP chunk
    Cataclysm,
    /// Mists of Pandaria (5.4.8)
    MoP,
    /// Warlords of Draenor (6.2.4) - introduced GFID chunk
    Warlords,
    /// Legion and later
    Legion,
}

/// Detect WMO version based on chunk patterns
///
/// Note: All versions use format version 17, so we detect by chunk presence:
/// - MCVP → Cataclysm or later
/// - GFID → Warlords or later
/// - Otherwise → Classic/TBC/WotLK
pub fn detect_version(discovery: &ChunkDiscovery) -> WmoVersion {
    // Collect chunk IDs for pattern matching
    let chunk_ids: Vec<&str> = discovery.chunks.iter().map(|c| c.id.as_str()).collect();

    // Check for version-specific chunks
    if chunk_ids.contains(&"GFID") {
        // GFID was introduced in Warlords of Draenor
        WmoVersion::Warlords
    } else if chunk_ids.contains(&"MCVP") {
        // MCVP was introduced in Cataclysm for vertex painting
        WmoVersion::Cataclysm
    } else {
        // No version-specific chunks - likely Classic, TBC, or WotLK
        // These can't be distinguished by chunks alone
        WmoVersion::Classic
    }
}

impl WmoVersion {
    /// Check if this version supports a specific feature
    pub fn supports_feature(&self, feature: WmoFeature) -> bool {
        match feature {
            WmoFeature::VertexPainting => {
                matches!(
                    self,
                    WmoVersion::Cataclysm
                        | WmoVersion::MoP
                        | WmoVersion::Warlords
                        | WmoVersion::Legion
                )
            }
            WmoFeature::FileDataId => {
                matches!(self, WmoVersion::Warlords | WmoVersion::Legion)
            }
        }
    }

    /// Get the earliest expansion that supports this version
    pub fn expansion_name(&self) -> &'static str {
        match self {
            WmoVersion::Classic => "Classic/TBC",
            WmoVersion::WotLK => "Wrath of the Lich King",
            WmoVersion::Cataclysm => "Cataclysm",
            WmoVersion::MoP => "Mists of Pandaria",
            WmoVersion::Warlords => "Warlords of Draenor",
            WmoVersion::Legion => "Legion+",
        }
    }
}

/// WMO feature flags for version compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WmoFeature {
    /// Vertex painting support (Cataclysm+)
    VertexPainting,
    /// File data ID support (Warlords+)
    FileDataId,
}
