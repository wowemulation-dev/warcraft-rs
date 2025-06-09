//! Cryptographic operations for MPQ files
//!
//! This module provides all cryptographic functionality needed for MPQ archives:
//!
//! ## Features
//!
//! - **Encryption/Decryption**: Block and single-value encryption using MPQ's custom algorithm
//! - **Hashing**: String hashing for file name lookup in hash tables
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
//! ## Example
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
mod keys;
mod signature;
mod types;

// Re-export public API
pub use decryption::{decrypt_block, decrypt_dword};
pub use encryption::encrypt_block;
pub use hash::{hash_string, jenkins_hash};
pub use signature::{
    DIGEST_UNIT_SIZE, STRONG_SIGNATURE_HEADER, STRONG_SIGNATURE_SIZE, SignatureInfo, SignatureType,
    StrongSignatureTailType, WEAK_SIGNATURE_FILE_SIZE, WEAK_SIGNATURE_SIZE, calculate_mpq_hash_md5,
    generate_strong_signature, generate_weak_signature, parse_strong_signature,
    parse_weak_signature, public_keys, verify_strong_signature, verify_weak_signature,
    verify_weak_signature_stormlib,
};
pub use types::hash_type;

// Re-export constants that might be needed elsewhere
pub use keys::ENCRYPTION_TABLE;

// Internal-only exports
