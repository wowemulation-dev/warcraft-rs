//! MPQ test archive creation utilities
//!
//! Replaces the functionality of mpq_tools.py

use crate::{ArchiveBuilder, FormatVersion, compression};
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for creating test MPQ archives
#[derive(Debug, Clone)]
pub struct TestArchiveConfig {
    /// Name of the archive (used for filename)
    pub name: String,
    /// MPQ format version to use
    pub version: FormatVersion,
    /// List of files to include in the archive
    pub files: Vec<TestFile>,
    /// Hash table size (if None, automatically determined)
    pub hash_table_size: Option<u32>,
    /// Block size shift value (sector size = 512 << block_size)
    pub block_size: Option<u8>,
    /// Whether to manually include a (listfile) entry
    pub include_listfile: bool,
    /// Whether to include an (attributes) file
    pub include_attributes: bool,
}

/// A file to include in the test archive
#[derive(Debug, Clone)]
pub struct TestFile {
    /// Path/name of the file within the archive
    pub name: String,
    /// File content data
    pub data: Vec<u8>,
    /// Compression method flags (None for no compression)
    pub compression: Option<u8>,
    /// Whether the file should be encrypted
    pub encrypted: bool,
    /// Whether to use FIX_KEY encryption mode
    pub fix_key: bool,
}

/// Type of test archive to create
#[derive(Debug, Clone, Copy)]
pub enum TestArchiveType {
    /// Minimal archive with single file
    Minimal,
    /// Archive with compressed files
    Compressed,
    /// Archive with encrypted files
    Encrypted,
    /// Archive with various edge cases
    EdgeCases,
    /// Comprehensive test archive
    Comprehensive,
    /// Archive with CRC verification
    WithCrc,
}

impl TestArchiveConfig {
    /// Create a minimal test archive configuration
    pub fn minimal(version: FormatVersion) -> Self {
        Self {
            name: format!("minimal_v{}", version as u8 + 1),
            version,
            files: vec![TestFile {
                name: "test.txt".to_string(),
                data: b"Hello, MPQ!".to_vec(),
                compression: None,
                encrypted: false,
                fix_key: false,
            }],
            hash_table_size: Some(16),
            block_size: Some(3),
            include_listfile: version == FormatVersion::V1,
            include_attributes: false,
        }
    }

    /// Create a compressed files test archive
    pub fn compressed(compression_type: &str) -> Self {
        let data = generate_compressible_data(50 * 1024); // 50KB

        let compression_flag = match compression_type {
            "zlib" => Some(compression::flags::ZLIB),
            "bzip2" => Some(compression::flags::BZIP2),
            "lzma" => Some(compression::flags::LZMA),
            "sparse" => Some(compression::flags::SPARSE),
            _ => None,
        };

        Self {
            name: format!("compressed_{}", compression_type),
            version: FormatVersion::V2,
            files: vec![
                TestFile {
                    name: "compressed.dat".to_string(),
                    data: data.clone(),
                    compression: compression_flag,
                    encrypted: false,
                    fix_key: false,
                },
                TestFile {
                    name: "uncompressed.dat".to_string(),
                    data: data[..1024].to_vec(),
                    compression: None,
                    encrypted: false,
                    fix_key: false,
                },
            ],
            hash_table_size: Some(32),
            block_size: Some(4),
            include_listfile: false,
            include_attributes: false,
        }
    }

    /// Create an encrypted files test archive
    pub fn encrypted() -> Self {
        Self {
            name: "encrypted".to_string(),
            version: FormatVersion::V2,
            files: vec![
                TestFile {
                    name: "secret.dat".to_string(),
                    data: b"This is encrypted data!".to_vec(),
                    compression: Some(compression::flags::ZLIB),
                    encrypted: true,
                    fix_key: false,
                },
                TestFile {
                    name: "fixed_key.dat".to_string(),
                    data: b"This uses fix key encryption!".to_vec(),
                    compression: None,
                    encrypted: true,
                    fix_key: true,
                },
            ],
            hash_table_size: Some(16),
            block_size: Some(3),
            include_listfile: false,
            include_attributes: false,
        }
    }

