use crate::chunk_id::ChunkId;
use binrw::{BinRead, BinWrite};

/// A chunk header containing the ID and size
#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
#[br(little)]
#[bw(little)]
pub struct ChunkHeader {
    /// The chunk identifier
    pub id: ChunkId,
    /// The size of the chunk data (excluding the header)
    pub size: u32,
}
