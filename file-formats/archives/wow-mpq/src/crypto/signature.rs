//! MPQ digital signature support
//!
//! MPQ archives can contain digital signatures to verify their integrity:
//! - Weak signatures (v1+): 512-bit RSA with MD5, stored in (signature) file
//! - Strong signatures (v2+): 2048-bit RSA with SHA-1, appended after archive

use crate::{Error, Result};
use md5::{Digest, Md5};
use num_bigint::BigUint;
use num_traits::Num;
use rsa::traits::PublicKeyParts;
use rsa::{BigUint as RsaBigUint, RsaPublicKey};
use sha1::Sha1;
use std::io::Read;

/// Weak signature size (512-bit RSA)
pub const WEAK_SIGNATURE_SIZE: usize = 64; // 512 bits / 8

/// Weak signature file size (signature + 8 byte header)
pub const WEAK_SIGNATURE_FILE_SIZE: usize = WEAK_SIGNATURE_SIZE + 8; // 72 bytes total

/// Hash digest unit size for chunked processing (matches StormLib)
pub const DIGEST_UNIT_SIZE: usize = 0x10000; // 65536 bytes

/// Strong signature header
pub const STRONG_SIGNATURE_HEADER: [u8; 4] = *b"NGIS"; // "SIGN" reversed

/// Strong signature size (2048-bit RSA + 4 byte header)
pub const STRONG_SIGNATURE_SIZE: usize = 256 + 4; // 2048 bits / 8 + header

/// Signature type in the archive
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignatureType {
    /// Weak signature (512-bit RSA with MD5)
    Weak,
    /// Strong signature (2048-bit RSA with SHA-1)
    Strong,
}

/// Signature information for hash calculation (matches StormLib's MPQ_SIGNATURE_INFO)
#[derive(Debug, Clone)]
pub struct SignatureInfo {
    /// File offset where the hashing starts (archive beginning)
    pub begin_mpq_data: u64,
    /// Begin of the excluded area (signature file position)  
    pub begin_exclude: u64,
    /// End of the excluded area (signature file position + size)
    pub end_exclude: u64,
    /// File offset where the hashing ends (archive end)
    pub end_mpq_data: u64,
    /// Size of the entire file
    pub end_of_file: u64,
    /// The signature data
    pub signature: Vec<u8>,
    /// Length of the signature
    pub signature_size: u32,
    /// Signature types present
    pub signature_types: u32,
}

impl SignatureInfo {
    /// Create a new SignatureInfo for weak signature verification
    pub fn new_weak(
        archive_start: u64,
        archive_size: u64,
        signature_file_pos: u64,
        signature_file_size: u64,
        signature_data: Vec<u8>,
    ) -> Self {
        let signature_size = signature_data.len() as u32;
        Self {
            begin_mpq_data: archive_start,
            begin_exclude: signature_file_pos,
            end_exclude: signature_file_pos + signature_file_size,
            end_mpq_data: archive_start + archive_size,
            end_of_file: archive_start + archive_size, // For weak signatures, no data after archive
            signature: signature_data,
            signature_size,
            signature_types: 1, // SIGNATURE_TYPE_WEAK = 1
        }
    }
}

/// Blizzard public keys for signature verification
pub mod public_keys {
    use super::*;

    /// Blizzard weak signature public key (512-bit)
    /// This is the well-known public key used for weak signatures
    pub const BLIZZARD_WEAK_PUBLIC_KEY_N: &str = "92627704BFB882CC0523B90CB1AC0459272175968D025EDA47DD7C49371BF8FAEB0E0A92167557AD51B78CCB68C5426290EE9FB14BC118E430349EA4ED6AD837";

    /// Weak signature public exponent
    pub const BLIZZARD_WEAK_PUBLIC_KEY_E: u32 = 0x10001; // 65537

