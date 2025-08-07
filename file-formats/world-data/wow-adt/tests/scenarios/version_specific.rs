//! Version-specific ADT parsing tests
//!
//! These tests validate parsing behavior for different WoW client versions
//! by testing the version detection logic and public API.

use wow_adt::AdtVersion;

#[test]
fn test_vanilla_chunk_detection() {
    // Vanilla ADT should only have basic chunks, no MFBO, MH2O, etc.
    let version = AdtVersion::detect_from_chunks_extended(
        false, // no MFBO
        false, // no MH2O
        false, // no MTFX
        false, // no MCCV
        false, // no MTXP
        false, // no MAMP
    );

    assert_eq!(version, AdtVersion::Vanilla);
}

#[test]
fn test_tbc_chunk_detection() {
    // TBC introduced MFBO (flight boundaries)
    let version = AdtVersion::detect_from_chunks_extended(
        true,  // has MFBO
        false, // no MH2O
        false, // no MTFX
        false, // no MCCV
        false, // no MTXP
        false, // no MAMP
    );

    assert_eq!(version, AdtVersion::TBC);
}

#[test]
fn test_wotlk_chunk_detection() {
    // WotLK introduced MH2O (water/lava chunks)
    let version = AdtVersion::detect_from_chunks_extended(
        true,  // has MFBO
        true,  // has MH2O
        false, // no MTFX
        false, // no MCCV
        false, // no MTXP
        false, // no MAMP
    );

    assert_eq!(version, AdtVersion::WotLK);
}

#[test]
fn test_cataclysm_chunk_detection() {
    // Cataclysm introduced MAMP (texture amplifiers)
    let version = AdtVersion::detect_from_chunks_extended(
        true,  // has MFBO
        true,  // has MH2O
        false, // no MTFX
        false, // no MCCV
        false, // no MTXP
        true,  // has MAMP
    );

    assert_eq!(version, AdtVersion::Cataclysm);
}

#[test]
fn test_mop_chunk_detection() {
    // MoP introduced MTXP (texture parameters)
    let version = AdtVersion::detect_from_chunks_extended(
        true,  // has MFBO
        true,  // has MH2O
        false, // no MTFX
        false, // no MCCV
        true,  // has MTXP
        true,  // has MAMP
    );

    assert_eq!(version, AdtVersion::MoP);
}

#[test]
fn test_version_specific_chunk_combinations() {
    // Test that different chunk combinations correctly detect versions

    // Vanilla: minimal chunks
    let vanilla = AdtVersion::detect_from_chunks_extended(false, false, false, false, false, false);
    assert_eq!(vanilla, AdtVersion::Vanilla);

    // TBC: adds MFBO
    let tbc = AdtVersion::detect_from_chunks_extended(true, false, false, false, false, false);
    assert_eq!(tbc, AdtVersion::TBC);

    // WotLK: adds MH2O
    let wotlk = AdtVersion::detect_from_chunks_extended(true, true, false, false, false, false);
    assert_eq!(wotlk, AdtVersion::WotLK);

    // Cataclysm: adds MAMP
    let cata = AdtVersion::detect_from_chunks_extended(true, true, false, false, false, true);
    assert_eq!(cata, AdtVersion::Cataclysm);

    // MoP: adds MTXP
    let mop = AdtVersion::detect_from_chunks_extended(true, true, false, false, true, true);
    assert_eq!(mop, AdtVersion::MoP);
}

#[test]
fn test_version_ordering() {
    // Test that versions can be compared and ordered correctly
    assert!(AdtVersion::Vanilla < AdtVersion::TBC);
    assert!(AdtVersion::TBC < AdtVersion::WotLK);
    assert!(AdtVersion::WotLK < AdtVersion::Cataclysm);
    assert!(AdtVersion::Cataclysm < AdtVersion::MoP);
    assert!(AdtVersion::MoP < AdtVersion::WoD);
    assert!(AdtVersion::WoD < AdtVersion::Legion);
    assert!(AdtVersion::Legion < AdtVersion::BfA);
    assert!(AdtVersion::BfA < AdtVersion::Shadowlands);
    assert!(AdtVersion::Shadowlands < AdtVersion::Dragonflight);
}

