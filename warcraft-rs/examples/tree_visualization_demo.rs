//! Demonstrates the tree visualization functionality for different file formats
//!
//! This example shows how to use the programmatic tree visualization API
//! to display the structure of various WoW file formats.

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl"
))]
use warcraft_rs::utils::{NodeType, TreeNode, TreeOptions, detect_ref_type, render_tree};

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl"
))]
fn main() {
    println!("🌳 Tree Visualization Demo");
    println!("=========================\n");

    // Create a sample MPQ archive tree structure
    let mpq_tree = create_sample_mpq_tree();
    println!("📦 Sample MPQ Archive Structure:");
    println!("{}\n", render_tree(&mpq_tree, &TreeOptions::default()));

    // Create a sample WDT file tree structure
    let wdt_tree = create_sample_wdt_tree();
    println!("🗺️  Sample WDT File Structure:");
    println!("{}\n", render_tree(&wdt_tree, &TreeOptions::default()));

    // Demonstrate compact mode
    let compact_options = TreeOptions {
        max_depth: Some(2),
        show_external_refs: false,
        no_color: false,
        show_metadata: true,
        compact: true,
    };

    println!("📋 Compact Mode (max depth 2, no external refs):");
    println!("{}", render_tree(&mpq_tree, &compact_options));
}

#[cfg(not(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl"
)))]
fn main() {
    println!("This example requires at least one file format feature to be enabled.");
    println!("Try running with: cargo run --example tree_visualization_demo --features mpq");
}

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl"
))]
fn create_sample_mpq_tree() -> TreeNode {
    TreeNode::new("example.mpq".to_string(), NodeType::Root)
        .with_size(1024 * 1024) // 1MB
        .with_metadata("format", "MPQ v2")
        .with_metadata("files", "4")
        .add_child(
            TreeNode::new("Header".to_string(), NodeType::Header)
                .with_size(32)
                .with_metadata("version", "v2")
                .with_metadata("sector_size", "512"),
        )
        .add_child(
            TreeNode::new("Hash Table".to_string(), NodeType::Table)
                .with_size(1024)
                .with_metadata("entries", "64")
                .with_metadata("encrypted", "true"),
        )
        .add_child(
            TreeNode::new("Block Table".to_string(), NodeType::Table)
                .with_size(256)
                .with_metadata("entries", "16")
                .with_metadata("encrypted", "true"),
        )
        .add_child(
            TreeNode::new("Files".to_string(), NodeType::Directory)
                .with_metadata("count", "4")
                .add_child(
                    TreeNode::new("Interface/".to_string(), NodeType::Directory)
                        .with_metadata("files", "2")
                        .add_child(
                            TreeNode::new("icon.blp".to_string(), NodeType::File)
                                .with_size(2048)
                                .with_metadata("type", "Texture")
                                .with_external_ref(
                                    "Interface/Icons/icon.blp",
                                    detect_ref_type("file.blp"),
                                ),
                        )
                        .add_child(
                            TreeNode::new("frame.xml".to_string(), NodeType::File)
                                .with_size(512)
                                .with_metadata("type", "Interface Definition"),
                        ),
                )
                .add_child(
                    TreeNode::new("model.m2".to_string(), NodeType::File)
                        .with_size(8192)
                        .with_metadata("type", "Model")
                        .with_external_ref("model.skin", detect_ref_type("file.skin"))
                        .with_external_ref("*.blp", detect_ref_type("file.blp")),
                )
                .add_child(
                    TreeNode::new("(listfile)".to_string(), NodeType::File)
                        .with_size(128)
                        .with_metadata("type", "Auto-generated file list")
                        .with_metadata("purpose", "File enumeration"),
                ),
        )
}

#[cfg(any(
    feature = "mpq",
    feature = "dbc",
    feature = "blp",
    feature = "m2",
    feature = "wmo",
    feature = "adt",
    feature = "wdt",
    feature = "wdl"
))]
fn create_sample_wdt_tree() -> TreeNode {
    TreeNode::new("Azeroth.wdt".to_string(), NodeType::Root)
        .with_metadata("type", "Terrain map")
        .with_metadata("tiles", "2048")
        .add_child(
            TreeNode::new("MVER".to_string(), NodeType::Chunk)
                .with_size(4)
                .with_metadata("version", "18")
                .with_metadata("purpose", "Format version identifier"),
        )
        .add_child(
            TreeNode::new("MPHD".to_string(), NodeType::Chunk)
                .with_size(32)
                .with_metadata("flags", "0x00000001")
                .with_metadata("purpose", "Map properties and global data")
                .add_child(
                    TreeNode::new("Flag: WMO-only".to_string(), NodeType::Property)
                        .with_metadata("value", "0x0001"),
                ),
        )
        .add_child(
            TreeNode::new("MAIN".to_string(), NodeType::Chunk)
                .with_size(32768) // 64x64 * 8 bytes
                .with_metadata("grid_size", "64x64")
                .with_metadata("existing_tiles", "2048")
                .with_metadata("purpose", "Tile existence and area mapping")
                .add_child(
                    TreeNode::new("[32,32]".to_string(), NodeType::Data)
                        .with_metadata("area_id", "1519")
                        .with_metadata("has_adt", "true")
                        .with_external_ref("Azeroth_32_32.adt", detect_ref_type("file.adt")),
                )
                .add_child(
                    TreeNode::new("[33,32]".to_string(), NodeType::Data)
                        .with_metadata("area_id", "1519")
                        .with_metadata("has_adt", "true")
                        .with_external_ref("Azeroth_33_32.adt", detect_ref_type("file.adt")),
                )
                .add_child(TreeNode::new(
                    "... and 2046 more tiles".to_string(),
                    NodeType::Data,
                )),
        )
}