    /// Blizzard strong signature public key (2048-bit)
    /// This is the well-known public key used for strong signatures in WoW
    pub const BLIZZARD_STRONG_PUBLIC_KEY_N: &str = "B1067ECE24F687C87E27F88C42981DB47D47689CCE044DDA823538C8C3DCAE2C5A3CE668038B7C6F07DECBBA9CCDF5B2C28718A37A657B2B4517E22E0F81C3165F4E5CDD52172BA94A0331D441999606C50289A76EAF4C409C8CA90B4C8510231608384E7752ED835BF893120042A991736A636F27FC45411C3E53B0CB9508BE7BF6021E9DBAFAD5D23DD830C4772EFDD08CC81B454A58B87F28E4DC4C97E60ECFFB1D04E41A8B955BE594B1F7A4BAA350A3B343F4306784B8CB8E9B71785136019A98700D5AA374BD2CDDC62F5B569555C5217F5CEDF5AA6954D0959DA836C23F011540A4E2B782B360AAFC07E98A156155E3349128E6C409B0FB1D57F86477";

    /// Strong signature public exponent
    pub const BLIZZARD_STRONG_PUBLIC_KEY_E: u32 = 0x10001; // 65537

    /// Get the weak signature public key
    pub fn weak_public_key() -> Result<RsaPublicKey> {
        let n = RsaBigUint::from_str_radix(BLIZZARD_WEAK_PUBLIC_KEY_N, 16)
            .map_err(|e| Error::invalid_format(format!("Invalid weak key modulus: {}", e)))?;
        let e = RsaBigUint::from(BLIZZARD_WEAK_PUBLIC_KEY_E);

        RsaPublicKey::new(n, e)
            .map_err(|e| Error::invalid_format(format!("Invalid weak public key: {}", e)))
    }

    /// Get the strong signature public key
    pub fn strong_public_key() -> Result<RsaPublicKey> {
        let n = RsaBigUint::from_str_radix(BLIZZARD_STRONG_PUBLIC_KEY_N, 16)
            .map_err(|e| Error::invalid_format(format!("Invalid strong key modulus: {}", e)))?;
        let e = RsaBigUint::from(BLIZZARD_STRONG_PUBLIC_KEY_E);

        RsaPublicKey::new(n, e)
            .map_err(|e| Error::invalid_format(format!("Invalid strong public key: {}", e)))
    }
}

/// Parse weak signature from (signature) file data
/// StormLib stores the actual signature at offset 8 in the 72-byte file
pub fn parse_weak_signature(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < WEAK_SIGNATURE_FILE_SIZE {
        return Err(Error::invalid_format(format!(
            "Weak signature file too small: {} bytes, expected {}",
            data.len(),
            WEAK_SIGNATURE_FILE_SIZE
        )));
    }

    // Extract the signature data from offset 8 (skip 8-byte header)
    // StormLib reads from &pSI->Signature[8] with MPQ_WEAK_SIGNATURE_SIZE bytes
    let signature = data[8..8 + WEAK_SIGNATURE_SIZE].to_vec();

    // Check for zero signature (like StormLib's IsValidSignature check)
    if signature.iter().all(|&b| b == 0) {
        return Err(Error::invalid_format("Signature is all zeros (invalid)"));
    }

    Ok(signature)
}

/// Parse strong signature from data after the archive
pub fn parse_strong_signature(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < STRONG_SIGNATURE_SIZE {
        return Err(Error::invalid_format(format!(
            "Strong signature too small: {} bytes, expected {}",
            data.len(),
            STRONG_SIGNATURE_SIZE
        )));
    }

    // Check the "NGIS" header
    if data[0..4] != STRONG_SIGNATURE_HEADER {
        return Err(Error::invalid_format(format!(
            "Invalid strong signature header: expected {:?}, got {:?}",
            STRONG_SIGNATURE_HEADER,
            &data[0..4]
        )));
    }

    // Extract the 256-byte signature data (skip 4-byte header)
    let signature = data[4..4 + 256].to_vec();

    // Check for zero signature (like StormLib's IsValidSignature check)
    if signature.iter().all(|&b| b == 0) {
        return Err(Error::invalid_format("Signature is all zeros (invalid)"));
    }

    Ok(signature)
}

