//! WDT map definition command implementations

use anyhow::{Context, Result};
use clap::Subcommand;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use wow_wdt::{
    WdtReader, WdtWriter,
    chunks::{Chunk, MphdFlags},
    conversion::{convert_wdt, get_conversion_summary},
    version::WowVersion,
};

use crate::utils::{NodeType, TreeNode, TreeOptions, detect_ref_type, render_tree};

#[derive(Subcommand)]
pub enum WdtCommands {
    /// Parse and display information about a WDT file
    Info {
        /// Path to the WDT file
        file: PathBuf,

        /// WoW version (e.g., "1.12.1", "3.3.5a", "WotLK", "TBC", "MoP")
        #[arg(long, default_value = "WotLK")]
        version: String,

        /// Show detailed chunk information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validate a WDT file
    Validate {
        /// Path to the WDT file
        file: PathBuf,

        /// WoW version (e.g., "1.12.1", "3.3.5a", "WotLK", "TBC", "MoP")
        #[arg(long, default_value = "WotLK")]
        version: String,

        /// Show all warnings (not just errors)
        #[arg(short, long)]
        warnings: bool,
    },

    /// Convert a WDT file between versions
    Convert {
        /// Input WDT file
        input: PathBuf,

        /// Output WDT file
        output: PathBuf,

        /// Source WoW version (e.g., "1.12.1", "3.3.5a", "WotLK", "TBC", "MoP")
        #[arg(short = 'f', long)]
        from_version: String,

        /// Target WoW version (e.g., "1.12.1", "3.3.5a", "WotLK", "TBC", "MoP")
        #[arg(short = 't', long)]
        to_version: String,

        /// Preview changes without writing
        #[arg(short, long)]
        preview: bool,
    },

    /// List all tiles with ADT data
    Tiles {
        /// Path to the WDT file
        file: PathBuf,

        /// WoW version (e.g., "1.12.1", "3.3.5a", "WotLK", "TBC", "MoP")
        #[arg(long, default_value = "WotLK")]
        version: String,

        /// Output format (text, json, csv)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show tree structure of a WDT file
    Tree {
        /// Path to the WDT file
        file: PathBuf,

        /// WoW version (e.g., "1.12.1", "3.3.5a", "WotLK", "TBC", "MoP")
        #[arg(long, default_value = "WotLK")]
        version: String,

        /// Maximum depth to display
        #[arg(long)]
        depth: Option<usize>,

        /// Hide external file references
        #[arg(long)]
        no_external_refs: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Show compact metadata inline
        #[arg(long)]
        compact: bool,
    },
}

pub fn execute(command: WdtCommands) -> Result<()> {
    match command {
        WdtCommands::Info {
            file,
            version,
            detailed,
        } => execute_info(file, version, detailed),
        WdtCommands::Validate {
            file,
            version,
            warnings,
        } => execute_validate(file, version, warnings),
        WdtCommands::Convert {
            input,
            output,
            from_version,
            to_version,
            preview,
        } => execute_convert(input, output, from_version, to_version, preview),
        WdtCommands::Tiles {
            file,
            version,
            format,
        } => execute_tiles(file, version, format),
        WdtCommands::Tree {
            file,
            version,
            depth,
            no_external_refs,
            no_color,
            compact,
        } => execute_tree(file, version, depth, !no_external_refs, no_color, compact),
    }
}

fn execute_info(path: PathBuf, version_str: String, detailed: bool) -> Result<()> {
    use console::style;

    let version =
        WowVersion::from_expansion_name(&version_str).context("Invalid version string")?;

    println!("{}", style("WDT File Information").bold().cyan());
    println!("{}", style("===================").cyan());
    println!();

    let file = File::open(&path).context("Failed to open WDT file")?;
    let mut reader = WdtReader::new(BufReader::new(file), version);
    let wdt = reader.read().context("Failed to parse WDT file")?;

    // Basic information
    println!("{}: {}", style("File").bold(), path.display());
    println!("{}: {}", style("Version").bold(), wdt.mver.version);
    println!(
        "{}: {}",
        style("Type").bold(),
        if wdt.is_wmo_only() {
            "WMO-only map"
        } else {
            "Terrain map"
        }
    );
    println!();

    // Flags
    println!("{}", style("MPHD Flags:").bold());
    print_flags(&wdt.mphd.flags);
    println!();

    // Tile statistics
    let tile_count = wdt.count_existing_tiles();
    println!("{}: {} / 4096 tiles", style("ADT Tiles").bold(), tile_count);

    if let Some(ref wmo) = wdt.mwmo
        && !wmo.is_empty()
    {
        println!(
            "{}: {}",
            style("Global WMO").bold(),
            wmo.filenames.first().unwrap_or(&"<empty>".to_string())
        );
    }

    if wdt.maid.is_some() {
        println!("{}: Present (BfA+ format)", style("MAID Chunk").bold());
    }

    // Detailed chunk information
    if detailed {
        println!();
        println!("{}", style("Detailed Chunk Information:").bold());
        println!("{}", style("=========================").cyan());

        // MPHD details
        println!("\n{} (32 bytes)", style("MPHD").yellow());
        println!("  Flags: 0x{:08X}", wdt.mphd.flags.bits());
        if wdt.mphd.has_maid() {
            println!("  LGT FileDataID: {:?}", wdt.mphd.lgt_file_data_id);
            println!("  OCC FileDataID: {:?}", wdt.mphd.occ_file_data_id);
            println!("  FOGS FileDataID: {:?}", wdt.mphd.fogs_file_data_id);
            println!("  MPV FileDataID: {:?}", wdt.mphd.mpv_file_data_id);
            println!("  TEX FileDataID: {:?}", wdt.mphd.tex_file_data_id);
            println!("  WDL FileDataID: {:?}", wdt.mphd.wdl_file_data_id);
            println!("  PD4 FileDataID: {:?}", wdt.mphd.pd4_file_data_id);
        }

        // MAIN details
        println!("\n{} ({} bytes)", style("MAIN").yellow(), 64 * 64 * 8);
        println!("  Existing tiles: {tile_count}");

        // MAID details
        if let Some(ref maid) = wdt.maid {
            println!("\n{} ({} bytes)", style("MAID").yellow(), maid.size());
            println!("  Sections: {}", maid.section_count());
            println!("  Tiles with data: {}", maid.count_existing_tiles());
        }

        // MWMO details
        if let Some(ref mwmo) = wdt.mwmo {
            println!("\n{} ({} bytes)", style("MWMO").yellow(), mwmo.size());
            println!("  Filenames: {}", mwmo.filenames.len());
            for (i, filename) in mwmo.filenames.iter().enumerate() {
                println!("    [{i}] {filename}");
            }
        }

        // MODF details
        if let Some(ref modf) = wdt.modf {
            println!("\n{} ({} bytes)", style("MODF").yellow(), modf.size());
            for (i, entry) in modf.entries.iter().enumerate() {
                println!("  Entry {i}:");
                println!(
                    "    Position: [{:.2}, {:.2}, {:.2}]",
                    entry.position[0], entry.position[1], entry.position[2]
                );
                println!(
                    "    Rotation: [{:.2}, {:.2}, {:.2}] (radians)",
                    entry.rotation[0], entry.rotation[1], entry.rotation[2]
                );
                println!(
                    "             [{:.1}°, {:.1}°, {:.1}°] (degrees)",
                    entry.rotation[0].to_degrees(),
                    entry.rotation[1].to_degrees(),
                    entry.rotation[2].to_degrees()
                );
                let scale_str = if entry.scale == 0 {
                    "0".to_string()
                } else {
                    format!("{:.2}", entry.scale as f32 / 1024.0)
                };
                println!("    Scale: {} ({})", entry.scale, scale_str);
                println!("    UniqueID: 0x{:08X}", entry.unique_id);
            }
        }
    }

    Ok(())
}

fn execute_validate(path: PathBuf, version_str: String, show_warnings: bool) -> Result<()> {
    use console::style;

    let version =
        WowVersion::from_expansion_name(&version_str).context("Invalid version string")?;

    println!("{}", style("Validating WDT File").bold().cyan());
    println!("{}", style("==================").cyan());
    println!();

    let file = File::open(&path).context("Failed to open WDT file")?;
    let mut reader = WdtReader::new(BufReader::new(file), version);
    let wdt = reader.read().context("Failed to parse WDT file")?;

    let warnings = wdt.validate();

    if warnings.is_empty() {
        println!("{} {}", style("✓").green(), style("File is valid!").green());
    } else {
        let errors: Vec<_> = warnings
            .iter()
            .filter(|w| w.contains("Invalid") || w.contains("Missing required"))
            .collect();
        let warnings_only: Vec<_> = warnings
            .iter()
            .filter(|w| !w.contains("Invalid") && !w.contains("Missing required"))
            .collect();

        if !errors.is_empty() {
            println!("{} {} error(s) found:", style("✗").red(), errors.len());
            for error in errors {
                println!("  {} {}", style("•").red(), error);
            }
        }

        if show_warnings && !warnings_only.is_empty() {
            println!(
                "\n{} {} warning(s):",
                style("⚠").yellow(),
                warnings_only.len()
            );
            for warning in warnings_only {
                println!("  {} {}", style("•").yellow(), warning);
            }
        } else if !warnings_only.is_empty() && !show_warnings {
            println!(
                "\n{} {} warning(s) (use -w to show)",
                style("ℹ").blue(),
                warnings_only.len()
            );
        }
    }

    Ok(())
}

fn execute_convert(
    input: PathBuf,
    output: PathBuf,
    from_str: String,
    to_str: String,
    preview: bool,
) -> Result<()> {
    use console::style;

    let from_version =
        WowVersion::from_expansion_name(&from_str).context("Invalid source version")?;
    let to_version = WowVersion::from_expansion_name(&to_str).context("Invalid target version")?;

    println!("{}", style("WDT Version Conversion").bold().cyan());
    println!("{}", style("=====================").cyan());
    println!();

    let file = File::open(&input).context("Failed to open input WDT file")?;
    let mut reader = WdtReader::new(BufReader::new(file), from_version);
    let mut wdt = reader.read().context("Failed to parse WDT file")?;

    let changes = get_conversion_summary(from_version, to_version, wdt.is_wmo_only());

    println!(
        "{}: {} → {}",
        style("Converting").bold(),
        from_version,
        to_version
    );
    println!("{}: {}", style("Input").bold(), input.display());
    println!("{}: {}", style("Output").bold(), output.display());
    println!();

    if changes.is_empty() || (changes.len() == 1 && changes[0].contains("No conversion needed")) {
        println!("{} No conversion needed", style("ℹ").blue());
        return Ok(());
    }

    println!("{}", style("Changes to be made:").bold());
    for change in &changes {
        println!("  {} {}", style("•").cyan(), change);
    }

    if preview {
        println!("\n{} Preview mode - no files written", style("ℹ").blue());
        return Ok(());
    }

    println!();
    convert_wdt(&mut wdt, from_version, to_version).context("Conversion failed")?;

    let output_file = File::create(&output).context("Failed to create output file")?;
    let mut writer = WdtWriter::new(BufWriter::new(output_file));
    writer.write(&wdt).context("Failed to write output file")?;

    println!("{} Conversion complete!", style("✓").green());

    Ok(())
}

fn execute_tiles(path: PathBuf, version_str: String, format: String) -> Result<()> {
    use console::style;

    let version =
        WowVersion::from_expansion_name(&version_str).context("Invalid version string")?;

    let file = File::open(&path).context("Failed to open WDT file")?;
    let mut reader = WdtReader::new(BufReader::new(file), version);
    let wdt = reader.read().context("Failed to parse WDT file")?;

    let mut tiles = Vec::new();

    for y in 0..64 {
        for x in 0..64 {
            if let Some(tile_info) = wdt.get_tile(x, y)
                && tile_info.has_adt
            {
                tiles.push((x, y, tile_info.area_id));
            }
        }
    }

    match format.as_str() {
        "json" => {
            #[cfg(feature = "serde")]
            {
                let json_tiles: Vec<_> = tiles
                    .iter()
                    .map(|(x, y, area_id)| {
                        serde_json::json!({
                            "x": x,
                            "y": y,
                            "area_id": area_id,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_tiles)?);
            }
            #[cfg(not(feature = "serde"))]
            {
                anyhow::bail!("JSON output requires the 'serde' feature to be enabled");
            }
        }
        "csv" => {
            println!("x,y,area_id");
            for (x, y, area_id) in tiles {
                println!("{x},{y},{area_id}");
            }
        }
        _ => {
            println!("{}", style("Existing ADT Tiles").bold().cyan());
            println!("{}", style("==================").cyan());
            println!();
            println!("Total: {} tiles", tiles.len());
            println!();

            for (x, y, area_id) in tiles {
                println!("  [{x:2},{y:2}] - Area ID: {area_id}");
            }
        }
    }

    Ok(())
}

fn print_flags(flags: &MphdFlags) {
    use console::style;

    let flag_descriptions = [
        (MphdFlags::WDT_USES_GLOBAL_MAP_OBJ, "WMO-only map"),
        (MphdFlags::ADT_HAS_MCCV, "ADTs have vertex colors"),
        (MphdFlags::ADT_HAS_BIG_ALPHA, "ADTs use big alpha"),
        (
            MphdFlags::ADT_HAS_DOODADREFS_SORTED_BY_SIZE_CAT,
            "Doodads sorted by size",
        ),
        (
            MphdFlags::ADT_HAS_LIGHTING_VERTICES,
            "ADTs have lighting vertices (deprecated)",
        ),
        (MphdFlags::ADT_HAS_UPSIDE_DOWN_GROUND, "Flip ground display"),
        (MphdFlags::UNK_FIRELANDS, "Universal flag (4.3.4+)"),
        (
            MphdFlags::ADT_HAS_HEIGHT_TEXTURING,
            "Height texturing enabled",
        ),
        (MphdFlags::UNK_LOAD_LOD, "Load LOD files"),
        (MphdFlags::WDT_HAS_MAID, "Has MAID chunk (8.1.0+)"),
    ];

    for (flag, desc) in flag_descriptions {
        if flags.contains(flag) {
            println!("  {} 0x{:04X} - {}", style("✓").green(), flag.bits(), desc);
        }
    }

    // Show raw value
    println!("  Raw value: 0x{:08X}", flags.bits());
}

fn execute_tree(
    path: PathBuf,
    version_str: String,
    depth: Option<usize>,
    show_external_refs: bool,
    no_color: bool,
    compact: bool,
) -> Result<()> {
    let version =
        WowVersion::from_expansion_name(&version_str).context("Invalid version string")?;

    let file = File::open(&path).context("Failed to open WDT file")?;
    let mut reader = WdtReader::new(BufReader::new(file), version);
    let wdt = reader.read().context("Failed to parse WDT file")?;

    // Create root node
    let file_name = path
        .file_name()
        .expect("path should have a file name component")
        .to_string_lossy();
    let map_type = if wdt.is_wmo_only() {
        "WMO-only map"
    } else {
        "Terrain map"
    };

    let mut root = TreeNode::new(file_name.to_string(), NodeType::Root)
        .with_metadata("type", map_type)
        .with_metadata("tiles", &wdt.count_existing_tiles().to_string());

    // Add MVER chunk
    let mver_node = TreeNode::new("MVER".to_string(), NodeType::Chunk)
        .with_size(4)
        .with_metadata("version", &wdt.mver.version.to_string())
        .with_metadata("purpose", "Format version identifier");
    root = root.add_child(mver_node);

    // Add MPHD chunk with detailed flag information
    let mut mphd_node = TreeNode::new("MPHD".to_string(), NodeType::Chunk)
        .with_size(32)
        .with_metadata("flags", &format!("0x{:08X}", wdt.mphd.flags.bits()))
        .with_metadata("purpose", "Map properties and global data");

    // Add flag details
    let flag_descriptions = [
        (MphdFlags::WDT_USES_GLOBAL_MAP_OBJ, "WMO-only"),
        (MphdFlags::ADT_HAS_MCCV, "Vertex colors"),
        (MphdFlags::ADT_HAS_BIG_ALPHA, "Big alpha"),
        (
            MphdFlags::ADT_HAS_DOODADREFS_SORTED_BY_SIZE_CAT,
            "Sorted doodads",
        ),
        (MphdFlags::UNK_FIRELANDS, "Universal flag"),
        (MphdFlags::ADT_HAS_HEIGHT_TEXTURING, "Height texturing"),
        (MphdFlags::WDT_HAS_MAID, "Has MAID chunk"),
    ];

    for (flag, desc) in flag_descriptions {
        if wdt.mphd.flags.contains(flag) {
            let flag_node = TreeNode::new(format!("Flag: {desc}"), NodeType::Property)
                .with_metadata("value", &format!("0x{:04X}", flag.bits()));
            mphd_node = mphd_node.add_child(flag_node);
        }
    }

    // Add BfA+ file data IDs if present
    if wdt.mphd.has_maid() {
        if let Some(lgt_id) = wdt.mphd.lgt_file_data_id {
            mphd_node = mphd_node.with_external_ref(
                &format!("LGT FileDataID: {lgt_id}"),
                detect_ref_type("file.lgt"),
            );
        }
        if let Some(occ_id) = wdt.mphd.occ_file_data_id {
            mphd_node = mphd_node.with_external_ref(
                &format!("OCC FileDataID: {occ_id}"),
                detect_ref_type("file.occ"),
            );
        }
    }

    root = root.add_child(mphd_node);

    // Add MAIN chunk with tile information
    let mut main_node = TreeNode::new("MAIN".to_string(), NodeType::Chunk)
        .with_size(64 * 64 * 8) // 64x64 grid, 8 bytes per entry
        .with_metadata("grid_size", "64x64")
        .with_metadata("existing_tiles", &wdt.count_existing_tiles().to_string())
        .with_metadata("purpose", "Tile existence and area mapping");

    // Add sample tiles with external references
    let base_name = file_name.trim_end_matches(".wdt");
    let mut tile_count = 0;
    for y in 0..64 {
        for x in 0..64 {
            if let Some(tile_info) = wdt.get_tile(x, y)
                && tile_info.has_adt
            {
                tile_count += 1;

                // Show first few tiles as examples
                if tile_count <= 5 || !compact {
                    let mut tile_node = TreeNode::new(format!("[{x:02},{y:02}]"), NodeType::Data)
                        .with_metadata("area_id", &tile_info.area_id.to_string())
                        .with_metadata("has_adt", "true");

                    if show_external_refs {
                        tile_node = tile_node.with_external_ref(
                            &format!("{base_name}_{x:02}_{y:02}.adt"),
                            detect_ref_type("file.adt"),
                        );
                    }

                    main_node = main_node.add_child(tile_node);
                }
            }
        }
    }

    // Add summary if we truncated
    if compact && tile_count > 5 {
        let summary_node = TreeNode::new(
            format!("... and {} more tiles", tile_count - 5),
            NodeType::Data,
        );
        main_node = main_node.add_child(summary_node);
    }

    root = root.add_child(main_node);

    // Add MAID chunk if present
    if let Some(ref maid) = wdt.maid {
        let mut maid_node = TreeNode::new("MAID".to_string(), NodeType::Chunk)
            .with_size(maid.size() as u64)
            .with_metadata("sections", &maid.section_count().to_string())
            .with_metadata("tiles_with_data", &maid.count_existing_tiles().to_string())
            .with_metadata("purpose", "File Data IDs for all map components");

        // Add section information
        let sections = [
            "Root ADT files",
            "Obj0 ADT files",
            "Obj1 ADT files",
            "Tex0 ADT files",
            "LOD ADT files",
            "Map textures",
            "Map normal textures",
            "Minimap textures",
        ];

        for (i, section_name) in sections.iter().enumerate() {
            if i < maid.section_count() {
                let section_node =
                    TreeNode::new(format!("Section {i}: {section_name}"), NodeType::Property)
                        .with_metadata("entries", "4096");
                maid_node = maid_node.add_child(section_node);
            }
        }

        root = root.add_child(maid_node);
    }

    // Add MWMO chunk if present
    if let Some(ref mwmo) = wdt.mwmo {
        let mut mwmo_node = TreeNode::new("MWMO".to_string(), NodeType::Chunk)
            .with_size(mwmo.size() as u64)
            .with_metadata("wmo_count", &mwmo.filenames.len().to_string())
            .with_metadata("purpose", "WMO filename strings");

        if mwmo.filenames.is_empty() {
            let empty_node = TreeNode::new("(empty)".to_string(), NodeType::Data)
                .with_metadata("note", "Expected for terrain maps");
            mwmo_node = mwmo_node.add_child(empty_node);
        } else {
            for (i, filename) in mwmo.filenames.iter().enumerate() {
                let mut wmo_node = TreeNode::new(format!("[{i}] {filename}"), NodeType::File);

                if show_external_refs {
                    wmo_node = wmo_node.with_external_ref(filename, detect_ref_type("file.wmo"));
                }

                mwmo_node = mwmo_node.add_child(wmo_node);
            }
        }

        root = root.add_child(mwmo_node);
    }

    // Add MODF chunk if present
    if let Some(ref modf) = wdt.modf {
        let mut modf_node = TreeNode::new("MODF".to_string(), NodeType::Chunk)
            .with_size(modf.size() as u64)
            .with_metadata("wmo_placements", &modf.entries.len().to_string())
            .with_metadata("purpose", "WMO placement data");

        for (i, entry) in modf.entries.iter().enumerate() {
            let placement_node = TreeNode::new(format!("Placement {i}"), NodeType::Data)
                .with_metadata(
                    "position",
                    &format!(
                        "[{:.1}, {:.1}, {:.1}]",
                        entry.position[0], entry.position[1], entry.position[2]
                    ),
                )
                .with_metadata(
                    "rotation",
                    &format!(
                        "[{:.1}°, {:.1}°, {:.1}°]",
                        entry.rotation[0].to_degrees(),
                        entry.rotation[1].to_degrees(),
                        entry.rotation[2].to_degrees()
                    ),
                )
                .with_metadata("scale", &format!("{:.2}", entry.scale as f32 / 1024.0))
                .with_metadata("unique_id", &format!("0x{:08X}", entry.unique_id));

            modf_node = modf_node.add_child(placement_node);
        }

        root = root.add_child(modf_node);
    }

    // Render the tree
    let options = TreeOptions {
        verbose: false,
        max_depth: depth,
        show_external_refs,
        no_color,
        show_metadata: true,
        compact,
    };

    println!("{}", render_tree(&root, &options));
    Ok(())
}
