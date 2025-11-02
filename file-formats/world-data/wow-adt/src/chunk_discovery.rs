//! Fast chunk discovery phase for two-pass parsing
//!
//! The discovery phase scans ADT files to enumerate all chunks,
//! recording their offsets, sizes, and types without parsing content.
//! This enables selective parsing and version detection before full parse.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

use binrw::BinRead;

use crate::chunk_header::ChunkHeader;
use crate::chunk_id::ChunkId;
use crate::error::{AdtError, Result};

/// Chunk discovery results from Phase 1 of two-pass parsing
///
/// Stores chunk metadata for selective parsing:
/// - Chunk locations and sizes
/// - Grouped by chunk type for quick lookup
/// - File statistics for validation
#[derive(Debug, Clone)]
pub struct ChunkDiscovery {
    /// Map of chunk types to their file offsets
    /// Each ChunkId can appear multiple times (e.g., 256 MCNK chunks)
    pub chunks: HashMap<ChunkId, Vec<ChunkLocation>>,

    /// Total file size in bytes
    pub file_size: u64,

    /// Total number of chunks discovered
    pub total_chunks: usize,
}

/// Location and metadata for a discovered chunk
#[derive(Debug, Clone, Copy)]
pub struct ChunkLocation {
    /// Absolute file offset where chunk header starts
    pub offset: u64,

    /// Chunk data size (excludes 8-byte header)
    pub size: u32,
}

impl ChunkDiscovery {
    /// Create empty discovery result
    pub fn new(file_size: u64) -> Self {
        Self {
            chunks: HashMap::new(),
            file_size,
            total_chunks: 0,
        }
    }

    /// Get all locations for a specific chunk type
    pub fn get_chunks(&self, id: ChunkId) -> Option<&Vec<ChunkLocation>> {
        self.chunks.get(&id)
    }

    /// Check if chunk type exists
    pub fn has_chunk(&self, id: ChunkId) -> bool {
        self.chunks.contains_key(&id)
    }

    /// Count occurrences of chunk type
    pub fn chunk_count(&self, id: ChunkId) -> usize {
        self.chunks.get(&id).map_or(0, |v| v.len())
    }

    /// Get list of all unique chunk types found
    pub fn chunk_types(&self) -> Vec<ChunkId> {
        let mut types: Vec<_> = self.chunks.keys().copied().collect();
        types.sort_by_key(|id| id.0);
        types
    }

    /// Detect ADT file type from discovered chunks.
    ///
    /// Analyzes chunk presence patterns to determine whether this is a root ADT,
    /// texture file, object file, or LOD file. This is a convenience method that
    /// delegates to [`AdtFileType::from_discovery()`](crate::file_type::AdtFileType::from_discovery).
    ///
    /// # Returns
    ///
    /// - `AdtFileType::Root` - Contains MCNK terrain chunks
    /// - `AdtFileType::Tex0` - Contains MTEX texture definitions
    /// - `AdtFileType::Obj0` - Contains MDDF/MODF placement data
    /// - `AdtFileType::Lod` - Minimal chunk set (LOD file)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wow_adt::chunk_discovery::discover_chunks;
    /// use wow_adt::AdtFileType;
    /// use std::fs::File;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut file = File::open("terrain_tex0.adt")?;
    /// let discovery = discover_chunks(&mut file)?;
    ///
    /// let file_type = discovery.detect_file_type();
    /// assert_eq!(file_type, AdtFileType::Tex0);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn detect_file_type(&self) -> crate::file_type::AdtFileType {
        crate::file_type::AdtFileType::from_discovery(self)
    }

    /// Add discovered chunk location
    fn add_chunk(&mut self, id: ChunkId, location: ChunkLocation) {
        self.chunks.entry(id).or_default().push(location);
        self.total_chunks += 1;
    }
}