/// Calculate MD5 hash for weak signature verification (matches StormLib's CalculateMpqHashMd5)
/// This implementation exactly matches StormLib's chunk-based approach:
/// - Uses exactly 64KB chunks (DIGEST_UNIT_SIZE = 0x10000)
/// - Zeros out signature area when it overlaps with current chunk
/// - Processes chunks with proper overlap detection for signature boundaries
pub fn calculate_mpq_hash_md5<R: Read + std::io::Seek>(
    mut reader: R,
    signature_info: &SignatureInfo,
) -> Result<[u8; 16]> {
    let mut hasher = Md5::new();
    let mut buffer = vec![0u8; DIGEST_UNIT_SIZE]; // Exactly 64KB chunks like StormLib
    let mut current_pos = signature_info.begin_mpq_data;

    log::debug!(
        "StormLib-compatible hash calculation: archive range 0x{:X}-0x{:X}, exclude range 0x{:X}-0x{:X}",
        signature_info.begin_mpq_data,
        signature_info.end_mpq_data,
        signature_info.begin_exclude,
        signature_info.end_exclude
    );

    // Hash data from BeginMpqData to EndMpqData in 64KB chunks
    while current_pos < signature_info.end_mpq_data {
        let remaining = signature_info.end_mpq_data - current_pos;
        let to_read = (remaining as usize).min(DIGEST_UNIT_SIZE);

        // Seek to current position and read chunk
        reader.seek(std::io::SeekFrom::Start(current_pos))?;
        let bytes_read = reader.read(&mut buffer[..to_read])?;
        if bytes_read == 0 {
            break; // EOF reached
        }

        let chunk_end_pos = current_pos + bytes_read as u64;

        // Check if signature area overlaps with this chunk
        let sig_begin = signature_info.begin_exclude;
        let sig_end = signature_info.end_exclude;

        if current_pos < sig_end && chunk_end_pos > sig_begin {
            // Calculate the overlap of signature area within this chunk
            let chunk_sig_start = if sig_begin > current_pos {
                (sig_begin - current_pos) as usize
            } else {
                0
            };
            let chunk_sig_end = if sig_end < chunk_end_pos {
                (sig_end - current_pos) as usize
            } else {
                bytes_read
            };

            // Zero out the signature area within this chunk (StormLib approach)
            // This matches StormLib's behavior of zeroing signature bytes rather than skipping them
            for byte in buffer[chunk_sig_start..chunk_sig_end].iter_mut() {
                *byte = 0;
            }

            log::debug!(
                "Chunk 0x{:X}-0x{:X}: zeroed signature bytes [{}-{}]",
                current_pos,
                chunk_end_pos,
                chunk_sig_start,
                chunk_sig_end
            );
        }

        // Hash the entire chunk (including any zeroed signature area)
        hasher.update(&buffer[..bytes_read]);
        current_pos += bytes_read as u64;
    }

    let hash = hasher.finalize();
    log::debug!("Final MD5 hash: {:02X?}", hash.as_slice());

    Ok(hash.into())
}

/// Verify a weak signature (512-bit RSA with MD5) using StormLib-compatible approach
pub fn verify_weak_signature_stormlib<R: Read + std::io::Seek>(
    reader: R,
    signature: &[u8],
    signature_info: &SignatureInfo,
) -> Result<bool> {
    // Get the public key
    let public_key = public_keys::weak_public_key()?;

    // Calculate MD5 hash using StormLib's approach
    let hash = calculate_mpq_hash_md5(reader, signature_info)?;

    // Convert signature from little-endian to big-endian (StormLib's memrev operation)
    let signature_be = reverse_bytes(signature);

    // Decrypt the signature using RSA
    let signature_int = BigUint::from_bytes_be(&signature_be);
    let n = BigUint::from_bytes_be(&public_key.n().to_bytes_be());
    let e = BigUint::from_bytes_be(&public_key.e().to_bytes_be());

    // Perform RSA operation: signature^e mod n
    let decrypted = signature_int.modpow(&e, &n);
    let decrypted_bytes = decrypted.to_bytes_be();

    // Verify PKCS#1 v1.5 padding
    verify_pkcs1_v15_md5(&decrypted_bytes, &hash)
}

