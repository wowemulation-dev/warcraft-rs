//! Block table implementation for MPQ archives

use byteorder::{LittleEndian, ReadBytesExt};
use crate::crypto::{decrypt_block, hash_string, hash_type};
use crate::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// Block table entry (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BlockEntry {
    /// Offset of the beginning of the file data, relative to the beginning of the archive
    pub file_pos: u32,
    /// Compressed file size
    pub compressed_size: u32,
    /// Size of uncompressed file
    pub file_size: u32,
    /// Flags for the file
    pub flags: u32,
}

impl BlockEntry {
    // Flag constants
    /// File is compressed using PKWARE Data compression library
    pub const FLAG_IMPLODE: u32 = 0x00000100;
    /// File is compressed using one or more compression methods
    pub const FLAG_COMPRESS: u32 = 0x00000200;
    /// File is encrypted
    pub const FLAG_ENCRYPTED: u32 = 0x00010000;
    /// The decryption key for the file is adjusted by the block position
    pub const FLAG_FIX_KEY: u32 = 0x00020000;
    /// The file is a patch file
    pub const FLAG_PATCH_FILE: u32 = 0x00100000;
    /// File is stored as a single unit, not split into sectors
    pub const FLAG_SINGLE_UNIT: u32 = 0x01000000;
    /// File is a deletion marker
    pub const FLAG_DELETE_MARKER: u32 = 0x02000000;
    /// File has checksums for each sector
    pub const FLAG_SECTOR_CRC: u32 = 0x04000000;
    /// File exists in the archive
    pub const FLAG_EXISTS: u32 = 0x80000000;

    /// Check if the file is compressed
    pub fn is_compressed(&self) -> bool {
        (self.flags & (Self::FLAG_IMPLODE | Self::FLAG_COMPRESS)) != 0
    }

    /// Check if the file is encrypted
    pub fn is_encrypted(&self) -> bool {
        (self.flags & Self::FLAG_ENCRYPTED) != 0
    }

    /// Check if the file is stored as a single unit
    pub fn is_single_unit(&self) -> bool {
        (self.flags & Self::FLAG_SINGLE_UNIT) != 0
    }

    /// Check if the file has sector CRCs
    pub fn has_sector_crc(&self) -> bool {
        (self.flags & Self::FLAG_SECTOR_CRC) != 0
    }

    /// Check if the file exists
    pub fn exists(&self) -> bool {
        (self.flags & Self::FLAG_EXISTS) != 0
    }

    /// Check if the file uses fixed key encryption
    pub fn has_fix_key(&self) -> bool {
        (self.flags & Self::FLAG_FIX_KEY) != 0
    }

    /// Check if the file is a patch file
    pub fn is_patch_file(&self) -> bool {
        (self.flags & Self::FLAG_PATCH_FILE) != 0
    }

    /// Read a block entry from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(Error::invalid_format("Block entry too small"));
        }

        let mut cursor = std::io::Cursor::new(data);
        Ok(Self {
            file_pos: cursor.read_u32::<LittleEndian>()?,
            compressed_size: cursor.read_u32::<LittleEndian>()?,
            file_size: cursor.read_u32::<LittleEndian>()?,
            flags: cursor.read_u32::<LittleEndian>()?,
        })
    }
}

/// Block table
#[derive(Debug)]
pub struct BlockTable {
    entries: Vec<BlockEntry>,
}

impl BlockTable {
    /// Create a new empty block table
    pub fn new(size: usize) -> Result<Self> {
        let entries = vec![
            BlockEntry {
                file_pos: 0,
                compressed_size: 0,
                file_size: 0,
                flags: 0,
            };
            size
        ];
        Ok(Self { entries })
    }

    /// Read and decrypt a block table from the archive
    pub fn read<R: Read + Seek>(reader: &mut R, offset: u64, size: u32) -> Result<Self> {
        // Seek to block table position
        reader.seek(SeekFrom::Start(offset))?;

        // Read raw data
        let byte_size = size as usize * 16; // 16 bytes per entry
        let mut raw_data = vec![0u8; byte_size];
        reader.read_exact(&mut raw_data)?;

        // Decrypt the table - SAFE VERSION
        let key = hash_string("(block table)", hash_type::FILE_KEY);

        // Convert to u32s, decrypt, then convert back
        let mut u32_buffer: Vec<u32> = raw_data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        decrypt_block(&mut u32_buffer, key);

        // Write decrypted u32s back to bytes
        for (chunk, &decrypted) in raw_data.chunks_exact_mut(4).zip(&u32_buffer) {
            chunk.copy_from_slice(&decrypted.to_le_bytes());
        }

        // Parse entries
        let mut entries = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            let offset = i * 16;
            let entry = BlockEntry::from_bytes(&raw_data[offset..offset + 16])?;
            entries.push(entry);
        }

