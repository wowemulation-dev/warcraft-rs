//! Support for the MPQ (attributes) special file.
//!
//! The (attributes) file stores extended metadata about files in the archive,
//! including CRC32 checksums, MD5 hashes, file timestamps, and patch information.

use crate::error::{Error, Result};
use bytes::Bytes;
use std::io::Cursor;

/// Flags indicating which attributes are present in the file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeFlags(u32);

impl AttributeFlags {
    /// CRC32 checksums are present
    pub const CRC32: u32 = 0x00000001;
    /// File timestamps are present
    pub const FILETIME: u32 = 0x00000002;
    /// MD5 hashes are present
    pub const MD5: u32 = 0x00000004;
    /// Patch bit indicators are present
    pub const PATCH_BIT: u32 = 0x00000008;
    /// All attributes are present
    pub const ALL: u32 = 0x0000000F;

    /// Create new attribute flags
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Check if CRC32 checksums are present
    pub fn has_crc32(&self) -> bool {
        self.0 & Self::CRC32 != 0
    }

    /// Check if file timestamps are present
    pub fn has_filetime(&self) -> bool {
        self.0 & Self::FILETIME != 0
    }

    /// Check if MD5 hashes are present
    pub fn has_md5(&self) -> bool {
        self.0 & Self::MD5 != 0
    }

    /// Check if patch bits are present
    pub fn has_patch_bit(&self) -> bool {
        self.0 & Self::PATCH_BIT != 0
    }

    /// Get the raw flags value
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// File attributes for a single file in the archive
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileAttributes {
    /// CRC32 checksum of the uncompressed file data
    pub crc32: Option<u32>,
    /// Windows FILETIME timestamp (100-nanosecond intervals since 1601-01-01)
    pub filetime: Option<u64>,
    /// MD5 hash of the uncompressed file data
    pub md5: Option<[u8; 16]>,
    /// Whether this file is a patch file
    pub is_patch: Option<bool>,
}

impl FileAttributes {
    /// Create empty attributes
    pub fn new() -> Self {
        Self {
            crc32: None,
            filetime: None,
            md5: None,
            is_patch: None,
        }
    }
}

impl Default for FileAttributes {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed (attributes) file data
#[derive(Debug, Clone)]
pub struct Attributes {
    /// Version of the attributes file (should be 100)
    pub version: u32,
    /// Flags indicating which attributes are present
    pub flags: AttributeFlags,
    /// Attributes for each file in the block table
    pub file_attributes: Vec<FileAttributes>,
}

impl Attributes {
    /// Expected version for the attributes file
    pub const EXPECTED_VERSION: u32 = 100;

