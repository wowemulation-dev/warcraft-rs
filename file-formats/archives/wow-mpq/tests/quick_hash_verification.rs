//! Quick verification test for hash function reorganization

use wow_mpq::crypto::{hash_string, hash_type, het_hash, jenkins_hash};

#[test]
fn test_hash_functions_after_reorganization() {
    // Test MPQ hash function
    let listfile_hash = hash_string("(listfile)", hash_type::TABLE_OFFSET);
    assert_eq!(listfile_hash, 0x5F3DE859);
    
    // Test Jenkins one-at-a-time (BET tables)
    let jenkins = jenkins_hash("(listfile)");
    assert_ne!(jenkins, 0); // Should be non-zero
    
    // Test Jenkins hashlittle2 (HET tables) - known result for "(attributes)"
    let (_het_hash_val, name_hash1) = het_hash("(attributes)", 48);
    assert_eq!(name_hash1, 0xE9, "NameHash1 for '(attributes)' with 48-bit hash should be 0xE9");
    
    // Test path normalization consistency
    let path1 = hash_string("path/to/file", hash_type::TABLE_OFFSET);
    let path2 = hash_string("path\\to\\file", hash_type::TABLE_OFFSET);
    assert_eq!(path1, path2, "Path normalization should work for MPQ hash");
    
    let jenkins1 = jenkins_hash("path/to/file");
    let jenkins2 = jenkins_hash("path\\to\\file");
    assert_eq!(jenkins1, jenkins2, "Path normalization should work for Jenkins one-at-a-time");
    
    let (het1, _) = het_hash("path/to/file", 48);
    let (het2, _) = het_hash("path\\to\\file", 48);
    assert_eq!(het1, het2, "Path normalization should work for Jenkins hashlittle2");
    
    // Test case insensitivity
    let case1 = hash_string("FILE.TXT", hash_type::TABLE_OFFSET);
    let case2 = hash_string("file.txt", hash_type::TABLE_OFFSET);
    assert_eq!(case1, case2, "MPQ hash should be case insensitive");
}