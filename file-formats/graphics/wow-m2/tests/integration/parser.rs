//! Integration tests for M2 model parsing

use wow_m2::version::M2Version;

#[test]
fn test_version_values() {
    // Test version enum discriminant values (0, 1, 2, 3...)
    assert_eq!(M2Version::Vanilla as u32, 0);
    assert_eq!(M2Version::TBC as u32, 1);
    assert_eq!(M2Version::WotLK as u32, 2);
    assert_eq!(M2Version::Cataclysm as u32, 3);
}

#[test]
fn test_version_ordering() {
    assert!(M2Version::Vanilla < M2Version::TBC);
    assert!(M2Version::TBC < M2Version::WotLK);
    assert!(M2Version::WotLK < M2Version::Cataclysm);
}

#[test]
fn test_version_display() {
    assert!(M2Version::Vanilla.to_string().contains("Vanilla"));
    assert!(M2Version::TBC.to_string().contains("Burning Crusade"));
    assert!(M2Version::WotLK.to_string().contains("Wrath"));
    assert!(M2Version::Cataclysm.to_string().contains("Cataclysm"));
}

#[test]
fn test_header_version_conversion() {
    // Test actual M2 file format version numbers
    assert_eq!(M2Version::Vanilla.to_header_version(), 256);
    assert_eq!(M2Version::TBC.to_header_version(), 260);
    assert_eq!(M2Version::WotLK.to_header_version(), 264);
    assert_eq!(M2Version::Cataclysm.to_header_version(), 272);
}
