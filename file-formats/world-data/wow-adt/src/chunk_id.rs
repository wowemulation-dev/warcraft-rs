use binrw::{BinRead, BinWrite};

/// 4-byte chunk identifier (magic bytes).
///
/// WoW ADT files use reversed magic bytes. When documentation refers to a chunk
/// as "MVER", the actual bytes stored in the file are reversed: `[0x52, 0x45, 0x56, 0x4D]`
/// (which is "REVM" in ASCII). This is because the bytes are stored in little-endian
/// order but interpreted as big-endian strings.
///
/// # File Format Example
///
/// Documentation: "MVER" chunk
/// File bytes: `[0x52, 0x45, 0x56, 0x4D]`
/// ASCII interpretation: "REVM"
/// Display: "MVER" (after reversal)
///
/// # Usage
///
/// ```rust
/// use wow_adt::chunk_id::ChunkId;
///
/// // Use predefined constants
/// let mver = ChunkId::MVER;
/// assert_eq!(mver.as_str(), "MVER");
///
/// // Create from string (automatically reverses)
/// let mcnk = ChunkId::from_str("MCNK").unwrap();
/// assert_eq!(mcnk, ChunkId::MCNK);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BinRead, BinWrite)]
pub struct ChunkId(pub [u8; 4]);

impl ChunkId {
    // Root-level chunks (present in all versions)

    /// Version chunk - stores ADT format version number
    pub const MVER: Self = Self(*b"REVM");

    /// Header chunk - contains offsets to other chunks
    pub const MHDR: Self = Self(*b"RDHM");

    /// Chunk index - MCNK offset table for efficient access
    pub const MCIN: Self = Self(*b"NICM");

    /// Texture filenames - null-terminated strings
    pub const MTEX: Self = Self(*b"XETM");

    /// Model (doodad) filenames - M2 model paths
    pub const MMDX: Self = Self(*b"XDMM");

    /// Model filename indices - offsets into MMDX
    pub const MMID: Self = Self(*b"DIMM");

    /// WMO (World Map Object) filenames
    pub const MWMO: Self = Self(*b"OMWM");

    /// WMO filename indices - offsets into MWMO
    pub const MWID: Self = Self(*b"DIWM");

    /// Doodad (M2) placement definitions
    pub const MDDF: Self = Self(*b"FDDM");

    /// WMO placement definitions
    pub const MODF: Self = Self(*b"FDOM");

    /// Terrain chunk - contains height map, textures, objects (16x16 grid)
    pub const MCNK: Self = Self(*b"KNCM");

    // Version-specific root-level chunks

    /// Flight bounds object (TBC 2.0+) - defines no-fly zones
    pub const MFBO: Self = Self(*b"OBFM");

    /// Water/liquid data (WotLK 3.0+) - replaces MCLQ
    pub const MH2O: Self = Self(*b"O2HM");

    /// Texture flags (WotLK 3.0+) - rendering flags for textures
    pub const MTXF: Self = Self(*b"FXTM");

    /// Texture amplitude/scale (Cataclysm 4.0+)
    pub const MAMP: Self = Self(*b"PMAM");

    /// Texture parameters (MoP 5.0+) - advanced texture properties
    pub const MTXP: Self = Self(*b"PXTM");

    /// Blend mesh headers (MoP 5.0+) - mesh metadata with index/vertex ranges
    pub const MBMH: Self = Self(*b"HMBM");

    /// Blend mesh bounding boxes (MoP 5.0+) - visibility culling boxes
    pub const MBBB: Self = Self(*b"BBBM");

    /// Blend mesh vertices (MoP 5.0+) - vertex data with position/normal/UV/colors
    pub const MBNV: Self = Self(*b"VNBM");

    /// Blend mesh indices (MoP 5.0+) - triangle indices referencing MBNV
    pub const MBMI: Self = Self(*b"IMBM");

    // MCNK subchunks

    /// Height map vertices - 9x9 + 8x8 grid (145 vertices total)
    pub const MCVT: Self = Self(*b"TVCM");

    /// Normal vectors - per-vertex lighting normals
    pub const MCNR: Self = Self(*b"RNCM");

    /// Texture layers - up to 4 texture layers per chunk
    pub const MCLY: Self = Self(*b"YLCM");

    /// Alpha maps - texture blending data
    pub const MCAL: Self = Self(*b"LACM");

    /// Shadow map - baked terrain shadows
    pub const MCSH: Self = Self(*b"HSCM");

