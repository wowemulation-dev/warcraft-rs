use crate::chunk_header::ChunkHeader;
use crate::chunk_id::ChunkId;
use binrw::BinRead;
use std::io::{Read, Seek, SeekFrom};

/// Information about a discovered chunk
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    /// The chunk identifier
    pub id: ChunkId,
    /// Offset in the file where the chunk starts
    pub offset: u64,
    /// Size of the chunk data
    pub size: u32,
}

/// Result of chunk discovery process
#[derive(Debug, Clone)]
pub struct ChunkDiscovery {
    /// All discovered chunks in order
    pub chunks: Vec<ChunkInfo>,
    /// Total file size
    pub file_size: u64,
    /// Number of malformed chunks encountered
    malformed_chunks: u32,
    /// Number of unknown chunks encountered
    unknown_chunks: u32,
    /// Whether the file appears truncated
    truncated: bool,
}

impl ChunkDiscovery {
    /// Get the total number of chunks discovered
    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }

    /// Check if any malformed chunks were encountered
    pub fn has_malformed_chunks(&self) -> bool {
        self.malformed_chunks > 0
    }

    /// Get the count of malformed chunks
    pub fn malformed_count(&self) -> u32 {
        self.malformed_chunks
    }

    /// Check if any unknown chunks were encountered
    pub fn has_unknown_chunks(&self) -> bool {
        self.unknown_chunks > 0
    }

    /// Get the count of unknown chunks
    pub fn unknown_count(&self) -> u32 {
        self.unknown_chunks
    }

    /// Check if the file appears truncated
    pub fn is_truncated(&self) -> bool {
        self.truncated
    }
}

/// Discover all chunks in a WMO file
pub fn discover_chunks<R: Read + Seek>(
    reader: &mut R,
) -> Result<ChunkDiscovery, Box<dyn std::error::Error>> {
    let mut chunks = Vec::new();
    let mut malformed_chunks = 0u32;
    let mut unknown_chunks = 0u32;
    let mut truncated = false;

    // Get file size
    let file_size = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;

    // Read chunks until end of file
    while reader.stream_position()? < file_size {
        let offset = reader.stream_position()?;

        // Try to read chunk header
        let header = match ChunkHeader::read(reader) {
            Ok(h) => h,
            Err(_) => {
                // Malformed chunk - try to recover by breaking
                malformed_chunks += 1;
                break;
            }
        };

        // Check if chunk ID is unknown
        if header.id.as_str() == "????" {
            unknown_chunks += 1;
        }

        // Check if chunk size is reasonable (prevent overflow)
        let remaining = file_size - reader.stream_position()?;
        if header.size as u64 > remaining {
            // File is truncated or chunk size is invalid
            if header.size > 0x10000000 {
                // Likely malformed (>256MB chunk)
                malformed_chunks += 1;
                break;
            } else {
                // Likely truncated
                truncated = true;
                chunks.push(ChunkInfo {
                    id: header.id,
                    offset,
                    size: header.size,
                });
                break;
            }
        }

        chunks.push(ChunkInfo {
            id: header.id,
            offset,
            size: header.size,
        });

        // Skip chunk data
        reader.seek(SeekFrom::Current(header.size as i64))?;
    }

    Ok(ChunkDiscovery {
        chunks,
        file_size,
        malformed_chunks,
        unknown_chunks,
        truncated,
    })
}
