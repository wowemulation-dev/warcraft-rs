//! Hash table implementation for MPQ archives

use super::common::ReadLittleEndian;
use crate::crypto::{decrypt_block, hash_string, hash_type};
use crate::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// Hash table entry (16 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HashEntry {
    /// The hash of the full file name (part A)
    pub name_1: u32,
    /// The hash of the full file name (part B)
    pub name_2: u32,
    /// The language of the file (Windows LANGID)
    pub locale: u16,
    /// The platform the file is used for (vestigial - always 0 in practice)
    pub platform: u16,
    /// Block table index or special value
    pub block_index: u32,
}

impl HashEntry {
    /// Value indicating the hash entry has never been used
    pub const EMPTY_NEVER_USED: u32 = 0xFFFFFFFF;
    /// Value indicating the hash entry was deleted
    pub const EMPTY_DELETED: u32 = 0xFFFFFFFE;

    /// Create an empty hash entry
    pub fn empty() -> Self {
        Self {
            name_1: 0,
            name_2: 0,
            locale: 0,
            platform: 0,
            block_index: Self::EMPTY_NEVER_USED,
        }
    }

    /// Check if this entry has never been used
    pub fn is_empty(&self) -> bool {
        self.block_index == Self::EMPTY_NEVER_USED
    }

    /// Check if this entry was deleted
    pub fn is_deleted(&self) -> bool {
        self.block_index == Self::EMPTY_DELETED
    }

    /// Check if this entry contains valid file information
    pub fn is_valid(&self) -> bool {
        self.block_index < Self::EMPTY_DELETED
    }

    /// Read a hash entry from raw bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < 16 {
            return Err(Error::invalid_format("Hash entry too small"));
        }

        let mut cursor = std::io::Cursor::new(data);
        Ok(Self {
            name_1: cursor.read_u32_le()?,
            name_2: cursor.read_u32_le()?,
            locale: cursor.read_u16_le()?,
            platform: cursor.read_u16_le()?,
            block_index: cursor.read_u32_le()?,
        })
    }
}

/// Hash table
#[derive(Debug)]
pub struct HashTable {
    entries: Vec<HashEntry>,
    mask: usize,
}

impl HashTable {
    /// Create a new empty hash table
    pub fn new(size: usize) -> Result<Self> {
        // Validate size is power of 2
        if !crate::is_power_of_two(size as u32) {
            return Err(Error::hash_table("Hash table size must be power of 2"));
        }

        let entries = vec![HashEntry::empty(); size];
        Ok(Self {
            entries,
            mask: size - 1,
        })
    }

    /// Read and decrypt a hash table from the archive
    pub fn read<R: Read + Seek>(reader: &mut R, offset: u64, size: u32) -> Result<Self> {
        // Validate size
        if !crate::is_power_of_two(size) {
            return Err(Error::hash_table("Hash table size must be power of 2"));
        }

        // Seek to hash table position
        reader.seek(SeekFrom::Start(offset))?;

        // Read raw data
        let byte_size = size as usize * 16; // 16 bytes per entry
        let mut raw_data = vec![0u8; byte_size];
        reader.read_exact(&mut raw_data)?;

        // Decrypt the table - SAFE VERSION
        let key = hash_string("(hash table)", hash_type::FILE_KEY);

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
            let entry = HashEntry::from_bytes(&raw_data[offset..offset + 16])?;
            entries.push(entry);
        }

        Ok(Self {
            entries,
            mask: size as usize - 1,
        })
    }

    /// Create a hash table from bytes (needs decryption)
    pub fn from_bytes(data: &[u8], size: u32) -> Result<Self> {
        // Validate size
        if !crate::is_power_of_two(size) {
            return Err(Error::hash_table("Hash table size must be power of 2"));
        }

        // Validate data size
        let expected_size = size as usize * 16; // 16 bytes per entry
        if data.len() < expected_size {
            return Err(Error::hash_table("Insufficient data for hash table"));
        }

        // Copy data for decryption
        let mut raw_data = data[..expected_size].to_vec();

        // Decrypt the table
        let key = hash_string("(hash table)", hash_type::FILE_KEY);

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
            let entry = HashEntry::from_bytes(&raw_data[offset..offset + 16])?;
            entries.push(entry);
        }

        Ok(Self {
            entries,
            mask: size as usize - 1,
        })
    }

    /// Get all entries
    pub fn entries(&self) -> &[HashEntry] {
        &self.entries
    }

    /// Get a specific entry
    pub fn get(&self, index: usize) -> Option<&HashEntry> {
        self.entries.get(index)
    }

    /// Get the size of the hash table
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Find a file in the hash table
    pub fn find_file(&self, filename: &str, locale: u16) -> Option<(usize, &HashEntry)> {
        // Calculate hash values
        let name_a = hash_string(filename, hash_type::NAME_A);
        let name_b = hash_string(filename, hash_type::NAME_B);
        let start_index = hash_string(filename, hash_type::TABLE_OFFSET) as usize;

        let mut index = start_index & self.mask;
        let end_index = index;

        // Linear probing to find the file
        loop {
            let entry = &self.entries[index];

            // Check if this is our file
            if entry.name_1 == name_a && entry.name_2 == name_b {
                // Check locale (0 = default/any locale)
                if (locale == 0 || entry.locale == 0 || entry.locale == locale) && entry.is_valid()
                {
                    return Some((index, entry));
                }
            }

            // If we hit an empty entry that was never used, file doesn't exist
            if entry.is_empty() {
                return None;
            }

            // Continue to next entry
            index = (index + 1) & self.mask;

            // If we've wrapped around to where we started, file doesn't exist
            if index == end_index {
                return None;
            }
        }
    }

    /// Create a new hash table with mutable entries
    pub fn new_mut(size: usize) -> Result<Self> {
        // Validate size is power of 2
        if !crate::is_power_of_two(size as u32) {
            return Err(Error::hash_table("Hash table size must be power of 2"));
        }

        let entries = vec![HashEntry::empty(); size];
        Ok(Self {
            entries,
            mask: size - 1,
        })
    }

    /// Get a mutable reference to a specific entry
    pub fn get_mut(&mut self, index: usize) -> Option<&mut HashEntry> {
        self.entries.get_mut(index)
    }

    /// Get mutable access to all entries
    pub fn entries_mut(&mut self) -> &mut [HashEntry] {
        &mut self.entries
    }

    /// Clear all entries to empty state
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = HashEntry::empty();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_entry_states() {
        let empty = HashEntry::empty();
        assert!(empty.is_empty());
        assert!(!empty.is_deleted());
        assert!(!empty.is_valid());

        let deleted = HashEntry {
            name_1: 0,
            name_2: 0,
            locale: 0,
            platform: 0,
            block_index: HashEntry::EMPTY_DELETED,
        };
        assert!(!deleted.is_empty());
        assert!(deleted.is_deleted());
        assert!(!deleted.is_valid());

        let valid = HashEntry {
            name_1: 0x12345678,
            name_2: 0x9ABCDEF0,
            locale: 0,
            platform: 0,
            block_index: 0,
        };
        assert!(!valid.is_empty());
        assert!(!valid.is_deleted());
        assert!(valid.is_valid());
    }

    #[test]
    fn test_hash_table_size_validation() {
        // Valid sizes (powers of 2)
        assert!(HashTable::new(16).is_ok());
        assert!(HashTable::new(256).is_ok());
        assert!(HashTable::new(4096).is_ok());

        // Invalid sizes
        assert!(HashTable::new(15).is_err());
        assert!(HashTable::new(100).is_err());
        assert!(HashTable::new(0).is_err());
    }
}
