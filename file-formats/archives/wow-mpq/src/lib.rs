//! # wow_mpq - MPQ Archive Library
//!
//! A high-performance, safe Rust implementation of the MPQ (Mo'PaQ) archive format
//! used by Blizzard Entertainment games.
//!
//! ## About the Name
//!
//! wow_mpq is named after the original format name "Mo'PaQ" (Mike O'Brien Pack),
//! which was later shortened to MPQ. This library provides the core MPQ functionality.
//!
//! ## Features
//!
//! - Support for all MPQ format versions (v1-v4)
//! - Full compatibility with StormLib API through FFI
//! - Multiple compression algorithms (zlib, bzip2, LZMA, etc.)
//! - Digital signature support (verification and generation)
//! - Strong security with signature verification
//! - Comprehensive error handling
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```no_run
//! use wow_mpq::{Archive, OpenOptions};
//!
//! # fn main() -> Result<(), wow_mpq::Error> {
//! // Open an existing MPQ archive
//! let mut archive = Archive::open("example.mpq")?;
//!
//! // List files in the archive
//! for entry in archive.list()? {
//!     println!("{}", entry.name);
//! }
//!
//! // Extract a specific file
//! let data = archive.read_file("war3map.j")?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Digital Signatures
//!
//! ```no_run
//! use wow_mpq::crypto::{generate_weak_signature, SignatureInfo, WEAK_SIGNATURE_FILE_SIZE};
//! use std::io::Cursor;
//!
//! # fn main() -> Result<(), wow_mpq::Error> {
//! // Generate a weak signature for an archive
//! let archive_data = std::fs::read("archive.mpq")?;
//! let archive_size = archive_data.len() as u64;
//!
//! let sig_info = SignatureInfo::new_weak(
//!     0,                               // Archive start offset
//!     archive_size,                    // Archive size
//!     archive_size,                    // Signature position (at end)
//!     WEAK_SIGNATURE_FILE_SIZE as u64, // Signature file size
//!     vec![],                          // Empty initially
//! );
//!
//! let cursor = Cursor::new(&archive_data);
//! let signature_file = generate_weak_signature(cursor, &sig_info)?;
//! # Ok(())
//! # }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

pub mod archive;
pub mod builder;
pub mod compare;
pub mod compression;
pub mod crypto;
pub mod error;
pub mod header;
pub mod io;
pub mod modification;
pub mod patch_chain;
pub mod path;
pub mod rebuild;
pub mod special_files;
pub mod tables;

#[cfg(any(test, feature = "test-utils", doc))]
pub mod test_utils;

pub mod debug;

// Re-export commonly used types
pub use archive::{
    Archive, ArchiveInfo, FileEntry, FileInfo, Md5Status, OpenOptions, SignatureStatus, TableInfo,
    UserDataInfo,
};
pub use builder::{ArchiveBuilder, AttributesOption, ListfileOption};
pub use compare::{
    CompareOptions, ComparisonResult, ComparisonSummary, FileComparison, MetadataComparison,
    compare_archives,
};
pub use error::{Error, Result};
pub use header::{FormatVersion, MpqHeader};
pub use modification::{AddFileOptions, MutableArchive};
pub use patch_chain::{ChainInfo, PatchChain};
pub use rebuild::{RebuildOptions, RebuildSummary, rebuild_archive};
pub use tables::{BetFileInfo, BetTable, BlockEntry, BlockTable, HashEntry, HashTable, HetTable};

// Re-export crypto for CLI usage
pub use crypto::{
    decrypt_block, decrypt_dword, encrypt_block, hash_string, hash_type, jenkins_hash,
};

// Re-export compression for testing
pub use compression::{compress, decompress};

// Re-export decryption for testing
pub use archive::decrypt_file_data;

/// MPQ signature constants
pub mod signatures {
    /// Standard MPQ archive signature ('MPQ\x1A')
    pub const MPQ_ARCHIVE: u32 = 0x1A51504D;

    /// MPQ user data signature ('MPQ\x1B')
    pub const MPQ_USERDATA: u32 = 0x1B51504D;

    /// HET table signature ('HET\x1A')
    pub const HET_TABLE: u32 = 0x1A544548;

    /// BET table signature ('BET\x1A')
    pub const BET_TABLE: u32 = 0x1A544542;

    /// Strong signature magic ('NGIS')
    pub const STRONG_SIGNATURE: [u8; 4] = *b"NGIS";
}

/// Block size calculation
#[inline]
pub fn calculate_sector_size(block_size_shift: u16) -> usize {
    512 << block_size_shift
}

