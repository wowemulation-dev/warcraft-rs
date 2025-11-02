//! ADT file type detection and classification.
//!
//! Starting with Cataclysm (4.x), Blizzard split ADT files into multiple components
//! for improved loading performance and data organization. This module provides
//! detection logic to identify which type of ADT file is being processed.
//!
//! # Cataclysm+ Split File Architecture
//!
//! Pre-Cataclysm (1.x - 3.x):
//! - Single monolithic ADT file containing all terrain, texture, and object data
//!
//! Cataclysm+ (4.x - 5.x):
//! - **Root ADT**: Core terrain data (heightmaps, normals, holes)
//! - **_tex0.adt**: Primary texture definitions and layer data
//! - **_tex1.adt**: Additional texture data (heightmaps, shadows)
//! - **_obj0.adt**: M2 model and WMO placement data
//! - **_obj1.adt**: Additional object placement data
//! - **_lod.adt**: Level-of-detail rendering data
//!
//! # Detection Strategy
//!
//! File type is determined by analyzing chunk presence patterns:
//!
//! | File Type | Key Chunks | Detection Logic |
//! |-----------|-----------|----------------|
//! | Root | MCNK | Has terrain chunks |
//! | Tex0/Tex1 | MTEX, MCLY | Has textures but no MCNK |
//! | Obj0/Obj1 | MDDF, MODF | Has object refs but no MCNK |
//! | Lod | Minimal chunks | Fallback for LOD data |
//!
//! Filename-based detection provides faster classification when file path is available.

use crate::chunk_discovery::{ChunkDiscovery, ChunkLocation};
use crate::chunk_id::ChunkId;
use std::collections::HashMap;

/// ADT file type in Cataclysm+ split file architecture.
///
/// Starting with World of Warcraft 4.0 (Cataclysm), ADT files were split into
/// multiple specialized files to improve loading performance and enable streaming.
///
/// # File Type Responsibilities
///
/// - **Root**: Core terrain geometry (heightmaps, vertex normals, hole masks)
/// - **Tex0**: Texture filenames, layer definitions, alpha maps
/// - **Tex1**: Height textures, shadow maps, additional texture data
/// - **Obj0**: M2 model placements (MDDF), WMO placements (MODF)
/// - **Obj1**: Additional object placement data
/// - **Lod**: Level-of-detail data for distant terrain rendering
///
/// # Version Compatibility
///
/// - **1.x - 3.x**: All data in single root file (detected as `Root`)
/// - **4.x - 5.x**: Split file architecture with specialized types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdtFileType {
    /// Root ADT file containing core terrain data.
    ///
    /// Contains MCNK chunks with heightmaps, normals, and terrain structure.
    /// Present in all WoW versions.
    Root,

    /// Texture file 0 containing primary texture data.
    ///
    /// Contains MTEX (texture filenames) and MCLY (layer definitions).
    /// Cataclysm+ only.
    Tex0,

    /// Texture file 1 containing additional texture data.
    ///
    /// Contains height textures, shadow maps, and auxiliary texture information.
    /// Cataclysm+ only.
    Tex1,

    /// Object file 0 containing M2 and WMO placement data.
    ///
    /// Contains MDDF (M2 doodad placements) and MODF (WMO placements).
    /// Cataclysm+ only.
    Obj0,

    /// Object file 1 containing additional object data.
    ///
    /// Contains supplementary object placement information.
    /// Cataclysm+ only.
    Obj1,

    /// Level-of-detail file for distant terrain rendering.
    ///
    /// Contains simplified geometry for rendering distant terrain tiles.
    /// Cataclysm+ only.
    Lod,
}

