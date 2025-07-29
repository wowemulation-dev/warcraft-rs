//! Database models and data structures

use chrono::{DateTime, Utc};

/// Type of hash used in the archive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    /// Traditional MPQ hash table
    Traditional,
    /// HET (Hash Extended Table) with specified bit size
    Het(u8),
}

/// Represents a filename and all its associated hashes
#[derive(Debug, Clone)]
pub struct FileRecord {
    /// Unique database identifier
    pub id: Option<i64>,
    /// The filename as stored in the archive
    pub filename: String,
    /// Traditional MPQ hash A value
    pub hash_a: u32,
    /// Traditional MPQ hash B value
    pub hash_b: u32,
    /// Traditional MPQ hash offset value
    pub hash_offset: u32,
    /// HET 40-bit hash pair (file_hash, name_hash)
    pub het_hash_40: Option<(u64, u64)>,
    /// HET 48-bit hash pair (file_hash, name_hash)
    pub het_hash_48: Option<(u64, u64)>,
    /// HET 56-bit hash pair (file_hash, name_hash)
    pub het_hash_56: Option<(u64, u64)>,
    /// HET 64-bit hash pair (file_hash, name_hash)
    pub het_hash_64: Option<(u64, u64)>,
    /// Source of the filename (e.g., archive path, listfile)
    pub source: Option<String>,
    /// Timestamp when the record was created
    pub created_at: DateTime<Utc>,
}

/// Represents an analyzed MPQ archive
#[derive(Debug, Clone)]
pub struct ArchiveRecord {
    /// Unique database identifier
    pub id: Option<i64>,
    /// Path to the MPQ archive file
    pub archive_path: String,
    /// Hash of the archive file for integrity checking
    pub archive_hash: Option<String>,
    /// Timestamp when the archive was analyzed
    pub analysis_date: DateTime<Utc>,
    /// MPQ format version (1, 2, 3, or 4)
    pub mpq_version: Option<u32>,
    /// Number of files in the archive
    pub file_count: Option<u32>,
}

/// Represents a file found in an archive (unused, for future functionality)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(super) struct ArchiveFileRecord {
    pub archive_id: i64,
    pub hash_a: u32,
    pub hash_b: u32,
    pub file_size: Option<u64>,
    pub compressed_size: Option<u64>,
    pub flags: Option<u32>,
    pub filename_id: Option<i64>,
}

/// Statistics about the database (unused, for future functionality)
#[allow(dead_code)]
#[derive(Debug)]
pub(super) struct DatabaseStats {
    pub total_filenames: usize,
    pub total_archives: usize,
    pub total_archive_files: usize,
    pub filenames_by_source: Vec<(String, usize)>,
}