    /// Create an edge cases test archive
    pub fn edge_cases() -> Self {
        Self {
            name: "edge_cases".to_string(),
            version: FormatVersion::V2,
            files: vec![
                // Empty file
                TestFile {
                    name: "empty.txt".to_string(),
                    data: vec![],
                    compression: None,
                    encrypted: false,
                    fix_key: false,
                },
                // Single byte file
                TestFile {
                    name: "single_byte.dat".to_string(),
                    data: vec![0x42],
                    compression: Some(compression::flags::ZLIB),
                    encrypted: false,
                    fix_key: false,
                },
                // File with spaces in name
                TestFile {
                    name: "file with spaces.txt".to_string(),
                    data: b"Spaces in filename!".to_vec(),
                    compression: None,
                    encrypted: false,
                    fix_key: false,
                },
                // File with path
                TestFile {
                    name: "folder/subfolder/nested.dat".to_string(),
                    data: b"Nested file".to_vec(),
                    compression: None,
                    encrypted: false,
                    fix_key: false,
                },
                // Large uncompressible file
                TestFile {
                    name: "random.bin".to_string(),
                    data: generate_random_data(100 * 1024), // 100KB
                    compression: Some(compression::flags::ZLIB),
                    encrypted: false,
                    fix_key: false,
                },
            ],
            hash_table_size: Some(64),
            block_size: Some(5),
            include_listfile: true,
            include_attributes: false,
        }
    }

    /// Create a comprehensive test archive
    pub fn comprehensive(version: FormatVersion) -> Self {
        let mut files = vec![
            TestFile {
                name: "readme.txt".to_string(),
                data: b"MPQ Archive Test Suite\n\nThis archive contains various test files."
                    .to_vec(),
                compression: None,
                encrypted: false,
                fix_key: false,
            },
            TestFile {
                name: "data/config.ini".to_string(),
                data: b"[Settings]\nversion=1.0\ntest=true".to_vec(),
                compression: Some(compression::flags::ZLIB),
                encrypted: false,
                fix_key: false,
            },
            TestFile {
                name: "data/binary.dat".to_string(),
                data: generate_binary_pattern(10 * 1024),
                compression: Some(compression::flags::BZIP2),
                encrypted: false,
                fix_key: false,
            },
            TestFile {
                name: "secure/encrypted.bin".to_string(),
                data: b"Secret data".to_vec(),
                compression: None,
                encrypted: true,
                fix_key: false,
            },
        ];

        // Add version-specific features
        if version >= FormatVersion::V2 {
            files.push(TestFile {
                name: "large/bigfile.dat".to_string(),
                data: generate_compressible_data(1024 * 1024), // 1MB
                compression: Some(compression::flags::LZMA),
                encrypted: false,
                fix_key: false,
            });
        }

        Self {
            name: format!("comprehensive_v{}", version as u8 + 1),
            version,
            files,
            hash_table_size: Some(128),
            block_size: Some(7), // 64KB sectors
            include_listfile: true,
            include_attributes: version >= FormatVersion::V2,
        }
    }

    /// Create test archive with CRC verification
    pub fn with_crc() -> Self {
        Self {
            name: "crc_test".to_string(),
            version: FormatVersion::V2,
            files: vec![TestFile {
                name: "crc_protected.dat".to_string(),
                data: b"This file has CRC protection".to_vec(),
                compression: Some(compression::flags::ZLIB),
                encrypted: false,
                fix_key: false,
            }],
            hash_table_size: Some(16),
            block_size: Some(3),
            include_listfile: false,
            include_attributes: false,
        }
    }
}

