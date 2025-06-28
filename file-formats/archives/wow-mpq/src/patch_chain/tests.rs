//! Tests for patch chain functionality

use super::*;
use crate::{ArchiveBuilder, ListfileOption};
use tempfile::TempDir;

/// Helper to create test archives
fn create_test_archive(dir: &Path, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let path = dir.join(name);
    let mut builder = ArchiveBuilder::new().listfile_option(ListfileOption::Generate);

    for (filename, data) in files {
        builder = builder.add_file_data(data.to_vec(), filename);
    }

    builder.build(&path).unwrap();
    path
}

#[test]
fn test_wow_loading_order() {
    // Test that our patch chain correctly implements WoW's loading order
    let temp = TempDir::new().unwrap();

    // Create test files that exist in multiple archives
    let common_file = "Interface/FrameXML/UIParent.lua";
    let locale_file = "Sound/Music/ZoneMusic/Forest.mp3";
    let patch_file = "DBFilesClient/Spell.dbc";

    // Create archives with different versions of the same files
    let common_mpq = create_test_archive(
        temp.path(),
        "common.MPQ",
        &[
            (common_file, b"common version"),
            (locale_file, b"common locale"),
            (patch_file, b"common patch"),
        ],
    );

    let locale_mpq = create_test_archive(
        temp.path(),
        "locale-enUS.MPQ",
        &[
            (locale_file, b"locale version"), // Should override common
        ],
    );

    let patch_mpq = create_test_archive(
        temp.path(),
        "patch.MPQ",
        &[
            (common_file, b"patch version"), // Should override common
            (patch_file, b"patch version"),  // Should override common
        ],
    );

    let patch2_mpq = create_test_archive(
        temp.path(),
        "patch-2.MPQ",
        &[
            (patch_file, b"patch2 version"), // Should override patch
        ],
    );

    // Create patch chain following WoW's loading order
    let mut chain = PatchChain::new();

    // Load in the correct order (lower priority first)
    chain.add_archive(&common_mpq, 0).unwrap(); // Base content
    chain.add_archive(&locale_mpq, 100).unwrap(); // Locale overrides base
    chain.add_archive(&patch_mpq, 1000).unwrap(); // Patches override everything
    chain.add_archive(&patch2_mpq, 1001).unwrap(); // Later patches override earlier

    // Verify correct file resolution
    assert_eq!(
        chain.read_file(common_file).unwrap(),
        b"patch version", // From patch.MPQ, not common.MPQ
        "Common file should come from patch"
    );

    assert_eq!(
        chain.read_file(locale_file).unwrap(),
        b"locale version", // From locale-enUS.MPQ, not common.MPQ
        "Locale file should come from locale archive"
    );

    assert_eq!(
        chain.read_file(patch_file).unwrap(),
        b"patch2 version", // From patch-2.MPQ, not patch.MPQ or common.MPQ
        "Patched file should come from latest patch"
    );
}

#[test]
fn test_locale_patch_priority() {
    // Test that locale patches have higher priority than general patches
    let temp = TempDir::new().unwrap();
    let locale = "enUS";

    let test_file = "Interface/AddOns/Blizzard_AuctionUI/Blizzard_AuctionUI.lua";

    // Create subdirectory for locale patches
    std::fs::create_dir_all(temp.path().join(locale)).unwrap();

    // Create archives
    let patch_mpq = create_test_archive(temp.path(), "patch.MPQ", &[(test_file, b"general patch")]);

    let locale_patch = create_test_archive(
        &temp.path().join(locale),
        &format!("patch-{locale}.MPQ"),
        &[(test_file, b"locale patch")],
    );

    // Create chain with WoW's priority system
    let mut chain = PatchChain::new();
    chain.add_archive(&patch_mpq, 10).unwrap(); // General patch
    chain.add_archive(&locale_patch, 20).unwrap(); // Locale patch (higher priority)

    // Locale patch should override general patch
    assert_eq!(
        chain.read_file(test_file).unwrap(),
        b"locale patch",
        "Locale patch should override general patch"
    );
}

#[test]
fn test_expansion_loading_order() {
    // Test expansion archive loading order (3.3.5a)
    let temp = TempDir::new().unwrap();

    let test_file = "World/Maps/Northrend/Northrend.wdt";

    // Create archives in 3.3.5a structure
    let common = create_test_archive(temp.path(), "common.MPQ", &[(test_file, b"common")]);

    let expansion = create_test_archive(temp.path(), "expansion.MPQ", &[(test_file, b"expansion")]);

    let lichking = create_test_archive(temp.path(), "lichking.MPQ", &[(test_file, b"lichking")]);

    // Load in WoW order
    let mut chain = PatchChain::new();
    chain.add_archive(&common, 0).unwrap();
    chain.add_archive(&expansion, 1).unwrap();
    chain.add_archive(&lichking, 2).unwrap();

    // Latest expansion should win
    assert_eq!(
        chain.read_file(test_file).unwrap(),
        b"lichking",
        "Expansion archives should override in order"
    );
}

#[test]
fn test_custom_patch_priority() {
    // Test that custom patches (patch-4.MPQ, patch-x.MPQ) work correctly
    let temp = TempDir::new().unwrap();

    let test_file = "Interface/AddOns/MyAddon/Core.lua";

    // Create official and custom patches
    let patch3 = create_test_archive(
        temp.path(),
        "patch-3.MPQ",
        &[(test_file, b"official patch 3")],
    );

    let patch4 = create_test_archive(
        temp.path(),
        "patch-4.MPQ",
        &[(test_file, b"custom patch 4")],
    );

    let patch_x = create_test_archive(
        temp.path(),
        "patch-x.MPQ",
        &[(test_file, b"custom patch x")],
    );

    // Test numerical ordering
    let mut chain = PatchChain::new();
    chain.add_archive(&patch3, 3).unwrap();
    chain.add_archive(&patch4, 4).unwrap();

    assert_eq!(
        chain.read_file(test_file).unwrap(),
        b"custom patch 4",
        "Higher numbered patches should override"
    );

    // Test alphabetical ordering
    chain.add_archive(&patch_x, 100).unwrap(); // Much higher priority

    assert_eq!(
        chain.read_file(test_file).unwrap(),
        b"custom patch x",
        "Letter patches with high priority should override"
    );
}
