//! Database functionality for storing and retrieving MPQ filename-to-hash mappings
//!
//! This module provides a persistent SQLite database that records relationships between
//! filenames and their MPQ hash values, enabling file resolution even when archives
//! lack internal listfiles.

mod connection;
mod import;
mod lookup;
mod models;
mod schema;

pub use connection::{Database, DatabaseError};
pub use import::{ImportSource, Importer};
pub use lookup::{HashLookup, HetHashLookup};
pub use models::{ArchiveRecord, FileRecord, HashType};

use crate::crypto::{hash_string, hash_type, het_hash};

/// Calculate traditional MPQ hash values for a filename
pub fn calculate_mpq_hashes(filename: &str) -> (u32, u32, u32) {
    let normalized = filename.replace('/', "\\").to_uppercase();
    let hash_a = hash_string(&normalized, hash_type::NAME_A);
    let hash_b = hash_string(&normalized, hash_type::NAME_B);
    let hash_offset = hash_string(&normalized, hash_type::TABLE_OFFSET);
    (hash_a, hash_b, hash_offset)
}

/// Calculate HET (Hash Extended Table) hash values for a filename
/// Returns (file_hash, name_hash) used by HET tables
pub fn calculate_het_hashes(filename: &str, hash_bits: u8) -> (u64, u64) {
    let normalized = filename.replace('/', "\\");
    let (file_hash, name_hash_u8) = het_hash(&normalized, hash_bits as u32);
    (file_hash, name_hash_u8 as u64)
}