    /// Parse attributes from raw data
    pub fn parse(data: &Bytes, block_count: usize) -> Result<Self> {
        if data.len() < 8 {
            return Err(Error::invalid_format(
                "Attributes file too small for header",
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read header
        let version = read_u32_le(&mut cursor)?;
        if version != Self::EXPECTED_VERSION {
            return Err(Error::invalid_format(format!(
                "Unsupported attributes version: {} (expected {})",
                version,
                Self::EXPECTED_VERSION
            )));
        }

        let flags = AttributeFlags::new(read_u32_le(&mut cursor)?);

        // Calculate expected size
        let mut expected_size = 8; // header
        if flags.has_crc32() {
            expected_size += block_count * 4;
        }
        if flags.has_filetime() {
            expected_size += block_count * 8;
        }
        if flags.has_md5() {
            expected_size += block_count * 16;
        }
        if flags.has_patch_bit() {
            expected_size += block_count.div_ceil(8);
        }

        if data.len() < expected_size {
            return Err(Error::invalid_format(format!(
                "Attributes file too small: {} bytes (expected at least {})",
                data.len(),
                expected_size
            )));
        }

        // Parse attributes for each file
        let mut file_attributes = Vec::with_capacity(block_count);

        // Parse CRC32 array if present
        let crc32_values = if flags.has_crc32() {
            let mut values = Vec::with_capacity(block_count);
            for _ in 0..block_count {
                values.push(read_u32_le(&mut cursor)?);
            }
            Some(values)
        } else {
            None
        };

        // Parse timestamp array if present
        let filetime_values = if flags.has_filetime() {
            let mut values = Vec::with_capacity(block_count);
            for _ in 0..block_count {
                values.push(read_u64_le(&mut cursor)?);
            }
            Some(values)
        } else {
            None
        };

        // Parse MD5 array if present
        let md5_values = if flags.has_md5() {
            let mut values = Vec::with_capacity(block_count);
            for _ in 0..block_count {
                let mut hash = [0u8; 16];
                read_exact(&mut cursor, &mut hash)?;
                values.push(hash);
            }
            Some(values)
        } else {
            None
        };

        // Parse patch bits if present
        let patch_bits = if flags.has_patch_bit() {
            let byte_count = block_count.div_ceil(8);
            let mut bits = vec![0u8; byte_count];
            read_exact(&mut cursor, &mut bits)?;
            Some(bits)
        } else {
            None
        };

        // Combine into FileAttributes structs
        for i in 0..block_count {
            let mut attrs = FileAttributes::new();

            if let Some(ref values) = crc32_values {
                attrs.crc32 = Some(values[i]);
            }

            if let Some(ref values) = filetime_values {
                attrs.filetime = Some(values[i]);
            }

            if let Some(ref values) = md5_values {
                attrs.md5 = Some(values[i]);
            }

            if let Some(ref bits) = patch_bits {
                let byte_index = i / 8;
                let bit_index = i % 8;
                attrs.is_patch = Some((bits[byte_index] & (1 << bit_index)) != 0);
            }

            file_attributes.push(attrs);
        }

        Ok(Self {
            version,
            flags,
            file_attributes,
        })
    }

    /// Get attributes for a specific block index
    pub fn get_file_attributes(&self, block_index: usize) -> Option<&FileAttributes> {
        self.file_attributes.get(block_index)
    }

    /// Create attributes data for writing to an archive
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let block_count = self.file_attributes.len();
        let mut data = Vec::new();

        // Write header
        data.extend_from_slice(&self.version.to_le_bytes());
        data.extend_from_slice(&self.flags.as_u32().to_le_bytes());

        // Write CRC32 array if present
        if self.flags.has_crc32() {
            for attrs in &self.file_attributes {
                let crc = attrs.crc32.unwrap_or(0);
                data.extend_from_slice(&crc.to_le_bytes());
            }
        }

        // Write timestamp array if present
        if self.flags.has_filetime() {
            for attrs in &self.file_attributes {
                let time = attrs.filetime.unwrap_or(0);
                data.extend_from_slice(&time.to_le_bytes());
            }
        }

        // Write MD5 array if present
        if self.flags.has_md5() {
            for attrs in &self.file_attributes {
                let hash = attrs.md5.unwrap_or([0u8; 16]);
                data.extend_from_slice(&hash);
            }
        }

        // Write patch bits if present
        if self.flags.has_patch_bit() {
            let byte_count = block_count.div_ceil(8);
            let mut bits = vec![0u8; byte_count];

            for (i, attrs) in self.file_attributes.iter().enumerate() {
                if attrs.is_patch.unwrap_or(false) {
                    let byte_index = i / 8;
                    let bit_index = i % 8;
                    bits[byte_index] |= 1 << bit_index;
                }
            }

            data.extend_from_slice(&bits);
        }

        Ok(data)
    }
}

// Helper functions for reading from cursor
fn read_u32_le(cursor: &mut Cursor<&Bytes>) -> Result<u32> {
    use std::io::Read;
    let mut bytes = [0u8; 4];
    cursor.read_exact(&mut bytes).map_err(Error::Io)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64_le(cursor: &mut Cursor<&Bytes>) -> Result<u64> {
    use std::io::Read;
    let mut bytes = [0u8; 8];
    cursor.read_exact(&mut bytes).map_err(Error::Io)?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_exact(cursor: &mut Cursor<&Bytes>, buf: &mut [u8]) -> Result<()> {
    use std::io::Read;
    cursor.read_exact(buf).map_err(Error::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_flags() {
        let flags = AttributeFlags::new(AttributeFlags::ALL);
        assert!(flags.has_crc32());
        assert!(flags.has_filetime());
        assert!(flags.has_md5());
        assert!(flags.has_patch_bit());

        let flags = AttributeFlags::new(AttributeFlags::CRC32 | AttributeFlags::MD5);
        assert!(flags.has_crc32());
        assert!(!flags.has_filetime());
        assert!(flags.has_md5());
        assert!(!flags.has_patch_bit());
    }

    #[test]
    fn test_parse_empty_attributes() {
        // Create minimal attributes file with no attributes
        let mut data = Vec::new();
        data.extend_from_slice(&100u32.to_le_bytes()); // version
        data.extend_from_slice(&0u32.to_le_bytes()); // flags (no attributes)

        let bytes = Bytes::from(data);
        let attrs = Attributes::parse(&bytes, 0).unwrap();

        assert_eq!(attrs.version, 100);
        assert_eq!(attrs.flags.as_u32(), 0);
        assert_eq!(attrs.file_attributes.len(), 0);
    }

    #[test]
    fn test_parse_crc32_only() {
        let mut data = Vec::new();
        data.extend_from_slice(&100u32.to_le_bytes()); // version
        data.extend_from_slice(&AttributeFlags::CRC32.to_le_bytes()); // flags

        // Add CRC32 values for 2 files
        data.extend_from_slice(&0x12345678u32.to_le_bytes());
        data.extend_from_slice(&0x9ABCDEF0u32.to_le_bytes());

        let bytes = Bytes::from(data);
        let attrs = Attributes::parse(&bytes, 2).unwrap();

        assert_eq!(attrs.version, 100);
        assert!(attrs.flags.has_crc32());
        assert!(!attrs.flags.has_filetime());
        assert!(!attrs.flags.has_md5());
        assert!(!attrs.flags.has_patch_bit());

        assert_eq!(attrs.file_attributes.len(), 2);
        assert_eq!(attrs.file_attributes[0].crc32, Some(0x12345678));
        assert_eq!(attrs.file_attributes[1].crc32, Some(0x9ABCDEF0));
    }

    #[test]
    fn test_roundtrip() {
        // Create attributes with all fields
        let mut file_attrs = Vec::new();

        let mut attr1 = FileAttributes::new();
        attr1.crc32 = Some(0x12345678);
        attr1.filetime = Some(0x01234567_89ABCDEF);
        attr1.md5 = Some([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        attr1.is_patch = Some(false);
        file_attrs.push(attr1);

        let mut attr2 = FileAttributes::new();
        attr2.crc32 = Some(0x9ABCDEF0);
        attr2.filetime = Some(0xFEDCBA98_76543210);
        attr2.md5 = Some([16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
        attr2.is_patch = Some(true);
        file_attrs.push(attr2);

        let original = Attributes {
            version: 100,
            flags: AttributeFlags::new(AttributeFlags::ALL),
            file_attributes: file_attrs,
        };

        // Convert to bytes and back
        let bytes = original.to_bytes().unwrap();
        let parsed = Attributes::parse(&Bytes::from(bytes), 2).unwrap();

        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.flags.as_u32(), original.flags.as_u32());
        assert_eq!(parsed.file_attributes.len(), original.file_attributes.len());

        for i in 0..2 {
            assert_eq!(parsed.file_attributes[i], original.file_attributes[i]);
        }
    }
}