impl AdtFileType {
    /// Detect file type from chunk location map.
    ///
    /// Analyzes which chunks are present in the file to determine its type.
    /// This method works without knowing the filename.
    ///
    /// # Detection Logic
    ///
    /// 1. **Has MCNK chunks** → Root file (terrain data)
    /// 2. **Has MTEX but no MCNK** → Tex0/Tex1 file (texture data)
    /// 3. **Has MDDF/MODF but no MCNK** → Obj0/Obj1 file (object placements)
    /// 4. **Minimal chunks** → Lod file (level-of-detail data)
    ///
    /// # Arguments
    ///
    /// * `chunks` - Map of chunk IDs to their locations
    ///
    /// # Returns
    ///
    /// Detected file type based on chunk presence patterns.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wow_adt::file_type::AdtFileType;
    /// use wow_adt::chunk_id::ChunkId;
    /// use wow_adt::chunk_discovery::ChunkLocation;
    /// use std::collections::HashMap;
    ///
    /// let mut chunks = HashMap::new();
    /// chunks.insert(ChunkId::MCNK, vec![ChunkLocation { offset: 1024, size: 500 }]);
    ///
    /// let file_type = AdtFileType::detect_from_chunks(&chunks);
    /// assert_eq!(file_type, AdtFileType::Root);
    /// ```
    pub fn detect_from_chunks(chunks: &HashMap<ChunkId, Vec<ChunkLocation>>) -> Self {
        let has_mcnk = chunks.contains_key(&ChunkId::MCNK);
        let has_mhdr = chunks.contains_key(&ChunkId::MHDR);
        let has_mtex = chunks.contains_key(&ChunkId::MTEX);
        let has_mddf = chunks.contains_key(&ChunkId::MDDF);
        let has_modf = chunks.contains_key(&ChunkId::MODF);
        let has_mmdx = chunks.contains_key(&ChunkId::MMDX);
        let has_mwmo = chunks.contains_key(&ChunkId::MWMO);

        // Distinguish between Root and split files (Cataclysm+):
        // - Root files: MHDR + MCNK (may or may not have MTEX/MDDF/MODF depending on version)
        // - Tex0 files: MTEX + MCNK (texture-specific sub-chunks) but NO MHDR
        // - Obj0 files: MDDF/MODF/MMDX/MWMO + MCNK (object sub-chunks) but NO MHDR

        if has_mhdr && has_mcnk {
            // Root file - has header and terrain chunks
            Self::Root
        } else if has_mtex && !has_mhdr {
            // Texture file - has textures but no header (split file)
            Self::Tex0
        } else if (has_mddf || has_modf || has_mmdx || has_mwmo) && !has_mhdr {
            // Object file - has placements but no header (split file)
            Self::Obj0
        } else if has_mcnk {
            // Has MCNK but no MHDR - could be legacy or unusual format
            // Default to Root for backward compatibility
            Self::Root
        } else {
            // LOD files or other minimal chunk inventory
            Self::Lod
        }
    }

    /// Detect file type from chunk discovery result.
    ///
    /// Convenience method that extracts chunk locations from `ChunkDiscovery`
    /// and performs file type detection.
    ///
    /// # Arguments
    ///
    /// * `discovery` - Chunk discovery result from Phase 1 parsing
    ///
    /// # Returns
    ///
    /// Detected file type based on discovered chunks.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wow_adt::file_type::AdtFileType;
    /// use wow_adt::chunk_discovery::{ChunkDiscovery, ChunkLocation};
    /// use wow_adt::chunk_id::ChunkId;
    /// use std::collections::HashMap;
    ///
    /// let mut discovery = ChunkDiscovery::new(1000);
    /// let mut chunks = HashMap::new();
    /// chunks.insert(ChunkId::MCNK, vec![ChunkLocation { offset: 100, size: 500 }]);
    /// discovery.chunks = chunks;
    ///
    /// let file_type = AdtFileType::from_discovery(&discovery);
    /// assert_eq!(file_type, AdtFileType::Root);
    /// ```
    pub fn from_discovery(discovery: &ChunkDiscovery) -> Self {
        Self::detect_from_chunks(&discovery.chunks)
    }

