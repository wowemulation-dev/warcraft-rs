//! PTCH format header parsing

use crate::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;

/// Magic signature for PTCH header
const PTCH_SIGNATURE: u32 = 0x48435450; // 'PTCH'

/// Magic signature for MD5 block
const MD5_SIGNATURE: u32 = 0x5f35444d; // 'MD5_'

/// Magic signature for XFRM block
const XFRM_SIGNATURE: u32 = 0x4d524658; // 'XFRM'

/// Patch file type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchType {
    /// Simple file replacement - patch data is the complete new file
    Copy,
    /// Binary diff using bsdiff40 algorithm
    Bsd0,
}

impl PatchType {
    /// Parse patch type from u32 magic value
    fn from_magic(magic: u32) -> Result<Self> {
        match magic {
            0x59504f43 => Ok(PatchType::Copy), // 'COPY'
            0x30445342 => Ok(PatchType::Bsd0), // 'BSD0'
            _ => Err(Error::invalid_format(format!(
                "Unknown patch type: 0x{magic:08X}"
            ))),
        }
    }

    /// Get magic value for this patch type
    pub fn to_magic(self) -> u32 {
        match self {
            PatchType::Copy => 0x59504f43,
            PatchType::Bsd0 => 0x30445342,
        }
    }
}

/// PTCH file header containing all metadata
#[derive(Debug, Clone)]
pub struct PatchHeader {
    /// Total size of patch data (decompressed)
    pub patch_data_size: u32,
    /// Size of original file before patching
    pub size_before: u32,
    /// Size of file after patching
    pub size_after: u32,
    /// MD5 hash of original file (before patch)
    pub md5_before: [u8; 16],
    /// MD5 hash of patched file (after patch)
    pub md5_after: [u8; 16],
    /// Type of patch (COPY or BSD0)
    pub patch_type: PatchType,
    /// Size of XFRM block data (excludes 12-byte XFRM header)
    pub xfrm_data_size: u32,
}

impl PatchHeader {
    /// Parse PTCH header from reader
    ///
    /// Reads and validates:
    /// - PTCH header (12 bytes)
    /// - MD5 block (40 bytes)
    /// - XFRM header (12 bytes)
    ///
    /// Returns the parsed header. Caller must read remaining patch data.
    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        // --- PTCH Header (12 bytes) ---
        let ptch_sig = reader.read_u32::<LittleEndian>()?;
        if ptch_sig != PTCH_SIGNATURE {
            return Err(Error::invalid_format(format!(
                "Invalid PTCH signature: expected 0x{PTCH_SIGNATURE:08X}, got 0x{ptch_sig:08X}"
            )));
        }

        let patch_data_size = reader.read_u32::<LittleEndian>()?;
        let size_before = reader.read_u32::<LittleEndian>()?;
        let size_after = reader.read_u32::<LittleEndian>()?;

        log::debug!(
            "PTCH header: patch_size={patch_data_size}, before={size_before}, after={size_after}"
        );

        // --- MD5 Block (40 bytes) ---
        let md5_sig = reader.read_u32::<LittleEndian>()?;
        if md5_sig != MD5_SIGNATURE {
            return Err(Error::invalid_format(format!(
                "Invalid MD5 signature: expected 0x{MD5_SIGNATURE:08X}, got 0x{md5_sig:08X}"
            )));
        }

        let md5_block_size = reader.read_u32::<LittleEndian>()?;
        if md5_block_size != 40 {
            return Err(Error::invalid_format(format!(
                "Invalid MD5 block size: expected 40, got {md5_block_size}"
            )));
        }

        let mut md5_before = [0u8; 16];
        reader.read_exact(&mut md5_before)?;

        let mut md5_after = [0u8; 16];
        reader.read_exact(&mut md5_after)?;

        log::debug!(
            "MD5 before: {}, after: {}",
            hex::encode(md5_before),
            hex::encode(md5_after)
        );