#[test]
fn test_version_display() {
    // Test that versions display correctly
    assert_eq!(AdtVersion::Vanilla.to_string(), "Vanilla (1.x)");
    assert_eq!(AdtVersion::TBC.to_string(), "The Burning Crusade (2.x)");
    assert_eq!(
        AdtVersion::WotLK.to_string(),
        "Wrath of the Lich King (3.x)"
    );
    assert_eq!(AdtVersion::Cataclysm.to_string(), "Cataclysm (4.x)");
    assert_eq!(AdtVersion::MoP.to_string(), "Mists of Pandaria (5.x)");
}

#[test]
fn test_version_mver_value() {
    // All ADT versions should use MVER value 18
    assert_eq!(AdtVersion::Vanilla.to_mver_value(), 18);
    assert_eq!(AdtVersion::TBC.to_mver_value(), 18);
    assert_eq!(AdtVersion::WotLK.to_mver_value(), 18);
    assert_eq!(AdtVersion::Cataclysm.to_mver_value(), 18);
    assert_eq!(AdtVersion::MoP.to_mver_value(), 18);
}

#[test]
fn test_version_from_mver() {
    // Test creating version from MVER value
    let version = AdtVersion::from_mver(18).unwrap();

    // Since MVER value 18 is used for all versions, it defaults to Vanilla
    assert_eq!(version, AdtVersion::Vanilla);

    // Invalid MVER values should return error
    assert!(AdtVersion::from_mver(42).is_err());
    assert!(AdtVersion::from_mver(0).is_err());
}

#[test]
fn test_chunk_progression() {
    // Test that each version adds new capabilities

    // Vanilla has basic chunks only
    let vanilla_chunks = (false, false, false, false, false, false);
    let vanilla = AdtVersion::detect_from_chunks_extended(
        vanilla_chunks.0,
        vanilla_chunks.1,
        vanilla_chunks.2,
        vanilla_chunks.3,
        vanilla_chunks.4,
        vanilla_chunks.5,
    );
    assert_eq!(vanilla, AdtVersion::Vanilla);

    // Each successive version should add features
    assert!(AdtVersion::TBC > AdtVersion::Vanilla);
    assert!(AdtVersion::WotLK > AdtVersion::TBC);
    assert!(AdtVersion::Cataclysm > AdtVersion::WotLK);
    assert!(AdtVersion::MoP > AdtVersion::Cataclysm);
}

#[test]
fn test_version_capabilities() {
    // Test that we can distinguish between key version capabilities

    // Flight boundaries were added in TBC
    let pre_flight =
        AdtVersion::detect_from_chunks_extended(false, false, false, false, false, false);
    let post_flight =
        AdtVersion::detect_from_chunks_extended(true, false, false, false, false, false);
    assert!(pre_flight < post_flight);

    // Water/lava rendering was enhanced in WotLK
    let pre_water =
        AdtVersion::detect_from_chunks_extended(true, false, false, false, false, false);
    let post_water =
        AdtVersion::detect_from_chunks_extended(true, true, false, false, false, false);
    assert!(pre_water < post_water);

    // Texture amplifiers were added in Cataclysm
    let pre_amp = AdtVersion::detect_from_chunks_extended(true, true, false, false, false, false);
    let post_amp = AdtVersion::detect_from_chunks_extended(true, true, false, false, false, true);
    assert!(pre_amp < post_amp);

    // Texture parameters were added in MoP
    let pre_params = AdtVersion::detect_from_chunks_extended(true, true, false, false, false, true);
    let post_params = AdtVersion::detect_from_chunks_extended(true, true, false, false, true, true);
    assert!(pre_params < post_params);
}
