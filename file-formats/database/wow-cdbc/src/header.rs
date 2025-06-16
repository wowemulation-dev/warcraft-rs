//! DBC header structure and parsing functionality.

use crate::{Error, Result};
use std::io::{Read, Seek, SeekFrom};

/// The magic signature at the beginning of a DBC file
pub const DBC_MAGIC: [u8; 4] = *b"WDBC";

/// Represents a DBC file header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DbcHeader {
    /// The magic signature, should be "WDBC"
    pub magic: [u8; 4],
    /// Number of records in the file
    pub record_count: u32,
    /// Number of fields in each record
    pub field_count: u32,
    /// Size of each record in bytes
    pub record_size: u32,
    /// Size of the string block in bytes
    pub string_block_size: u32,
}

impl DbcHeader {
    /// The size of a DBC header in bytes
    pub const SIZE: usize = 20;

    /// Parse a DBC header from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Ensure we're at the beginning of the file
        reader.seek(SeekFrom::Start(0))?;

        // Read the magic signature
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;

        // Validate the magic signature
        if magic != DBC_MAGIC {
            return Err(Error::InvalidHeader(format!(
                "Invalid magic signature: {:?}, expected: {:?}",
                magic, DBC_MAGIC
            )));
        }

        // Read the rest of the header
        let mut buf = [0u8; 4];

        reader.read_exact(&mut buf)?;
        let record_count = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let field_count = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let record_size = u32::from_le_bytes(buf);

        reader.read_exact(&mut buf)?;
        let string_block_size = u32::from_le_bytes(buf);

        // Perform basic validation
        if record_size == 0 && record_count > 0 {
            return Err(Error::InvalidHeader(
                "Record size cannot be 0 if record count is greater than 0".to_string(),
            ));
        }

        if field_count == 0 && record_count > 0 {
            return Err(Error::InvalidHeader(
                "Field count cannot be 0 if record count is greater than 0".to_string(),
            ));
        }

        Ok(Self {
            magic,
            record_count,
            field_count,
            record_size,
            string_block_size,
        })
    }

    /// Calculates the offset to the string block
    pub fn string_block_offset(&self) -> u64 {
        DbcHeader::SIZE as u64 + (self.record_count as u64 * self.record_size as u64)
    }

    /// Calculates the total size of the DBC file
    pub fn total_size(&self) -> u64 {
        self.string_block_offset() + self.string_block_size as u64
    }
}