/// Create a test MPQ archive
pub fn create_test_archive(
    output_path: &Path,
    config: &TestArchiveConfig,
) -> Result<PathBuf, crate::Error> {
    let mut builder = if config.include_listfile {
        // If we're manually including a listfile, don't auto-generate
        ArchiveBuilder::new().listfile_option(crate::ListfileOption::None)
    } else {
        // Otherwise, auto-generate
        ArchiveBuilder::new().listfile_option(crate::ListfileOption::Generate)
    };

    // Set version
    builder = builder.version(config.version);

    // Set block size if specified
    if let Some(block_size) = config.block_size {
        builder = builder.block_size(block_size.into());
    }

    // Note: hash_table_size is automatically determined by the builder

    // Add files
    for file in &config.files {
        if file.encrypted {
            builder = builder.add_file_data_with_encryption(
                file.data.clone(),
                &file.name,
                file.compression.unwrap_or(0),
                file.fix_key,
                0, // locale
            );
        } else if let Some(compression) = file.compression {
            builder = builder.add_file_data_with_options(
                file.data.clone(),
                &file.name,
                compression,
                false, // encrypt
                0,     // locale
            );
        } else {
            builder = builder.add_file_data(file.data.clone(), &file.name);
        }
    }

    // Add (listfile) if requested
    if config.include_listfile {
        let listfile_content = config
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        builder = builder.add_file_data(listfile_content.into_bytes(), "(listfile)");
    }

    // Add (attributes) if requested
    if config.include_attributes {
        let attributes = generate_attributes(&config.files);
        builder = builder.add_file_data(attributes, "(attributes)");
    }

    // Build the archive
    let archive_path = output_path.join(&config.name).with_extension("mpq");
    builder.build(&archive_path)?;

    Ok(archive_path)
}

/// Generate compressible test data
fn generate_compressible_data(size: usize) -> Vec<u8> {
    let pattern = b"This is test data that should compress well because it has repeated patterns. ";
    let mut data = Vec::with_capacity(size);

    while data.len() < size {
        let remaining = size - data.len();
        let to_copy = remaining.min(pattern.len());
        data.extend_from_slice(&pattern[..to_copy]);
    }

    data
}

/// Generate random uncompressible data
fn generate_random_data(size: usize) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(42);
    let mut data = vec![0u8; size];
    rng.fill(&mut data[..]);
    data
}

/// Generate binary pattern data
fn generate_binary_pattern(size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let mut value = 0u8;

    while data.len() < size {
        data.push(value);
        value = value.wrapping_add(1);
    }

    data
}

/// Generate attributes file content
fn generate_attributes(files: &[TestFile]) -> Vec<u8> {
    // Simple attributes format: CRC32 and timestamps
    let mut attributes = Vec::new();

    // Version
    attributes.extend_from_slice(&100u32.to_le_bytes());

    // Flags (CRC32 + TIMESTAMP)
    attributes.extend_from_slice(&0x03u32.to_le_bytes());

    // For each file: CRC32 and timestamp
    for file in files {
        // CRC32 (simplified - just use data length as fake CRC)
        attributes.extend_from_slice(&(file.data.len() as u32).to_le_bytes());
        // Timestamp (fake)
        attributes.extend_from_slice(&0x5F000000u32.to_le_bytes());
    }

    attributes
}

/// Create all test archive types
pub fn create_all_test_archives(output_dir: &Path) -> Result<Vec<PathBuf>, crate::Error> {
    fs::create_dir_all(output_dir)?;
    let mut created = Vec::new();

    // Create minimal archives for each version
    for version in [
        FormatVersion::V1,
        FormatVersion::V2,
        FormatVersion::V3,
        FormatVersion::V4,
    ] {
        let config = TestArchiveConfig::minimal(version);
        let path = create_test_archive(output_dir, &config)?;
        created.push(path);
    }

    // Create compressed archives
    for compression in ["zlib", "bzip2", "lzma", "sparse"] {
        let config = TestArchiveConfig::compressed(compression);
        let path = create_test_archive(output_dir, &config)?;
        created.push(path);
    }

    // Create other test types
    let configs = vec![
        TestArchiveConfig::encrypted(),
        TestArchiveConfig::edge_cases(),
        TestArchiveConfig::comprehensive(FormatVersion::V2),
        TestArchiveConfig::comprehensive(FormatVersion::V4),
        TestArchiveConfig::with_crc(),
    ];

    for config in configs {
        let path = create_test_archive(output_dir, &config)?;
        created.push(path);
    }

    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_minimal_archive_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = TestArchiveConfig::minimal(FormatVersion::V1);
        let result = create_test_archive(temp_dir.path(), &config).unwrap();

        assert!(result.exists());
        assert!(
            result
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("minimal")
        );
    }

    #[test]
    fn test_compressed_archive_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = TestArchiveConfig::compressed("zlib");
        let result = create_test_archive(temp_dir.path(), &config).unwrap();

        assert!(result.exists());
    }
}