/// Legacy verify function (kept for backward compatibility)
pub fn verify_weak_signature<R: Read>(
    mut reader: R,
    signature: &[u8],
    archive_size: u64,
) -> Result<bool> {
    // Get the public key
    let public_key = public_keys::weak_public_key()?;

    // Calculate MD5 hash of the archive (excluding the signature)
    let mut hasher = Md5::new();
    let mut buffer = vec![0u8; 65536]; // 64KB buffer
    let mut bytes_read = 0u64;

    // Read up to archive_size (which should exclude the signature)
    while bytes_read < archive_size {
        let to_read = ((archive_size - bytes_read) as usize).min(buffer.len());
        let n = reader.read(&mut buffer[..to_read])?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        bytes_read += n as u64;
    }

    let hash = hasher.finalize();

    // Convert signature from little-endian to big-endian
    let signature_be = reverse_bytes(signature);

    // Decrypt the signature using RSA
    let signature_int = BigUint::from_bytes_be(&signature_be);
    let n = BigUint::from_bytes_be(&public_key.n().to_bytes_be());
    let e = BigUint::from_bytes_be(&public_key.e().to_bytes_be());

    // Perform RSA operation: signature^e mod n
    let decrypted = signature_int.modpow(&e, &n);
    let decrypted_bytes = decrypted.to_bytes_be();

    // Verify PKCS#1 v1.5 padding
    verify_pkcs1_v15_md5(&decrypted_bytes, &hash)
}

/// Verify a strong signature (2048-bit RSA with SHA-1)
pub fn verify_strong_signature<R: Read>(
    mut reader: R,
    signature: &[u8],
    archive_size: u64,
) -> Result<bool> {
    // Get the public key
    let public_key = public_keys::strong_public_key()?;

    // Calculate SHA-1 hash of the archive (excluding the signature)
    let mut hasher = Sha1::new();
    let mut buffer = vec![0u8; 65536]; // 64KB buffer
    let mut bytes_read = 0u64;

    // Read up to archive_size (which should exclude the signature)
    while bytes_read < archive_size {
        let to_read = ((archive_size - bytes_read) as usize).min(buffer.len());
        let n = reader.read(&mut buffer[..to_read])?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        bytes_read += n as u64;
    }

    let hash = hasher.finalize();

    // Convert signature from little-endian to big-endian
    let signature_be = reverse_bytes(signature);

    // Decrypt the signature using RSA
    let signature_int = BigUint::from_bytes_be(&signature_be);
    let n = BigUint::from_bytes_be(&public_key.n().to_bytes_be());
    let e = BigUint::from_bytes_be(&public_key.e().to_bytes_be());

    // Perform RSA operation: signature^e mod n
    let decrypted = signature_int.modpow(&e, &n);
    let decrypted_bytes = decrypted.to_bytes_be();

    // Verify custom MPQ strong signature padding
    verify_mpq_strong_signature_padding(&decrypted_bytes, &hash)
}

