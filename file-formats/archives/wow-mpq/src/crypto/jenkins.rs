//! Jenkins hash algorithms for HET and BET tables
//!
//! This module contains both Jenkins hash algorithms used in MPQ v3+ archives:
//! - Jenkins one-at-a-time: Used for BET table hashes
//! - Jenkins hashlittle2: Used for HET table hashes

/// Jenkins one-at-a-time hash function for BET tables
///
/// This is the original Jenkins one-at-a-time algorithm used by BET tables
/// in MPQ v3+ archives. It produces a 64-bit hash value from the input filename.
pub fn jenkins_one_at_a_time(filename: &str) -> u64 {
    let mut hash: u64 = 0;

    for &byte in filename.as_bytes() {
        // Get the next character and normalize it
        let mut ch = byte;

        // Convert path separators to backslash
        if ch == b'/' {
            ch = b'\\';
        }

        // Convert to lowercase using the table
        ch = super::keys::ASCII_TO_LOWER[ch as usize];

        // Jenkins one-at-a-time hash algorithm
        hash = hash.wrapping_add(ch as u64);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }

    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);

    hash
}

/// Jenkins hashlittle2 hash function (internal implementation)
///
/// This is the exact implementation used by StormLib for HET table hashing.
/// It produces two 32-bit hash values from the input data.
fn hashlittle2(key: &[u8], pc: &mut u32, pb: &mut u32) {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let length = key.len();
    let mut k = key;

    // Set up the internal state
    a = 0xdeadbeef_u32.wrapping_add(length as u32).wrapping_add(*pc);
    b = a;
    c = a.wrapping_add(*pb);

    // Process all but the last block
    while k.len() > 12 {
        a = a.wrapping_add(u32::from_le_bytes([k[0], k[1], k[2], k[3]]));
        b = b.wrapping_add(u32::from_le_bytes([k[4], k[5], k[6], k[7]]));
        c = c.wrapping_add(u32::from_le_bytes([k[8], k[9], k[10], k[11]]));

        // mix macro
        a = a.wrapping_sub(c);
        a ^= c.rotate_left(4);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= a.rotate_left(6);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= b.rotate_left(8);
        b = b.wrapping_add(a);
        a = a.wrapping_sub(c);
        a ^= c.rotate_left(16);
        c = c.wrapping_add(b);
        b = b.wrapping_sub(a);
        b ^= a.rotate_left(19);
        a = a.wrapping_add(c);
        c = c.wrapping_sub(b);
        c ^= b.rotate_left(4);
        b = b.wrapping_add(a);

        k = &k[12..];
    }

    // Process the last block
    let remaining = k.len();
    if remaining > 0 {
        // Handle remaining bytes
        let mut last_block = [0u8; 12];
        last_block[..remaining].copy_from_slice(&k[..remaining]);

        match remaining {
            12 => {
                c = c.wrapping_add(u32::from_le_bytes([
                    last_block[8],
                    last_block[9],
                    last_block[10],
                    last_block[11],
                ]));
                b = b.wrapping_add(u32::from_le_bytes([
                    last_block[4],
                    last_block[5],
                    last_block[6],
                    last_block[7],
                ]));
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            11 => {
                c = c.wrapping_add((last_block[10] as u32) << 16);
                c = c.wrapping_add(u32::from_le_bytes([last_block[8], last_block[9], 0, 0]));
                b = b.wrapping_add(u32::from_le_bytes([
                    last_block[4],
                    last_block[5],
                    last_block[6],
                    last_block[7],
                ]));
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            10 => {
                c = c.wrapping_add(u32::from_le_bytes([last_block[8], last_block[9], 0, 0]));
                b = b.wrapping_add(u32::from_le_bytes([
                    last_block[4],
                    last_block[5],
                    last_block[6],
                    last_block[7],
                ]));
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            9 => {
                c = c.wrapping_add(last_block[8] as u32);
                b = b.wrapping_add(u32::from_le_bytes([
                    last_block[4],
                    last_block[5],
                    last_block[6],
                    last_block[7],
                ]));
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            8 => {
                b = b.wrapping_add(u32::from_le_bytes([
                    last_block[4],
                    last_block[5],
                    last_block[6],
                    last_block[7],
                ]));
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            7 => {
                b = b.wrapping_add((last_block[6] as u32) << 16);
                b = b.wrapping_add(u32::from_le_bytes([last_block[4], last_block[5], 0, 0]));
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            6 => {
                b = b.wrapping_add(u32::from_le_bytes([last_block[4], last_block[5], 0, 0]));
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            5 => {
                b = b.wrapping_add(last_block[4] as u32);
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            4 => {
                a = a.wrapping_add(u32::from_le_bytes([
                    last_block[0],
                    last_block[1],
                    last_block[2],
                    last_block[3],
                ]));
            }
            3 => {
                a = a.wrapping_add((last_block[2] as u32) << 16);
                a = a.wrapping_add(u32::from_le_bytes([last_block[0], last_block[1], 0, 0]));
            }
            2 => {
                a = a.wrapping_add(u32::from_le_bytes([last_block[0], last_block[1], 0, 0]));
            }
            1 => {
                a = a.wrapping_add(last_block[0] as u32);
            }
            _ => {}
        }
    }

    if !k.is_empty() {
        // final macro
        c ^= b;
        c = c.wrapping_sub(b.rotate_left(14));
        a ^= c;
        a = a.wrapping_sub(c.rotate_left(11));
        b ^= a;
        b = b.wrapping_sub(a.rotate_left(25));
        c ^= b;
        c = c.wrapping_sub(b.rotate_left(16));
        a ^= c;
        a = a.wrapping_sub(c.rotate_left(4));
        b ^= a;
        b = b.wrapping_sub(a.rotate_left(14));
        c ^= b;
        c = c.wrapping_sub(b.rotate_left(24));
    }

    *pc = c;
    *pb = b;
}

/// Calculate HET table hash for a filename using Jenkins hashlittle2
///
/// This function normalizes the filename and applies Jenkins hashlittle2
/// to generate the hash value used in HET tables.
pub fn jenkins_hashlittle2(filename: &str, hash_bits: u32) -> (u64, u8) {
    // Normalize filename: lowercase and convert / to \
    let normalized = filename
        .bytes()
        .map(|b| {
            let b = if b == b'/' { b'\\' } else { b };
            // Simple ASCII lowercase
            if b.is_ascii_uppercase() { b + 32 } else { b }
        })
        .collect::<Vec<u8>>();

    // Initial seeds
    let mut primary = 1u32;
    let mut secondary = 2u32;

    // Apply hashlittle2
    hashlittle2(&normalized, &mut secondary, &mut primary);

    // Combine into 64-bit hash
    let full_hash = ((primary as u64) << 32) | (secondary as u64);

    // Calculate masks
    let (and_mask, or_mask) = if hash_bits < 64 {
        let and_mask = (1u64 << hash_bits) - 1;
        let or_mask = 1u64 << (hash_bits - 1);
        (and_mask, or_mask)
    } else {
        (0xFFFFFFFFFFFFFFFF, 0)
    };

    // Apply masks
    let file_name_hash = (full_hash & and_mask) | or_mask;

    // Extract NameHash1
    let name_hash1 = if hash_bits < 64 {
        ((file_name_hash >> (hash_bits - 8)) & 0xFF) as u8
    } else {
        ((file_name_hash >> 56) & 0xFF) as u8
    };

    (file_name_hash, name_hash1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jenkins_one_at_a_time() {
        // Test case insensitivity
        let hash1 = jenkins_one_at_a_time("File.txt");
        let hash2 = jenkins_one_at_a_time("FILE.TXT");
        assert_eq!(
            hash1, hash2,
            "Jenkins one-at-a-time should be case-insensitive"
        );

        // Test path normalization
        let hash1 = jenkins_one_at_a_time("path/to/file");
        let hash2 = jenkins_one_at_a_time("path\\to\\file");
        assert_eq!(hash1, hash2, "Path separators should be normalized");

        // Test that different files produce different hashes
        let hash1 = jenkins_one_at_a_time("file1.txt");
        let hash2 = jenkins_one_at_a_time("file2.txt");
        assert_ne!(
            hash1, hash2,
            "Different files should produce different hashes"
        );
    }

    #[test]
    fn test_jenkins_hashlittle2_attributes() {
        // Test with 48-bit hash (which should produce 0xE9 for "(attributes)")
        let (hash, name_hash1) = jenkins_hashlittle2("(attributes)", 48);

        println!("HET hash for '(attributes)' with 48-bit hash:");
        println!("  Full hash: 0x{hash:016X}");
        println!("  NameHash1: 0x{name_hash1:02X}");

        assert_eq!(
            name_hash1, 0xE9,
            "NameHash1 for '(attributes)' with 48-bit hash should be 0xE9"
        );
    }

    #[test]
    fn test_jenkins_hashlittle2_normalization() {
        // Test that forward slashes are converted to backslashes
        let (hash1, _) = jenkins_hashlittle2("path/to/file", 48);
        let (hash2, _) = jenkins_hashlittle2("path\\to\\file", 48);
        assert_eq!(hash1, hash2, "Path separators should be normalized");

        // Test case insensitivity
        let (hash1, _) = jenkins_hashlittle2("File.txt", 48);
        let (hash2, _) = jenkins_hashlittle2("FILE.TXT", 48);
        assert_eq!(hash1, hash2, "Filenames should be case-insensitive");
    }
}
