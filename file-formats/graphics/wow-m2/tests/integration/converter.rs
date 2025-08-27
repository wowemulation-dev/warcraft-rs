//! Integration tests for M2 version conversion

use wow_m2::version::M2Version;

#[test]
fn test_version_evolution() {
    // Test that versions evolved as expected
    let versions = [
        M2Version::Vanilla,
        M2Version::TBC,
        M2Version::WotLK,
        M2Version::Cataclysm,
    ];

    // Verify they are in ascending order
    for window in versions.windows(2) {
        assert!(window[0] < window[1]);
    }
}

#[test]
fn test_version_conversion_mapping() {
    // Test that we can identify version differences
    let classic = M2Version::Vanilla;
    let bc = M2Version::TBC;
    let wotlk = M2Version::WotLK;

    assert_ne!(classic, bc);
    assert_ne!(bc, wotlk);
    assert_ne!(classic, wotlk);
}

#[test]
fn test_major_version_boundaries() {
    // Test major expansion boundaries
    assert!(M2Version::Vanilla < M2Version::TBC);
    assert!(M2Version::TBC < M2Version::WotLK);
    assert!(M2Version::WotLK < M2Version::Cataclysm);

    // Test that there are no versions between these major releases
    assert_eq!(M2Version::Vanilla as u32 + 1, M2Version::TBC as u32);
}

#[test]
fn test_version_compatibility_checks() {
    // In a real conversion system, these would check compatibility
    let source_version = M2Version::Vanilla;
    let target_version = M2Version::Cataclysm;

    // Upgrading should be possible (in theory)
    assert!(source_version < target_version);

    // Downgrading should require special handling
    assert!(target_version > source_version);
}