/// Verify PKCS#1 v1.5 padding for MD5
fn verify_pkcs1_v15_md5(decrypted: &[u8], expected_hash: &[u8]) -> Result<bool> {
    // PKCS#1 v1.5 structure for MD5:
    // 0x00 || 0x01 || PS || 0x00 || DigestInfo
    // Where PS is padding bytes (0xFF) and DigestInfo contains the hash

    if decrypted.len() < 11 + 16 + 18 {
        // Minimum size for PKCS#1 padding + MD5
        return Ok(false);
    }

    // Check header
    if decrypted[0] != 0x00 || decrypted[1] != 0x01 {
        return Ok(false);
    }

    // Find 0x00 separator after padding
    let mut separator_pos = None;
    for (i, &byte) in decrypted.iter().enumerate().skip(2) {
        if byte == 0x00 {
            separator_pos = Some(i);
            break;
        } else if byte != 0xFF {
            return Ok(false); // Invalid padding byte
        }
    }

    let separator_pos = separator_pos
        .ok_or_else(|| Error::invalid_format("No separator found in PKCS#1 padding"))?;

    // MD5 DigestInfo (from PKCS#1)
    let md5_digest_info = [
        0x30, 0x20, // SEQUENCE, length 32
        0x30, 0x0C, // SEQUENCE, length 12
        0x06, 0x08, // OID, length 8
        0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x02, 0x05, // MD5 OID
        0x05, 0x00, // NULL
        0x04, 0x10, // OCTET STRING, length 16
    ];

    // Check DigestInfo
    let digest_start = separator_pos + 1;
    if digest_start + md5_digest_info.len() + 16 != decrypted.len() {
        return Ok(false);
    }

    if decrypted[digest_start..digest_start + md5_digest_info.len()] != md5_digest_info {
        return Ok(false);
    }

    // Check hash
    let hash_start = digest_start + md5_digest_info.len();
    Ok(&decrypted[hash_start..] == expected_hash)
}

/// Verify MPQ strong signature custom padding format
fn verify_mpq_strong_signature_padding(decrypted: &[u8], expected_hash: &[u8]) -> Result<bool> {
    // MPQ strong signature structure (when decrypted):
    // - 1 byte: padding type (must be 0x0B)
    // - 235 bytes: padding bytes (must all be 0xBB)
    // - 20 bytes: SHA-1 hash

    if decrypted.len() != 256 {
        // Strong signatures are always 2048 bits = 256 bytes when decrypted
        return Ok(false);
    }

    // Check padding type
    if decrypted[0] != 0x0B {
        log::debug!(
            "Invalid padding type: expected 0x0B, got 0x{:02X}",
            decrypted[0]
        );
        return Ok(false);
    }

    // Check padding bytes (235 bytes of 0xBB)
    for (i, &byte) in decrypted.iter().enumerate().take(236).skip(1) {
        if byte != 0xBB {
            log::debug!(
                "Invalid padding byte at position {}: expected 0xBB, got 0x{:02X}",
                i,
                byte
            );
            return Ok(false);
        }
    }

    // Check SHA-1 hash (last 20 bytes)
    let signature_hash = &decrypted[236..256];
    if signature_hash != expected_hash {
        log::debug!("Hash mismatch in strong signature");
        log::debug!("Expected: {:02X?}", expected_hash);
        log::debug!("Got:      {:02X?}", signature_hash);
        return Ok(false);
    }

    Ok(true)
}

