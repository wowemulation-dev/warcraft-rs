use crate::error::{Result, WmoError};
use crate::types::ChunkId;
use std::io::{Read, Seek, SeekFrom, Write};

/// Helper function to handle `read_exact` operations with proper EOF handling
fn read_exact_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<()> {
    match reader.read_exact(buf) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Err(WmoError::UnexpectedEof),
        Err(e) => Err(WmoError::Io(e)),
    }
}

/// Represents a chunk header in a WMO file
#[derive(Debug, Clone, Copy)]
pub struct ChunkHeader {
    /// 4-byte chunk identifier (magic)
    pub id: ChunkId,
    /// Size of the chunk data in bytes (not including this header)
    pub size: u32,
}

impl ChunkHeader {
    /// Size of a chunk header in bytes
    pub const SIZE: usize = 8;

    /// Read a chunk header from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut id_bytes = [0u8; 4];
        read_exact_or_eof(reader, &mut id_bytes)?;

        // WMO files store chunk IDs in little-endian order, so we need to reverse
        // the bytes to get the correct ASCII representation
        id_bytes.reverse();

        let mut size_bytes = [0u8; 4];
        read_exact_or_eof(reader, &mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes);

        Ok(Self {
            id: ChunkId(id_bytes),
            size,
        })
    }

    /// Write a chunk header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // WMO files store chunk IDs in little-endian order, so we need to reverse
        // the bytes when writing
        let mut id_bytes = self.id.0;
        id_bytes.reverse();
        writer.write_all(&id_bytes)?;
        writer.write_all(&self.size.to_le_bytes())?;
        Ok(())
    }
}

/// Represents a chunk in a WMO file
#[derive(Debug)]
pub struct Chunk {
    /// Chunk header
    pub header: ChunkHeader,
    /// Position of the chunk data in the file
    pub data_position: u64,
}

impl Chunk {
    /// Read a chunk from a reader
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let header = ChunkHeader::read(reader)?;
        let data_position = reader.stream_position()?;

        // Skip the chunk data for now
        reader.seek(SeekFrom::Current(header.size as i64))?;

        Ok(Self {
            header,
            data_position,
        })
    }

    /// Read a specific chunk from a reader, checking the expected ID
    pub fn read_expected<R: Read + Seek>(reader: &mut R, expected_id: ChunkId) -> Result<Self> {
        let chunk = Self::read(reader)?;

        if chunk.header.id != expected_id {
            return Err(WmoError::InvalidMagic {
                expected: *expected_id.as_bytes(),
                found: chunk.header.id.0,
            });
        }

        Ok(chunk)
    }

    /// Seek to the data portion of this chunk
    pub fn seek_to_data<S: Seek>(&self, seeker: &mut S) -> Result<()> {
        seeker.seek(SeekFrom::Start(self.data_position))?;
        Ok(())
    }

    /// Read the data of this chunk into a buffer
    pub fn read_data<R: Read + Seek>(&self, reader: &mut R) -> Result<Vec<u8>> {
        self.seek_to_data(reader)?;

        let mut data = vec![0; self.header.size as usize];
        reader.read_exact(&mut data)?;

        Ok(data)
    }
}