/// Check if a value is a power of two
#[inline]
pub fn is_power_of_two(value: u32) -> bool {
    value != 0 && (value & (value - 1)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sector_size() {
        // Test standard sector sizes used in MPQ archives

        // Block size 0: 512 bytes (minimum)
        assert_eq!(calculate_sector_size(0), 512);

        // Block size 1: 1024 bytes (1 KB)
        assert_eq!(calculate_sector_size(1), 1024);

        // Block size 2: 2048 bytes (2 KB)
        assert_eq!(calculate_sector_size(2), 2048);

        // Block size 3: 4096 bytes (4 KB) - Common default
        assert_eq!(calculate_sector_size(3), 4096);

        // Block size 4: 8192 bytes (8 KB)
        assert_eq!(calculate_sector_size(4), 8192);

        // Block size 5: 16384 bytes (16 KB)
        assert_eq!(calculate_sector_size(5), 16384);

        // Block size 6: 32768 bytes (32 KB)
        assert_eq!(calculate_sector_size(6), 32768);

        // Block size 7: 65536 bytes (64 KB)
        assert_eq!(calculate_sector_size(7), 65536);

        // Block size 8: 131072 bytes (128 KB)
        assert_eq!(calculate_sector_size(8), 131072);

        // Block size 9: 262144 bytes (256 KB)
        assert_eq!(calculate_sector_size(9), 262144);

        // Block size 10: 524288 bytes (512 KB)
        assert_eq!(calculate_sector_size(10), 524288);

        // Block size 11: 1048576 bytes (1 MB)
        assert_eq!(calculate_sector_size(11), 1048576);

        // Block size 12: 2097152 bytes (2 MB)
        assert_eq!(calculate_sector_size(12), 2097152);

        // Block size 13: 4194304 bytes (4 MB)
        assert_eq!(calculate_sector_size(13), 4194304);

        // Block size 14: 8388608 bytes (8 MB)
        assert_eq!(calculate_sector_size(14), 8388608);

        // Block size 15: 16777216 bytes (16 MB) - Maximum practical size
        assert_eq!(calculate_sector_size(15), 16777216);
    }

    #[test]
    fn test_calculate_sector_size_edge_cases() {
        // Test with maximum u16 value (though this would be impractical)
        // This would overflow on 32-bit systems, but Rust handles it gracefully
        let max_shift = 16; // Reasonable maximum to test
        let result = calculate_sector_size(max_shift);
        assert_eq!(result, 512 << 16); // 33,554,432 bytes (32 MB)
    }

    #[test]
    fn test_is_power_of_two() {
        // Valid powers of two
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(8));
        assert!(is_power_of_two(16));
        assert!(is_power_of_two(32));
        assert!(is_power_of_two(64));
        assert!(is_power_of_two(128));
        assert!(is_power_of_two(256));
        assert!(is_power_of_two(512));
        assert!(is_power_of_two(1024));
        assert!(is_power_of_two(2048));
        assert!(is_power_of_two(4096));
        assert!(is_power_of_two(8192));
        assert!(is_power_of_two(16384));
        assert!(is_power_of_two(32768));
        assert!(is_power_of_two(65536));
        assert!(is_power_of_two(0x100000)); // 1,048,576
        assert!(is_power_of_two(0x80000000)); // 2^31

        // Not powers of two
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(5));
        assert!(!is_power_of_two(6));
        assert!(!is_power_of_two(7));
        assert!(!is_power_of_two(9));
        assert!(!is_power_of_two(10));
        assert!(!is_power_of_two(15));
        assert!(!is_power_of_two(100));
        assert!(!is_power_of_two(127));
        assert!(!is_power_of_two(255));
        assert!(!is_power_of_two(1023));
        assert!(!is_power_of_two(1025));
        assert!(!is_power_of_two(0xFFFFFFFF));
    }

    #[test]
    fn test_hash_table_size_validation() {
        // Hash table sizes must be powers of two
        // This test demonstrates how is_power_of_two would be used

        let valid_sizes = [
            4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
        ];

        for size in &valid_sizes {
            assert!(
                is_power_of_two(*size),
                "Hash table size {} should be valid",
                size
            );
        }

        let invalid_sizes = [0, 3, 5, 7, 9, 15, 100, 1000, 1023, 1025, 4095, 4097];

        for size in &invalid_sizes {
            assert!(
                !is_power_of_two(*size),
                "Hash table size {} should be invalid",
                size
            );
        }
    }

    #[test]
    fn test_typical_mpq_configurations() {
        // Test typical MPQ configurations from various games

        // Diablo II: Often uses block size 3 (4KB sectors)
        let d2_sector_size = calculate_sector_size(3);
        assert_eq!(d2_sector_size, 4096);

        // Warcraft III: Typically uses block size 3-4 (4KB-8KB sectors)
        let wc3_sector_size_small = calculate_sector_size(3);
        let wc3_sector_size_large = calculate_sector_size(4);
        assert_eq!(wc3_sector_size_small, 4096);
        assert_eq!(wc3_sector_size_large, 8192);

        // World of Warcraft: Uses various sizes, often 4-8 (8KB-128KB sectors)
        let wow_sector_size_min = calculate_sector_size(4);
        let wow_sector_size_typical = calculate_sector_size(6);
        let wow_sector_size_max = calculate_sector_size(8);
        assert_eq!(wow_sector_size_min, 8192);
        assert_eq!(wow_sector_size_typical, 32768);
        assert_eq!(wow_sector_size_max, 131072);

        // StarCraft II: Can use larger sectors for HD assets
        let sc2_sector_size = calculate_sector_size(9);
        assert_eq!(sc2_sector_size, 262144); // 256 KB
    }
}