/// Reverse byte order (little-endian to big-endian conversion)
fn reverse_bytes(data: &[u8]) -> Vec<u8> {
    data.iter().rev().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_public_key_loading() {
        let key = public_keys::weak_public_key().unwrap();
        // Check modulus size
        let n_bytes = key.n().to_bytes_be();
        assert_eq!(n_bytes.len(), 64); // 512 bits / 8
        assert_eq!(key.e(), &RsaBigUint::from(65537u32));
    }

    #[test]
    fn test_strong_public_key_loading() {
        let key = public_keys::strong_public_key().unwrap();
        // Check modulus size (may be 255 or 256 bytes depending on leading zeros)
        let n_bytes = key.n().to_bytes_be();
        assert!(n_bytes.len() >= 255 && n_bytes.len() <= 256); // 2048 bits / 8
        assert_eq!(key.e(), &RsaBigUint::from(65537u32));
    }

    #[test]
    fn test_reverse_bytes() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let reversed = reverse_bytes(&data);
        assert_eq!(reversed, vec![0x04, 0x03, 0x02, 0x01]);
    }

    #[test]
    fn test_parse_weak_signature() {
        let mut data = vec![0x00; WEAK_SIGNATURE_FILE_SIZE];
        // Fill signature area (bytes 8-71) with non-zero data
        for item in data.iter_mut().skip(8).take(WEAK_SIGNATURE_SIZE) {
            *item = 0xFF;
        }

        let sig = parse_weak_signature(&data).unwrap();
        assert_eq!(sig.len(), WEAK_SIGNATURE_SIZE);
        assert_eq!(sig, vec![0xFF; WEAK_SIGNATURE_SIZE]);
    }

    #[test]
    fn test_parse_weak_signature_too_small() {
        let data = vec![0xFF; 32];
        let result = parse_weak_signature(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_weak_signature_zero_signature() {
        let data = vec![0x00; WEAK_SIGNATURE_FILE_SIZE]; // All zeros including signature area
        let result = parse_weak_signature(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("all zeros"));
    }

    #[test]
    fn test_parse_strong_signature() {
        let mut data = vec![0xFF; 300];
        // Set the correct header
        data[0..4].copy_from_slice(&STRONG_SIGNATURE_HEADER);

        let sig = parse_strong_signature(&data).unwrap();
        assert_eq!(sig.len(), 256);
        assert_eq!(sig, &data[4..260]);
    }

    #[test]
    fn test_parse_strong_signature_invalid_header() {
        let mut data = vec![0xFF; 300];
        // Set wrong header
        data[0..4].copy_from_slice(b"WXYZ");

        let result = parse_strong_signature(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_strong_signature_too_small() {
        let data = vec![0xFF; 100];
        let result = parse_strong_signature(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_strong_signature_zero_signature() {
        let mut data = vec![0x00; STRONG_SIGNATURE_SIZE];
        // Set the correct header but keep signature as zeros
        data[0..4].copy_from_slice(&STRONG_SIGNATURE_HEADER);

        let result = parse_strong_signature(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("all zeros"));
    }

    #[test]
    fn test_verify_mpq_strong_signature_padding() {
        // Create a properly formatted strong signature
        let mut decrypted = vec![0; 256];
        decrypted[0] = 0x0B; // Padding type
        for byte in decrypted.iter_mut().take(236).skip(1) {
            *byte = 0xBB; // Padding bytes
        }

        // Add test SHA-1 hash
        let test_hash = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67,
        ];
        decrypted[236..256].copy_from_slice(&test_hash);

        let result = verify_mpq_strong_signature_padding(&decrypted, &test_hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_mpq_strong_signature_padding_wrong_type() {
        let mut decrypted = vec![0; 256];
        decrypted[0] = 0x01; // Wrong padding type
        for byte in decrypted.iter_mut().take(236).skip(1) {
            *byte = 0xBB;
        }

        let test_hash = [0; 20];
        decrypted[236..256].copy_from_slice(&test_hash);

        let result = verify_mpq_strong_signature_padding(&decrypted, &test_hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_verify_mpq_strong_signature_padding_wrong_padding() {
        let mut decrypted = vec![0; 256];
        decrypted[0] = 0x0B;
        for byte in decrypted.iter_mut().take(236).skip(1) {
            *byte = 0xCC; // Wrong padding bytes
        }

        let test_hash = [0; 20];
        decrypted[236..256].copy_from_slice(&test_hash);

        let result = verify_mpq_strong_signature_padding(&decrypted, &test_hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_verify_mpq_strong_signature_padding_wrong_hash() {
        let mut decrypted = vec![0; 256];
        decrypted[0] = 0x0B;
        for byte in decrypted.iter_mut().take(236).skip(1) {
            *byte = 0xBB;
        }

        let wrong_hash = [0xFF; 20];
        decrypted[236..256].copy_from_slice(&wrong_hash);

        let correct_hash = [0x00; 20];
        let result = verify_mpq_strong_signature_padding(&decrypted, &correct_hash).unwrap();
        assert!(!result);
    }
}