/// Discover all chunks in an ADT file (Phase 1 of two-pass parsing)
///
/// Performs fast chunk enumeration without parsing content:
/// - Scans entire file reading only 8-byte headers
/// - Records offsets and sizes for each chunk
/// - Validates header integrity and file bounds
/// - Returns discovery results for selective parsing
///
/// # Performance
/// - Target: <10ms for typical ADT files (5-15 MB)
/// - Memory: O(n) where n = number of chunks (~200-300 chunks typical)
///
/// # Example
/// ```no_run
/// use std::fs::File;
/// use wow_adt::chunk_discovery::discover_chunks;
///
/// let mut file = File::open("Kalimdor_32_48.adt")?;
/// let discovery = discover_chunks(&mut file)?;
///
/// println!("Found {} chunks", discovery.total_chunks);
/// println!("Has water data: {}", discovery.has_chunk(wow_adt::ChunkId::MH2O));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn discover_chunks<R: Read + Seek>(reader: &mut R) -> Result<ChunkDiscovery> {
    // Get file size for bounds checking
    let file_size = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;

    // Minimum file size check (MVER header + data minimum)
    const MIN_FILE_SIZE: u64 = 12; // 8-byte header + 4-byte MVER data
    if file_size < MIN_FILE_SIZE {
        return Err(AdtError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("File too small: {} bytes", file_size),
        )));
    }

    let mut discovery = ChunkDiscovery::new(file_size);

    // Scan file for chunks
    while reader.stream_position()? < file_size {
        let chunk_offset = reader.stream_position()?;

        // Read chunk header (8 bytes)
        let header = match ChunkHeader::read_le(reader) {
            Ok(h) => h,
            Err(_) => {
                // End of file or corrupted header
                break;
            }
        };

        // Validate chunk boundaries
        let data_end = chunk_offset + 8 + u64::from(header.size);
        if data_end > file_size {
            log::warn!(
                "Chunk {} at offset {} exceeds file size (chunk ends at {}, file size {})",
                header.id,
                chunk_offset,
                data_end,
                file_size
            );
            break;
        }

        // Record chunk location
        let location = ChunkLocation {
            offset: chunk_offset,
            size: header.size,
        };
        discovery.add_chunk(header.id, location);

        // Skip to next chunk
        reader.seek(SeekFrom::Start(data_end))?;
    }

    log::debug!(
        "Discovery complete: {} chunks, {} unique types",
        discovery.total_chunks,
        discovery.chunks.len()
    );

    Ok(discovery)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunk_discovery_basic() {
        // Create minimal ADT file: MVER chunk
        let mut data = Vec::new();
        data.extend_from_slice(&ChunkId::MVER.0); // Magic (reversed)
        data.extend_from_slice(&4u32.to_le_bytes()); // Size
        data.extend_from_slice(&18u32.to_le_bytes()); // MVER data

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(discovery.total_chunks, 1);
        assert!(discovery.has_chunk(ChunkId::MVER));
        assert_eq!(discovery.chunk_count(ChunkId::MVER), 1);
    }

    #[test]
    fn test_multiple_chunks() {
        let mut data = Vec::new();

        // MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MHDR chunk
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&64u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 64]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(discovery.total_chunks, 2);
        assert!(discovery.has_chunk(ChunkId::MVER));
        assert!(discovery.has_chunk(ChunkId::MHDR));
    }

    #[test]
    fn test_chunk_location_tracking() {
        let mut data = Vec::new();

        // MVER chunk at offset 0
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MHDR chunk at offset 12 (8 + 4)
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 8]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        let mver_chunks = discovery.get_chunks(ChunkId::MVER).unwrap();
        assert_eq!(mver_chunks[0].offset, 0);
        assert_eq!(mver_chunks[0].size, 4);

        let mhdr_chunks = discovery.get_chunks(ChunkId::MHDR).unwrap();
        assert_eq!(mhdr_chunks[0].offset, 12);
        assert_eq!(mhdr_chunks[0].size, 8);
    }

    #[test]
    fn test_duplicate_chunk_types() {
        let mut data = Vec::new();

        // Two MCNK chunks (simulating terrain grid)
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 4]);

        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 4]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(discovery.total_chunks, 2);
        assert_eq!(discovery.chunk_count(ChunkId::MCNK), 2);

        let mcnk_chunks = discovery.get_chunks(ChunkId::MCNK).unwrap();
        assert_eq!(mcnk_chunks.len(), 2);
        assert_eq!(mcnk_chunks[0].offset, 0);
        assert_eq!(mcnk_chunks[1].offset, 12); // 8 + 4
    }

    #[test]
    fn test_chunk_types_sorted() {
        let mut data = Vec::new();

        // Add chunks in non-alphabetical order
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 4]);

        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 4]);

        data.extend_from_slice(&ChunkId::MCIN.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 4]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        let types = discovery.chunk_types();
        // Types should be sorted by raw bytes
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_file_too_small() {
        // File with only 10 bytes (less than minimum)
        let data = vec![0u8; 10];
        let mut cursor = Cursor::new(data);

        let result = discover_chunks(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_chunk() {
        let mut data = Vec::new();

        // Valid MVER chunk
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // Truncated MHDR chunk (header claims 64 bytes but file ends early)
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&64u32.to_le_bytes());
        // Only add 10 bytes instead of 64
        data.extend_from_slice(&[0u8; 10]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        // Should discover MVER but stop at truncated MHDR
        assert_eq!(discovery.total_chunks, 1);
        assert!(discovery.has_chunk(ChunkId::MVER));
        assert!(!discovery.has_chunk(ChunkId::MHDR));
    }

    #[test]
    fn test_empty_discovery() {
        let discovery = ChunkDiscovery::new(100);
        assert_eq!(discovery.total_chunks, 0);
        assert_eq!(discovery.file_size, 100);
        assert!(discovery.chunk_types().is_empty());
        assert!(!discovery.has_chunk(ChunkId::MVER));
    }

    #[test]
    fn test_detect_file_type_root() {
        // Root file has MCNK chunks
        let mut data = Vec::new();

        // MVER
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MCNK (indicates root file)
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 8]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(
            discovery.detect_file_type(),
            crate::file_type::AdtFileType::Root
        );
    }

    #[test]
    fn test_detect_file_type_texture() {
        // Texture file has MTEX but no MCNK
        let mut data = Vec::new();

        // MVER
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MTEX (indicates texture file)
        data.extend_from_slice(&ChunkId::MTEX.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 8]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(
            discovery.detect_file_type(),
            crate::file_type::AdtFileType::Tex0
        );
    }

    #[test]
    fn test_detect_file_type_object() {
        // Object file has MDDF/MODF but no MCNK or MTEX
        let mut data = Vec::new();

        // MVER
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MDDF (indicates object file)
        data.extend_from_slice(&ChunkId::MDDF.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 8]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(
            discovery.detect_file_type(),
            crate::file_type::AdtFileType::Obj0
        );
    }

    #[test]
    fn test_detect_file_type_lod() {
        // LOD file has minimal chunks (just MVER)
        let mut data = Vec::new();

        // MVER only
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(
            discovery.detect_file_type(),
            crate::file_type::AdtFileType::Lod
        );
    }

    #[test]
    fn test_detect_file_type_mcnk_takes_precedence() {
        // Root file has MHDR + MCNK
        let mut data = Vec::new();

        // MVER
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MHDR (header chunk - distinguishes Root from split files)
        data.extend_from_slice(&ChunkId::MHDR.0);
        data.extend_from_slice(&64u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 64]);

        // MTEX
        data.extend_from_slice(&ChunkId::MTEX.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 8]);

        // MCNK (terrain chunks)
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 8]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(
            discovery.detect_file_type(),
            crate::file_type::AdtFileType::Root
        );
    }

    #[test]
    fn test_detect_file_type_tex0_split_file() {
        // Tex0 split file has MTEX + MCNK but NO MHDR
        let mut data = Vec::new();

        // MVER
        data.extend_from_slice(&ChunkId::MVER.0);
        data.extend_from_slice(&4u32.to_le_bytes());
        data.extend_from_slice(&18u32.to_le_bytes());

        // MTEX (texture definitions)
        data.extend_from_slice(&ChunkId::MTEX.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 8]);

        // MCNK (texture sub-chunks, not full terrain chunks)
        data.extend_from_slice(&ChunkId::MCNK.0);
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 8]);

        let mut cursor = Cursor::new(data);
        let discovery = discover_chunks(&mut cursor).unwrap();

        assert_eq!(
            discovery.detect_file_type(),
            crate::file_type::AdtFileType::Tex0
        );
    }
}
