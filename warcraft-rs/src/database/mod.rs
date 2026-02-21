//! Database functionality for storing and retrieving MPQ filename-to-hash mappings
//!
//! This module provides a persistent SQLite database (via turso) that records
//! relationships between filenames and their MPQ hash values, enabling file
//! resolution even when archives lack internal listfiles.

mod connection;
mod import;
mod lookup;
mod models;
mod schema;

pub use connection::{Database, DatabaseError};
pub use import::{ImportSource, Importer};
#[allow(unused_imports)]
pub use lookup::{HashLookup, HetHashLookup};
#[allow(unused_imports)]
pub use models::{ArchiveRecord, FileRecord, HashType};

// Re-export hash computation functions from wow-mpq
pub use wow_mpq::{calculate_het_hashes, calculate_mpq_hashes};
