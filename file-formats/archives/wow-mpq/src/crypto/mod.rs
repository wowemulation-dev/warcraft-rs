//! Cryptographic operations for MPQ files

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
    WEAK_SIGNATURE_FILE_SIZE, WEAK_SIGNATURE_SIZE, calculate_mpq_hash_md5, parse_strong_signature,
    parse_weak_signature, public_keys, verify_strong_signature, verify_weak_signature,
    verify_weak_signature_stormlib,
};
pub use types::hash_type;

// Re-export constants that might be needed elsewhere
pub use keys::ENCRYPTION_TABLE;

// Internal-only exports
