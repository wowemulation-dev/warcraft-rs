//! Chunk infrastructure for M2 chunked format (MD21+)
//!
//! This module provides the core infrastructure for parsing chunked M2 files
//! introduced in Legion. Chunks allow for modular data storage and external
//! file references using FileDataIDs.

use std::io::{Read, Seek, SeekFrom};

use crate::error::{M2Error, Result};
use crate::io_ext::ReadExt;

/// A chunk header containing magic and size information
#[derive(Debug, Clone)]
pub struct ChunkHeader {
    /// 4-byte magic identifier for the chunk type
    pub magic: [u8; 4],
    /// Size of the chunk data in bytes (excluding header)
    pub size: u32,
}

impl ChunkHeader {
    /// Read a chunk header from a reader
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        let size = reader.read_u32_le()?;

        Ok(ChunkHeader { magic, size })
    }

    /// Get the magic as a string for debugging
    pub fn magic_str(&self) -> String {
        String::from_utf8_lossy(&self.magic).to_string()
    }

    /// Check if this chunk has the specified magic
    pub fn has_magic(&self, magic: &[u8; 4]) -> bool {
        &self.magic == magic
    }
}

/// A chunk reader that manages chunk boundaries and offset calculations
pub struct ChunkReader<R> {
    inner: R,
    chunk_start: u64,
    chunk_size: u32,
}

impl<R: Read + Seek> ChunkReader<R> {
    /// Create a new chunk reader from a reader and chunk header
    /// The reader should be positioned at the start of chunk data (after header)
    pub fn new(mut reader: R, header: ChunkHeader) -> Result<Self> {
        let chunk_start = reader.stream_position()?;
        Ok(ChunkReader {
            inner: reader,
            chunk_start,
            chunk_size: header.size,
        })
    }

    /// Resolve a chunk-relative offset to an absolute file position
    pub fn resolve_offset(&self, offset: u32) -> u64 {
        self.chunk_start + offset as u64
    }

    /// Get the current position within the chunk (relative to chunk start)
    pub fn chunk_position(&mut self) -> Result<u32> {
        let current = self.inner.stream_position()?;
        Ok((current - self.chunk_start) as u32)
    }

    /// Get the remaining bytes in the chunk
    pub fn remaining(&mut self) -> Result<u32> {
        let pos = self.chunk_position()?;
        Ok(self.chunk_size.saturating_sub(pos))
    }

    /// Check if we've reached the end of the chunk
    pub fn is_at_end(&mut self) -> Result<bool> {
        Ok(self.remaining()? == 0)
    }

    /// Seek to a chunk-relative position
    pub fn seek_to_chunk_offset(&mut self, offset: u32) -> Result<()> {
        if offset > self.chunk_size {
            return Err(M2Error::ParseError(format!(
                "Chunk offset {} exceeds chunk size {}",
                offset, self.chunk_size
            )));
        }

        let absolute_pos = self.chunk_start + offset as u64;
        self.inner.seek(SeekFrom::Start(absolute_pos))?;
        Ok(())
    }

    /// Skip to the end of the chunk
    pub fn skip_to_end(&mut self) -> Result<()> {
        let end_pos = self.chunk_start + self.chunk_size as u64;
        self.inner.seek(SeekFrom::Start(end_pos))?;
        Ok(())
    }

    /// Get the chunk size
    pub fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Get a reference to the underlying reader
    pub fn inner(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Get the current absolute position in the file
    pub fn current_position(&mut self) -> Result<u64> {
        Ok(self.inner.stream_position()?)
    }

    /// Seek to an absolute position in the file
    pub fn seek_to_position(&mut self, position: u64) -> Result<()> {
        self.inner.seek(SeekFrom::Start(position))?;
        Ok(())
    }
}

impl<R: Read + Seek> Read for ChunkReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Limit reads to the chunk boundary
        let remaining = match self.remaining() {
            Ok(r) => r as usize,
            Err(_) => {
                return Err(std::io::Error::other("Failed to get remaining chunk bytes"));
            }
        };

        if remaining == 0 {
            return Ok(0);
        }