    /// Object references - indices into MDDF/MODF (pre-Cataclysm)
    pub const MCRF: Self = Self(*b"FRCM");

    /// Doodad references - M2 model indices into MDDF (Cataclysm+ split files)
    pub const MCRD: Self = Self(*b"DRCM");

    /// WMO references - WMO indices into MODF (Cataclysm+ split files)
    pub const MCRW: Self = Self(*b"WRCM");

    /// Legacy liquid data (pre-WotLK) - replaced by MH2O in 3.0+
    pub const MCLQ: Self = Self(*b"QLCM");

    /// Vertex colors - per-vertex color data (WotLK+)
    pub const MCCV: Self = Self(*b"VCCM");

    /// Vertex lighting - per-vertex ARGB lighting (Cataclysm+)
    pub const MCLV: Self = Self(*b"VLCM");

    /// Terrain materials - material IDs for texture layers (Cataclysm+)
    pub const MCMT: Self = Self(*b"TMCM");

    /// Doodad disable - bitmap for disabling doodads (Cataclysm+)
    pub const MCDD: Self = Self(*b"DDCM");

    /// Blend batches - triangle batches for blend mesh (MoP+)
    pub const MCBB: Self = Self(*b"BBCM");

    /// Sound emitters - ambient sound placement
    pub const MCSE: Self = Self(*b"ESCM");

    /// Convert to human-readable string.
    ///
    /// Reverses the stored bytes to display the chunk name as it appears
    /// in documentation and format specifications.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wow_adt::chunk_id::ChunkId;
    ///
    /// // File contains: [0x52, 0x45, 0x56, 0x4D] (REVM in ASCII)
    /// let chunk = ChunkId::MVER;
    /// assert_eq!(chunk.as_str(), "MVER"); // Displays as documented
    /// ```
    #[must_use]
    pub fn as_str(&self) -> String {
        let reversed = [self.0[3], self.0[2], self.0[1], self.0[0]];
        String::from_utf8_lossy(&reversed).to_string()
    }

    /// Create from string (reverses bytes for file storage).
    ///
    /// Accepts a human-readable chunk name (e.g., "MVER") and converts it
    /// to the reversed byte representation used in ADT files.
    ///
    /// # Arguments
    ///
    /// * `s` - 4-character ASCII string representing the chunk identifier
    ///
    /// # Returns
    ///
    /// * `Some(ChunkId)` if the string is exactly 4 bytes
    /// * `None` if the string length is not 4
    ///
    /// # Example
    ///
    /// ```rust
    /// use wow_adt::chunk_id::ChunkId;
    ///
    /// // Input: "MCNK" (human-readable)
    /// let chunk = ChunkId::from_str("MCNK").unwrap();
    /// // Stored as: [b'K', b'N', b'C', b'M'] (reversed)
    /// assert_eq!(chunk, ChunkId::MCNK);
    ///
    /// // Invalid length
    /// assert!(ChunkId::from_str("ABC").is_none());
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() == 4 {
            Some(Self([bytes[3], bytes[2], bytes[1], bytes[0]]))
        } else {
            None
        }
    }
}

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_id_display_reverses_bytes() {
        // Stored as [b'R', b'E', b'V', b'M'], displays as "MVER"
        assert_eq!(ChunkId::MVER.as_str(), "MVER");
        assert_eq!(ChunkId::MCNK.as_str(), "MCNK");
        assert_eq!(ChunkId::MCVT.as_str(), "MCVT");
    }

    #[test]
    fn chunk_id_from_str_reverses_input() {
        let chunk = ChunkId::from_str("MVER").unwrap();
        assert_eq!(chunk, ChunkId::MVER);
        assert_eq!(chunk.0, [b'R', b'E', b'V', b'M']);
    }

    #[test]
    fn chunk_id_from_str_invalid_length() {
        assert!(ChunkId::from_str("ABC").is_none());
        assert!(ChunkId::from_str("ABCDE").is_none());
        assert!(ChunkId::from_str("").is_none());
    }

    #[test]
    fn chunk_id_display_trait() {
        let chunk = ChunkId::MCNK;
        assert_eq!(format!("{chunk}"), "MCNK");
    }

    #[test]
    fn chunk_id_roundtrip() {
        let original = "MCNK";
        let chunk = ChunkId::from_str(original).unwrap();
        assert_eq!(chunk.as_str(), original);
    }
}
