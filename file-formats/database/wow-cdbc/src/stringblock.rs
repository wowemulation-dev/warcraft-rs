//! String block parsing functionality

use crate::{Error, Result, StringRef};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;

/// Represents a string block in a DBC file
#[derive(Debug, Clone)]
pub struct StringBlock {
    /// The raw bytes of the string block
    data: Vec<u8>,
}

impl StringBlock {
    /// Parse a string block from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R, offset: u64, size: u32) -> Result<Self> {
        reader.seek(SeekFrom::Start(offset))?;

        let mut data = vec![0u8; size as usize];
        reader.read_exact(&mut data)?;

        Ok(Self { data })
    }

    /// Get a string from the string block using a string reference
    pub fn get_string(&self, string_ref: StringRef) -> Result<&str> {
        let offset = string_ref.offset() as usize;
        if offset >= self.data.len() {
            return Err(Error::OutOfBounds(format!(
                "String reference offset out of bounds: {} (max: {})",
                offset,
                self.data.len()
            )));
        }

        // Find the end of the string (null terminator)
        let mut end = offset;
        while end < self.data.len() && self.data[end] != 0 {
            end += 1;
        }

        // Convert the bytes to a string
        std::str::from_utf8(&self.data[offset..end])
            .map_err(|e| Error::TypeConversion(format!("Invalid UTF-8 string: {e}")))
    }

    /// Get the raw data of the string block
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the size of the string block in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if an offset is the start of a string in the block
    ///
    /// A valid string start is either at offset 0 (beginning of block)
    /// or immediately after a NUL terminator (byte at offset-1 is 0).
    pub fn is_string_start(&self, offset: u32) -> bool {
        let offset = offset as usize;
        if offset >= self.data.len() {
            return false;
        }
        // Offset 0 is always a valid string start
        // Otherwise, the previous byte must be a NUL terminator
        offset == 0 || self.data[offset - 1] == 0
    }
}

/// A cached string block for efficient string lookups
#[derive(Debug, Clone)]
pub struct CachedStringBlock {
    /// The raw bytes of the string block
    data: Arc<Vec<u8>>,
    /// Cache of string references to string slices
    cache: HashMap<u32, (usize, usize)>,
}

impl CachedStringBlock {
    /// Create a cached string block from a string block
    pub fn from_string_block(string_block: &StringBlock) -> Self {
        let data = Arc::new(string_block.data().to_vec());
        let mut cache = HashMap::new();

        let mut offset = 0;
        while offset < data.len() {
            let start_offset = offset;

            // Find the end of the string (null terminator)
            while offset < data.len() && data[offset] != 0 {
                offset += 1;
            }

            // Cache the string position
            cache.insert(start_offset as u32, (start_offset, offset));

            // Skip the null terminator
            offset += 1;
        }

        Self { data, cache }
    }

    /// Get a string from the string block using a string reference
    pub fn get_string(&self, string_ref: StringRef) -> Result<&str> {
        let offset = string_ref.offset() as usize;

        if let Some((start, end)) = self.cache.get(&string_ref.offset()) {
            // Convert the bytes to a string
            std::str::from_utf8(&self.data[*start..*end])
                .map_err(|e| Error::TypeConversion(format!("Invalid UTF-8 string: {e}")))
        } else {
            // If not cached, find the end of the string
            if offset >= self.data.len() {
                return Err(Error::OutOfBounds(format!(
                    "String reference offset out of bounds: {} (max: {})",
                    offset,
                    self.data.len()
                )));
            }

            let mut end = offset;
            while end < self.data.len() && self.data[end] != 0 {
                end += 1;
            }

            // Convert the bytes to a string
            std::str::from_utf8(&self.data[offset..end])
                .map_err(|e| Error::TypeConversion(format!("Invalid UTF-8 string: {e}")))
        }
    }
}