    /// Detect file type from filename pattern.
    ///
    /// Faster detection method when filename is available. Recognizes standard
    /// Cataclysm+ naming conventions.
    ///
    /// # Filename Patterns
    ///
    /// - `MapName_XX_YY.adt` → Root
    /// - `MapName_XX_YY_tex0.adt` → Tex0
    /// - `MapName_XX_YY_tex1.adt` → Tex1
    /// - `MapName_XX_YY_obj0.adt` → Obj0
    /// - `MapName_XX_YY_obj1.adt` → Obj1
    /// - `MapName_XX_YY_lod.adt` → Lod
    ///
    /// # Arguments
    ///
    /// * `filename` - ADT filename (case-insensitive)
    ///
    /// # Returns
    ///
    /// File type based on filename pattern. Defaults to `Root` if no pattern matches.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wow_adt::file_type::AdtFileType;
    ///
    /// assert_eq!(
    ///     AdtFileType::from_filename("Azeroth_32_48_tex0.adt"),
    ///     AdtFileType::Tex0
    /// );
    ///
    /// assert_eq!(
    ///     AdtFileType::from_filename("Kalimdor_16_32.adt"),
    ///     AdtFileType::Root
    /// );
    /// ```
    pub fn from_filename(filename: &str) -> Self {
        let lower = filename.to_lowercase();

        if lower.ends_with("_tex0.adt") {
            Self::Tex0
        } else if lower.ends_with("_tex1.adt") {
            Self::Tex1
        } else if lower.ends_with("_obj0.adt") {
            Self::Obj0
        } else if lower.ends_with("_obj1.adt") {
            Self::Obj1
        } else if lower.ends_with("_lod.adt") {
            Self::Lod
        } else {
            // Default to root for standard .adt files
            Self::Root
        }
    }

    /// Get human-readable description of file type.
    ///
    /// # Returns
    ///
    /// Static string describing the file type's purpose.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wow_adt::file_type::AdtFileType;
    ///
    /// assert_eq!(
    ///     AdtFileType::Root.description(),
    ///     "Root ADT (terrain data)"
    /// );
    /// ```
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Root => "Root ADT (terrain data)",
            Self::Tex0 => "Texture file 0 (primary textures)",
            Self::Tex1 => "Texture file 1 (additional textures)",
            Self::Obj0 => "Object file 0 (M2/WMO placements)",
            Self::Obj1 => "Object file 1 (additional objects)",
            Self::Lod => "Level-of-detail file",
        }
    }

    /// Check if file type is part of split file architecture.
    ///
    /// # Returns
    ///
    /// `true` for Cataclysm+ split files (Tex0, Tex1, Obj0, Obj1, Lod).
    /// `false` for root files (present in all versions).
    #[must_use]
    pub const fn is_split_file(&self) -> bool {
        !matches!(self, Self::Root)
    }

    /// Check if file type contains terrain geometry.
    ///
    /// # Returns
    ///
    /// `true` for Root and Lod files containing heightmap data.
    #[must_use]
    pub const fn has_terrain_geometry(&self) -> bool {
        matches!(self, Self::Root | Self::Lod)
    }

    /// Check if file type contains texture data.
    ///
    /// # Returns
    ///
    /// `true` for Tex0 and Tex1 files containing texture definitions.
    #[must_use]
    pub const fn has_texture_data(&self) -> bool {
        matches!(self, Self::Tex0 | Self::Tex1)
    }

    /// Check if file type contains object placement data.
    ///
    /// # Returns
    ///
    /// `true` for Obj0 and Obj1 files containing M2/WMO placements.
    #[must_use]
    pub const fn has_object_data(&self) -> bool {
        matches!(self, Self::Obj0 | Self::Obj1)
    }
}

