use binrw::{BinRead, BinWrite};
use std::fmt;

/// A 4-byte chunk identifier for WMO files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BinRead, BinWrite)]
#[br(little)]
#[bw(little)]
pub struct ChunkId {
    pub bytes: [u8; 4],
}

impl ChunkId {
    /// Create a ChunkId from a 4-byte array (as they appear in memory/string form)
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        // Store bytes reversed for little-endian file format
        Self {
            bytes: [bytes[3], bytes[2], bytes[1], bytes[0]],
        }
    }

    /// Get the chunk ID as a string
    pub fn as_str(&self) -> &'static str {
        // The bytes are stored reversed in the file
        // so we match against the reversed patterns
        match &self.bytes {
            b"REVM" => "MVER",
            b"DHOM" => "MOHD",
            b"TMOM" => "MOMT",
            b"XTOM" => "MOTX", // Added
            b"NGOM" => "MOGN",
            b"IGOM" => "MOGI",
            b"BSOM" => "MOSB",
            b"VPOM" => "MOPV",
            b"TPOM" => "MOPT",
            b"RPOM" => "MOPR",
            b"VVOM" => "MOVV", // Added
            b"VBOM" => "MOVB", // Added
            b"BVOM" => "MOVB", // Alternative pattern found in vanilla files
            b"SDOM" => "MODS", // Added
            b"NDOM" => "MODN", // Added
            b"VFOM" => "MFOV",
            b"HPOM" => "MOPH",
            b"BGOM" => "MOGB",
            b"TLOM" => "MOLT",
            b"DDOM" => "MODD",
            b"GDMM" => "MDDG",
            b"GFOM" => "MFOG",
            b"GOFM" => "MFOG", // Alternative pattern found in vanilla files
            b"PVCM" => "MCVP",
            b"DIFG" => "GFID",
            b"PGOM" => "MOGP",
            // Group file chunks
            b"YPOM" => "MOPY",
            b"IVOM" => "MOVI",
            b"TVOM" => "MOVT",
            b"RNOM" => "MONR",
            b"VTOM" => "MOTV",
            b"ABOM" => "MOBA",
            b"RLOM" => "MOLR",
            b"RDOM" => "MODR",
            b"NBOM" => "MOBN",
            b"RBOM" => "MOBR",
            b"VCOM" => "MOCV",
            b"QILM" => "MLIQ",
            b"IROM" => "MORI",
            b"BROM" => "MORB",
            b"ATOM" => "MOTA",
            b"SBOM" => "MOBS",
            _ => "????",
        }
    }
}

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
