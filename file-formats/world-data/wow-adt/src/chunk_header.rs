//! ADT chunk header parsing
//!
//! All ADT chunks follow a standard 8-byte header structure consisting of a 4-byte
//! magic identifier and a 4-byte size field. This module provides the core type for
//! parsing these headers.

use binrw::{BinRead, BinWrite};

use crate::chunk_id::ChunkId;

/// Standard ADT chunk header (8 bytes)
///
/// All chunks in ADT files follow this structure:
/// - 4 bytes: Magic identifier (reversed string, e.g., "MVER" stored as [0x52, 0x45, 0x56, 0x4D])
/// - 4 bytes: Data size (little-endian u32, excludes the 8-byte header)
///
/// # Binary Layout
///
/// ```text
/// Offset | Size | Field | Description
/// -------|------|-------|------------------------------------------
/// 0x00   |  4   | id    | Chunk magic identifier (reversed)
/// 0x04   |  4   | size  | Data size in bytes (excludes header)
/// ```
///
/// # Example
///
/// ```text
/// File bytes: [0x52, 0x45, 0x56, 0x4D] [0x04, 0x00, 0x00, 0x00] [0x12, 0x00, 0x00, 0x00]
///             └────── "REVM" ────────┘ └──── size: 4 ────────┘ └─── version data ──┘
///             (displays as "MVER")
/// ```
///
/// # Size Field Semantics
///
/// The size field represents the byte count of chunk data EXCLUDING the 8-byte header.
/// This is a common source of off-by-8 errors when calculating file offsets.
///
/// If size = 100, then:
/// - Chunk data occupies bytes [8..108] relative to chunk start
/// - Total chunk size (header + data) = 108 bytes
/// - Next chunk starts at offset 108
///
/// # Usage with binrw
///
/// ```rust,no_run
/// use binrw::BinRead;
/// use std::io::Cursor;
/// use wow_adt::chunk_header::ChunkHeader;
///
/// # fn example() -> binrw::BinResult<()> {
/// let data = [0x52, 0x45, 0x56, 0x4D, 0x04, 0x00, 0x00, 0x00];
/// let mut cursor = Cursor::new(&data);
///
/// let header = ChunkHeader::read(&mut cursor)?;
/// assert_eq!(header.size, 4);
/// assert_eq!(header.total_size(), 12); // 8-byte header + 4 bytes data
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[brw(little)]
pub struct ChunkHeader {
    /// Chunk magic identifier (4 bytes, reversed)
    ///
    /// Identifiers are stored in reverse byte order. For example, the "MVER" chunk
    /// is stored as [0x52, 0x45, 0x56, 0x4D] ("REVM" in ASCII).
    pub id: ChunkId,

    /// Size of chunk data in bytes (excludes 8-byte header)
    ///
    /// This field specifies the size of the chunk data following the header.
    /// The total chunk size is `size + 8` bytes.
    pub size: u32,
}

impl ChunkHeader {
    /// Total size including header (size + 8)
    ///
    /// Returns the complete chunk size including the 8-byte header.
    /// Use this when calculating file offsets to the next chunk.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wow_adt::chunk_header::ChunkHeader;
    /// use wow_adt::chunk_id::ChunkId;
    ///
    /// let header = ChunkHeader {
    ///     id: ChunkId::from_str("MVER").unwrap(),
    ///     size: 100,
    /// };
    ///
    /// assert_eq!(header.total_size(), 108); // 100 + 8
    /// ```
    #[must_use]
    pub const fn total_size(&self) -> u64 {
        self.size as u64 + 8
    }

    /// Check if chunk ID matches expected value
    ///
    /// Convenience method for validating chunk types during parsing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wow_adt::chunk_header::ChunkHeader;
    /// use wow_adt::chunk_id::ChunkId;
    ///
    /// let header = ChunkHeader {
    ///     id: ChunkId::from_str("MVER").unwrap(),
    ///     size: 4,
    /// };
    ///
    /// assert!(header.is_chunk(ChunkId::from_str("MVER").unwrap()));
    /// assert!(!header.is_chunk(ChunkId::from_str("MHDR").unwrap()));
    /// ```
    #[must_use]
    pub fn is_chunk(&self, expected: ChunkId) -> bool {
        self.id.0 == expected.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::BinRead;
    use std::io::Cursor;

    #[test]
    fn parse_chunk_header() {
        let data = [
            0x52, 0x45, 0x56, 0x4D, // "MVER" reversed
            0x04, 0x00, 0x00, 0x00, // size: 4
        ];

        let mut cursor = Cursor::new(&data);
        let header = ChunkHeader::read(&mut cursor).expect("parse chunk header");

        assert_eq!(header.id, ChunkId::from_str("MVER").unwrap());
        assert_eq!(header.size, 4);
        assert_eq!(header.total_size(), 12);
    }

    #[test]
    fn total_size_calculation() {
        let header = ChunkHeader {
            id: ChunkId::from_str("TEST").unwrap(),
            size: 100,
        };

        assert_eq!(header.total_size(), 108);
    }

    #[test]
    fn is_chunk_validation() {
        let header = ChunkHeader {
            id: ChunkId::from_str("MVER").unwrap(),
            size: 4,
        };

        assert!(header.is_chunk(ChunkId::from_str("MVER").unwrap()));
        assert!(!header.is_chunk(ChunkId::from_str("MHDR").unwrap()));
    }

    #[test]
    fn zero_size_chunk() {
        let data = [
            0x54, 0x53, 0x45, 0x54, // "TEST" reversed
            0x00, 0x00, 0x00, 0x00, // size: 0
        ];

        let mut cursor = Cursor::new(&data);
        let header = ChunkHeader::read(&mut cursor).expect("parse zero size chunk");

        assert_eq!(header.size, 0);
        assert_eq!(header.total_size(), 8);
    }
}