        let to_read = buf.len().min(remaining);
        self.inner.read(&mut buf[..to_read])
    }
}

impl<R: Seek> Seek for ChunkReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(pos) => {
                // Interpret as chunk-relative position
                if pos > self.chunk_size as u64 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "Seek position {} exceeds chunk size {}",
                            pos, self.chunk_size
                        ),
                    ));
                }
                let absolute_pos = self.chunk_start + pos;
                self.inner.seek(SeekFrom::Start(absolute_pos))
            }
            SeekFrom::End(offset) => {
                // Seek from end of chunk
                let end_pos = self.chunk_start + self.chunk_size as u64;
                let target = end_pos as i64 + offset;
                if target < self.chunk_start as i64 || target > end_pos as i64 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Seek position outside chunk bounds",
                    ));
                }
                self.inner.seek(SeekFrom::Start(target as u64))
            }
            SeekFrom::Current(offset) => {
                // Seek relative to current position, but stay within chunk
                let current = self.inner.stream_position()?;
                let target = current as i64 + offset;
                let chunk_end = self.chunk_start + self.chunk_size as u64;

                if target < self.chunk_start as i64 || target > chunk_end as i64 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Seek position outside chunk bounds",
                    ));
                }
                self.inner.seek(SeekFrom::Start(target as u64))
            }
        }
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        // Return position relative to chunk start
        let absolute_pos = self.inner.stream_position()?;
        Ok(absolute_pos - self.chunk_start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_chunk_header_read() {
        let data = b"TEST\x10\x00\x00\x00"; // "TEST" magic with size 16
        let mut cursor = Cursor::new(data);

        let header = ChunkHeader::read(&mut cursor).unwrap();
        assert_eq!(&header.magic, b"TEST");
        assert_eq!(header.size, 16);
        assert_eq!(header.magic_str(), "TEST");
        assert!(header.has_magic(b"TEST"));
        assert!(!header.has_magic(b"FAIL"));
    }

    #[test]
    fn test_chunk_reader_basic() {
        let data = b"TEST\x08\x00\x00\x00abcdefgh"; // Header + 8 bytes of data
        let mut cursor = Cursor::new(data);

        let header = ChunkHeader::read(&mut cursor).unwrap();
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        assert_eq!(chunk_reader.chunk_size(), 8);
        assert_eq!(chunk_reader.remaining().unwrap(), 8);
        assert!(!chunk_reader.is_at_end().unwrap());

        let mut buf = [0u8; 4];
        assert_eq!(chunk_reader.read(&mut buf).unwrap(), 4);
        assert_eq!(&buf, b"abcd");

        assert_eq!(chunk_reader.remaining().unwrap(), 4);
        assert_eq!(chunk_reader.chunk_position().unwrap(), 4);
    }

    #[test]
    fn test_chunk_reader_seek() {
        let data = b"TEST\x08\x00\x00\x00abcdefgh";
        let mut cursor = Cursor::new(data);

        let header = ChunkHeader::read(&mut cursor).unwrap();
        let mut chunk_reader = ChunkReader::new(cursor, header).unwrap();

        // Seek to position 2 within chunk
        chunk_reader.seek_to_chunk_offset(2).unwrap();
        assert_eq!(chunk_reader.chunk_position().unwrap(), 2);

        let mut buf = [0u8; 2];
        assert_eq!(chunk_reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"cd");

        // Test seek bounds
        assert!(chunk_reader.seek_to_chunk_offset(10).is_err()); // Beyond chunk
    }

    #[test]
    fn test_chunk_reader_offset_resolution() {
        let data = b"TEST\x08\x00\x00\x00abcdefgh";
        let mut cursor = Cursor::new(data);

        let header = ChunkHeader::read(&mut cursor).unwrap();
        let chunk_reader = ChunkReader::new(cursor, header).unwrap();

        // Chunk starts at position 8 (after 8-byte header)
        assert_eq!(chunk_reader.resolve_offset(0), 8);
        assert_eq!(chunk_reader.resolve_offset(4), 12);
    }
}
