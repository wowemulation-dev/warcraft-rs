//! Cryptographic operations for MPQ files
//!
//! This module provides all cryptographic functionality needed for MPQ archives:
//!
//! ## Features
//!
//! - **Encryption/Decryption**: Block and single-value encryption using MPQ's custom algorithm
//! - **Hashing**: Multiple hash algorithms for different table types:
//!   - MPQ hash algorithm: Classic hash for hash/block tables and encryption keys
//!   - Jenkins one-at-a-time: Used for BET table hashes in MPQ v3+
//!   - Jenkins hashlittle2: Used for HET table lookups in MPQ v3+
//! - **Digital Signatures**: Support for both weak (512-bit RSA) and strong (2048-bit RSA) signatures
//!
//! ## Digital Signatures
//!
//! MPQ archives support two types of digital signatures:
//!
//! ### Weak Signatures (v1+)
//! - 512-bit RSA with MD5 hash
//! - Stored in `(signature)` file within the archive
//! - Can be generated using the well-known Blizzard private key
//!
//! ### Strong Signatures (v2+)
//! - 2048-bit RSA with SHA-1 hash
//! - Appended after the archive data with "NGIS" header
//! - Generation requires private key (not publicly available)
//!
//! ## Examples
//!
//! ### Hash Algorithms
//!
//! ```no_run
//! use wow_mpq::crypto::{hash_string, hash_type, jenkins_hash, het_hash};
//!
//! // MPQ hash for hash table lookups
//! let table_hash = hash_string("Units\\Human\\Footman.mdx", hash_type::TABLE_OFFSET);
//! let name_a = hash_string("Units\\Human\\Footman.mdx", hash_type::NAME_A);
//! let name_b = hash_string("Units\\Human\\Footman.mdx", hash_type::NAME_B);
//!
//! // Jenkins one-at-a-time for BET tables
//! let bet_hash = jenkins_hash("Units\\Human\\Footman.mdx");
//!
//! // Jenkins hashlittle2 for HET tables
//! let (het_file_hash, het_name_hash) = het_hash("Units\\Human\\Footman.mdx", 48);
//! ```
//!
//! ### Digital Signatures
//!
//! ```no_run
//! use wow_mpq::crypto::{generate_weak_signature, verify_weak_signature_stormlib, SignatureInfo};
//! use std::io::Cursor;
//!
//! # fn main() -> Result<(), wow_mpq::Error> {
//! let archive_data = vec![0u8; 1024];
//! let sig_info = SignatureInfo::new_weak(0, 1024, 1024, 72, vec![]);
//!
//! // Generate signature
//! let signature = generate_weak_signature(Cursor::new(&archive_data), &sig_info)?;
//!
//! // Verify signature
//! let valid = verify_weak_signature_stormlib(
//!     Cursor::new(&archive_data),
//!     &signature[8..72],
//!     &sig_info
//! )?;
//! # Ok(())
//! # }
//! ```

mod decryption;
mod encryption;
mod hash;
mod jenkins;
mod keys;
mod signature;
mod types;

// Re-export public API
pub use decryption::{decrypt_block, decrypt_dword};
pub use encryption::encrypt_block;
pub use hash::hash_string;
pub use jenkins::{jenkins_hashlittle2 as het_hash, jenkins_one_at_a_time as jenkins_hash};
pub use signature::{
    DIGEST_UNIT_SIZE, STRONG_SIGNATURE_HEADER, STRONG_SIGNATURE_SIZE, SignatureInfo, SignatureType,
    StrongSignatureTailType, WEAK_SIGNATURE_FILE_SIZE, WEAK_SIGNATURE_SIZE, calculate_mpq_hash_md5,
    generate_strong_signature, generate_weak_signature, parse_strong_signature,
    parse_weak_signature, public_keys, verify_strong_signature, verify_weak_signature,
    verify_weak_signature_stormlib,
};
pub use types::hash_type;

// Re-export constants that might be needed elsewhere
pub use keys::{ASCII_TO_LOWER, ASCII_TO_UPPER, ENCRYPTION_TABLE};

// Internal-only exports

/// Calculate traditional MPQ hash values for a filename.
///
/// Returns `(hash_a, hash_b, hash_offset)` after normalizing the filename
/// (converting `/` to `\` and uppercasing).
pub fn calculate_mpq_hashes(filename: &str) -> (u32, u32, u32) {
    let normalized = filename.replace('/', "\\").to_uppercase();
    let hash_a = hash_string(&normalized, hash_type::NAME_A);
    let hash_b = hash_string(&normalized, hash_type::NAME_B);
    let hash_offset = hash_string(&normalized, hash_type::TABLE_OFFSET);
    (hash_a, hash_b, hash_offset)
}

/// Calculate HET (Hash Extended Table) hash values for a filename.
///
/// Returns `(file_hash, name_hash)` used by HET tables. The filename is
/// normalized (converting `/` to `\`) but not uppercased, as HET hashes
/// are case-sensitive.
pub fn calculate_het_hashes(filename: &str, hash_bits: u8) -> (u64, u64) {
    let normalized = filename.replace('/', "\\");
    let (file_hash, name_hash_u8) = het_hash(&normalized, hash_bits as u32);
    (file_hash, name_hash_u8 as u64)
}