        // --- XFRM Block Header (12 bytes) ---
        let xfrm_sig = reader.read_u32::<LittleEndian>()?;
        if xfrm_sig != XFRM_SIGNATURE {
            return Err(Error::invalid_format(format!(
                "Invalid XFRM signature: expected 0x{XFRM_SIGNATURE:08X}, got 0x{xfrm_sig:08X}"
            )));
        }

        let xfrm_block_size = reader.read_u32::<LittleEndian>()?;
        let patch_type_magic = reader.read_u32::<LittleEndian>()?;

        let patch_type = PatchType::from_magic(patch_type_magic)?;

        // XFRM block size includes the 12-byte header
        let xfrm_data_size = xfrm_block_size.saturating_sub(12);

        log::debug!(
            "XFRM header: type={:?}, block_size={xfrm_block_size}, data_size={xfrm_data_size}",
            patch_type
        );

        Ok(PatchHeader {
            patch_data_size,
            size_before,
            size_after,
            md5_before,
            md5_after,
            patch_type,
            xfrm_data_size,
        })
    }

    /// Total header size in bytes (PTCH + MD5 + XFRM headers)
    pub const HEADER_SIZE: usize = 12 + 40 + 12; // 64 bytes
}

/// Complete patch file with header and data
#[derive(Debug, Clone)]
pub struct PatchFile {
    /// Parsed header
    pub header: PatchHeader,
    /// Raw patch data (after XFRM header, decompressed if needed)
    pub data: Vec<u8>,
}

impl PatchFile {
    /// Parse patch file from complete data
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < PatchHeader::HEADER_SIZE {
            return Err(Error::invalid_format(format!(
                "Patch file too small: {} bytes, need at least {}",
                data.len(),
                PatchHeader::HEADER_SIZE
            )));
        }

        let mut reader = std::io::Cursor::new(data);
        let header = PatchHeader::parse(&mut reader)?;

        // Read remaining data as patch payload
        let mut patch_data = Vec::new();
        reader.read_to_end(&mut patch_data)?;

        log::debug!(
            "Parsed patch file: type={:?}, data_size={}",
            header.patch_type,
            patch_data.len()
        );

        Ok(PatchFile {
            header,
            data: patch_data,
        })
    }

    /// Verify if base file MD5 matches expected
    pub fn verify_base(&self, base_data: &[u8]) -> Result<()> {
        use md5::{Digest, Md5};

        let mut hasher = Md5::new();
        hasher.update(base_data);
        let result = hasher.finalize();

        if result.as_slice() != self.header.md5_before {
            return Err(Error::invalid_format(format!(
                "Base file MD5 mismatch: expected {}, got {}",
                hex::encode(self.header.md5_before),
                hex::encode(result)
            )));
        }

        Ok(())
    }

    /// Verify if patched result MD5 matches expected
    pub fn verify_patched(&self, patched_data: &[u8]) -> Result<()> {
        use md5::{Digest, Md5};

        let mut hasher = Md5::new();
        hasher.update(patched_data);
        let result = hasher.finalize();

        if result.as_slice() != self.header.md5_after {
            return Err(Error::invalid_format(format!(
                "Patched file MD5 mismatch: expected {}, got {}",
                hex::encode(self.header.md5_after),
                hex::encode(result)
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_type_magic() {
        assert_eq!(PatchType::Copy.to_magic(), 0x59504f43);
        assert_eq!(PatchType::Bsd0.to_magic(), 0x30445342);

        assert_eq!(PatchType::from_magic(0x59504f43).unwrap(), PatchType::Copy);
        assert_eq!(PatchType::from_magic(0x30445342).unwrap(), PatchType::Bsd0);

        assert!(PatchType::from_magic(0xDEADBEEF).is_err());
    }

    #[test]
    fn test_header_size() {
        assert_eq!(PatchHeader::HEADER_SIZE, 64);
    }
}