        Ok(Self { entries })
    }

    /// Create a block table from bytes (needs decryption)
    pub fn from_bytes(data: &[u8], size: u32) -> Result<Self> {
        // Validate data size
        let expected_size = size as usize * 16; // 16 bytes per entry
        if data.len() < expected_size {
            return Err(Error::block_table("Insufficient data for block table"));
        }

        // Copy data for decryption
        let mut raw_data = data[..expected_size].to_vec();

        // Decrypt the table
        let key = hash_string("(block table)", hash_type::FILE_KEY);

        // Convert to u32s, decrypt, then convert back
        let mut u32_buffer: Vec<u32> = raw_data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        decrypt_block(&mut u32_buffer, key);

        // Write decrypted u32s back to bytes
        for (chunk, &decrypted) in raw_data.chunks_exact_mut(4).zip(&u32_buffer) {
            chunk.copy_from_slice(&decrypted.to_le_bytes());
        }

        // Parse entries
        let mut entries = Vec::with_capacity(size as usize);
        for i in 0..size as usize {
            let offset = i * 16;
            let entry = BlockEntry::from_bytes(&raw_data[offset..offset + 16])?;
            entries.push(entry);
        }

        Ok(Self { entries })
    }

    /// Get all entries
    pub fn entries(&self) -> &[BlockEntry] {
        &self.entries
    }

    /// Get a specific entry
    pub fn get(&self, index: usize) -> Option<&BlockEntry> {
        self.entries.get(index)
    }

    /// Get the size of the block table
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Create a new block table with mutable entries
    pub fn new_mut(size: usize) -> Result<Self> {
        let entries = vec![
            BlockEntry {
                file_pos: 0,
                compressed_size: 0,
                file_size: 0,
                flags: 0,
            };
            size
        ];
        Ok(Self { entries })
    }

    /// Get a mutable reference to a specific entry
    pub fn get_mut(&mut self, index: usize) -> Option<&mut BlockEntry> {
        self.entries.get_mut(index)
    }

    /// Get mutable access to all entries
    pub fn entries_mut(&mut self) -> &mut [BlockEntry] {
        &mut self.entries
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = BlockEntry {
                file_pos: 0,
                compressed_size: 0,
                file_size: 0,
                flags: 0,
            };
        }
    }
}

/// Hi-block table for archives > 4GB (v2+)
#[derive(Debug)]
pub struct HiBlockTable {
    entries: Vec<u16>,
}

impl HiBlockTable {
    /// Read the hi-block table
    pub fn read<R: Read + Seek>(reader: &mut R, offset: u64, size: u32) -> Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;

        let mut entries = Vec::with_capacity(size as usize);
        for _ in 0..size {
            entries.push(reader.read_u16::<LittleEndian>()?);
        }

        Ok(Self { entries })
    }

    /// Get a hi-block entry
    pub fn get(&self, index: usize) -> Option<u16> {
        self.entries.get(index).copied()
    }

    /// Calculate full 64-bit file position
    pub fn get_file_pos_high(&self, index: usize) -> u64 {
        self.get(index).unwrap_or(0) as u64
    }

    /// Create a new Hi-block table with the given size
    pub fn new(size: usize) -> Self {
        Self {
            entries: vec![0; size],
        }
    }

    /// Set a hi-block entry
    pub fn set(&mut self, index: usize, value: u16) {
        if let Some(entry) = self.entries.get_mut(index) {
            *entry = value;
        }
    }

    /// Get all entries
    pub fn entries(&self) -> &[u16] {
        &self.entries
    }

    /// Check if any entry has a non-zero value (i.e., if the table is needed)
    pub fn is_needed(&self) -> bool {
        self.entries.iter().any(|&v| v != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_entry_flags() {
        let compressed = BlockEntry {
            file_pos: 0,
            compressed_size: 100,
            file_size: 200,
            flags: BlockEntry::FLAG_COMPRESS | BlockEntry::FLAG_EXISTS,
        };
        assert!(compressed.is_compressed());
        assert!(!compressed.is_encrypted());
        assert!(compressed.exists());

        let encrypted = BlockEntry {
            file_pos: 0,
            compressed_size: 100,
            file_size: 100,
            flags: BlockEntry::FLAG_ENCRYPTED | BlockEntry::FLAG_FIX_KEY | BlockEntry::FLAG_EXISTS,
        };
        assert!(encrypted.is_encrypted());
        assert!(encrypted.has_fix_key());
        assert!(!encrypted.is_compressed());
    }
}