impl std::fmt::Display for AdtFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_root_file_from_chunks() {
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MCNK,
            vec![ChunkLocation {
                offset: 1024,
                size: 500,
            }],
        );

        assert_eq!(AdtFileType::detect_from_chunks(&chunks), AdtFileType::Root);
    }

    #[test]
    fn detect_texture_file_from_chunks() {
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MTEX,
            vec![ChunkLocation {
                offset: 512,
                size: 200,
            }],
        );

        assert_eq!(AdtFileType::detect_from_chunks(&chunks), AdtFileType::Tex0);
    }

    #[test]
    fn detect_object_file_from_chunks() {
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MDDF,
            vec![ChunkLocation {
                offset: 2048,
                size: 100,
            }],
        );

        assert_eq!(AdtFileType::detect_from_chunks(&chunks), AdtFileType::Obj0);
    }

    #[test]
    fn detect_lod_file_from_chunks() {
        let chunks = HashMap::new();

        assert_eq!(AdtFileType::detect_from_chunks(&chunks), AdtFileType::Lod);
    }

    #[test]
    fn test_detect_from_discovery() {
        let mut discovery = ChunkDiscovery::new(1000);
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MCNK,
            vec![ChunkLocation {
                offset: 100,
                size: 500,
            }],
        );
        discovery.chunks = chunks;

        let file_type = AdtFileType::from_discovery(&discovery);
        assert_eq!(file_type, AdtFileType::Root);
    }

    #[test]
    fn test_detect_tex0_from_discovery() {
        let mut discovery = ChunkDiscovery::new(1000);
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MTEX,
            vec![ChunkLocation {
                offset: 100,
                size: 200,
            }],
        );
        discovery.chunks = chunks;

        let file_type = AdtFileType::from_discovery(&discovery);
        assert_eq!(file_type, AdtFileType::Tex0);
    }

    #[test]
    fn test_detect_obj0_from_discovery() {
        let mut discovery = ChunkDiscovery::new(1000);
        let mut chunks = HashMap::new();
        chunks.insert(
            ChunkId::MODF,
            vec![ChunkLocation {
                offset: 100,
                size: 300,
            }],
        );
        discovery.chunks = chunks;

        let file_type = AdtFileType::from_discovery(&discovery);
        assert_eq!(file_type, AdtFileType::Obj0);
    }

    #[test]
    fn test_detect_lod_from_discovery() {
        let discovery = ChunkDiscovery::new(1000);

        let file_type = AdtFileType::from_discovery(&discovery);
        assert_eq!(file_type, AdtFileType::Lod);
    }

    #[test]
    fn detect_from_filename_tex0() {
        assert_eq!(
            AdtFileType::from_filename("Azeroth_32_48_tex0.adt"),
            AdtFileType::Tex0
        );
    }

    #[test]
    fn detect_from_filename_case_insensitive() {
        assert_eq!(
            AdtFileType::from_filename("AZEROTH_32_48_TEX0.ADT"),
            AdtFileType::Tex0
        );
    }

    #[test]
    fn detect_from_filename_root() {
        assert_eq!(
            AdtFileType::from_filename("Kalimdor_16_32.adt"),
            AdtFileType::Root
        );
    }

    #[test]
    fn file_type_descriptions() {
        assert_eq!(AdtFileType::Root.description(), "Root ADT (terrain data)");
        assert_eq!(
            AdtFileType::Tex0.description(),
            "Texture file 0 (primary textures)"
        );
    }

    #[test]
    fn file_type_display() {
        assert_eq!(format!("{}", AdtFileType::Root), "Root ADT (terrain data)");
    }

    #[test]
    fn is_split_file() {
        assert!(!AdtFileType::Root.is_split_file());
        assert!(AdtFileType::Tex0.is_split_file());
        assert!(AdtFileType::Obj0.is_split_file());
        assert!(AdtFileType::Lod.is_split_file());
    }

    #[test]
    fn has_terrain_geometry() {
        assert!(AdtFileType::Root.has_terrain_geometry());
        assert!(AdtFileType::Lod.has_terrain_geometry());
        assert!(!AdtFileType::Tex0.has_terrain_geometry());
        assert!(!AdtFileType::Obj0.has_terrain_geometry());
    }

    #[test]
    fn has_texture_data() {
        assert!(AdtFileType::Tex0.has_texture_data());
        assert!(AdtFileType::Tex1.has_texture_data());
        assert!(!AdtFileType::Root.has_texture_data());
        assert!(!AdtFileType::Obj0.has_texture_data());
    }

    #[test]
    fn has_object_data() {
        assert!(AdtFileType::Obj0.has_object_data());
        assert!(AdtFileType::Obj1.has_object_data());
        assert!(!AdtFileType::Root.has_object_data());
        assert!(!AdtFileType::Tex0.has_object_data());
    }
}
