//! Compression method definitions and flags

/// Compression method flags
pub mod flags {
    /// Huffman encoding (WAVE files only)
    pub const HUFFMAN: u8 = 0x01;
    /// Deflate/zlib compression
    pub const ZLIB: u8 = 0x02;
    /// PKWare Implode compression (older format, appears in newer MPQ v4 archives)
    pub const IMPLODE: u8 = 0x04;
    /// PKWare DCL compression
    pub const PKWARE: u8 = 0x08;
    /// BZip2 compression
    pub const BZIP2: u8 = 0x10;
    /// Sparse/RLE compression
    pub const SPARSE: u8 = 0x20;
    /// IMA ADPCM mono
    pub const ADPCM_MONO: u8 = 0x40;
    /// IMA ADPCM stereo
    pub const ADPCM_STEREO: u8 = 0x80;
    /// LZMA compression (not a flag combination)
    pub const LZMA: u8 = 0x12;
}

/// Compression methods enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMethod {
    /// No compression
    None,
    /// Huffman encoding (WAVE files only)
    Huffman,
    /// Deflate/zlib compression
    Zlib,
    /// PKWare Implode compression
    Implode,
    /// PKWare DCL compression
    PKWare,
    /// BZip2 compression
    BZip2,
    /// Sparse/RLE compression
    Sparse,
    /// IMA ADPCM mono
    AdpcmMono,
    /// IMA ADPCM stereo
    AdpcmStereo,
    /// LZMA compression
    Lzma,
    /// Multiple compression methods applied in sequence
    Multiple(u8),
}

impl CompressionMethod {
    /// Determine compression method(s) from flags
    pub fn from_flags(flags: u8) -> Self {
        // Check for LZMA first (special case, not a bit flag)
        if flags == flags::LZMA {
            return CompressionMethod::Lzma;
        }

        // Check for single compression methods
        match flags {
            0 => CompressionMethod::None,
            flags::HUFFMAN => CompressionMethod::Huffman,
            flags::ZLIB => CompressionMethod::Zlib,
            flags::IMPLODE => CompressionMethod::Implode,
            flags::PKWARE => CompressionMethod::PKWare,
            flags::BZIP2 => CompressionMethod::BZip2,
            flags::SPARSE => CompressionMethod::Sparse,
            flags::ADPCM_MONO => CompressionMethod::AdpcmMono,
            flags::ADPCM_STEREO => CompressionMethod::AdpcmStereo,
            _ => CompressionMethod::Multiple(flags),
        }
    }

    /// Check if this is a multi-compression method
    pub fn is_multiple(&self) -> bool {
        matches!(self, CompressionMethod::Multiple(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_method_from_flags() {
        assert_eq!(CompressionMethod::from_flags(0), CompressionMethod::None);
        assert_eq!(
            CompressionMethod::from_flags(flags::ZLIB),
            CompressionMethod::Zlib
        );
        assert_eq!(
            CompressionMethod::from_flags(flags::BZIP2),
            CompressionMethod::BZip2
        );
        assert_eq!(
            CompressionMethod::from_flags(flags::LZMA),
            CompressionMethod::Lzma
        );

        // Multiple compression
        let multi = flags::ZLIB | flags::PKWARE;
        assert!(CompressionMethod::from_flags(multi).is_multiple());
    }
}
