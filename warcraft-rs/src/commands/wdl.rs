//! WDL low-resolution terrain command implementations

use anyhow::{Context, Result};
use clap::Subcommand;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use wow_wdl::parser::WdlParser;
use wow_wdl::validation::validate_wdl_file;
use wow_wdl::version::WdlVersion;

use crate::utils::{NodeType, TreeNode, TreeOptions, detect_ref_type, render_tree};

#[derive(Subcommand)]
pub enum WdlCommands {
    /// Display information about a WDL file
    Info {
        /// Path to the WDL file
        file: PathBuf,
    },

    /// Validate a WDL file
    Validate {
        /// Path to the WDL file
        file: PathBuf,

        /// Explicitly specify the WDL version to validate against (e.g., "WotLK", "TBC", "MoP", "Legion")
        #[arg(long, value_name = "VERSION")]
        version: Option<String>,
    },

    /// Convert a WDL file from one version to another
    Convert {
        /// Path to the input WDL file
        input: PathBuf,

        /// Path to write the converted WDL file
        output: PathBuf,

        /// Source version (if not auto-detected, e.g., "WotLK", "TBC", "MoP", "Legion")
        #[arg(long, value_name = "VERSION")]
        from: Option<String>,

        /// Target version (e.g., "WotLK", "TBC", "MoP", "Legion")
        #[arg(short, long, value_name = "VERSION")]
        to: String,
    },

    /// Show tree structure of a WDL file
    Tree {
        /// Path to the WDL file
        file: PathBuf,

        /// WDL version (e.g., "WotLK", "TBC", "MoP", "Legion")
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

/// Maps a version string to a WdlVersion
/// Supports both expansion short names and numeric versions
fn parse_version(version_str: &str) -> Result<WdlVersion> {
    match version_str.to_lowercase().as_str() {
        "vanilla" | "classic" => Ok(WdlVersion::Vanilla),
        "tbc" | "bc" | "burningcrusade" | "burning_crusade" => Ok(WdlVersion::Vanilla), // TBC uses same format as Vanilla
        "wotlk" | "wrath" | "lichking" | "lich_king" | "wlk" => Ok(WdlVersion::Wotlk),
        "cata" | "cataclysm" => Ok(WdlVersion::Cataclysm),
        "mop" | "pandaria" | "mists" | "mists_of_pandaria" => Ok(WdlVersion::Mop),
        "wod" | "draenor" | "warlords" | "warlords_of_draenor" => Ok(WdlVersion::Wod),
        "legion" => Ok(WdlVersion::Legion),
        "bfa" | "bfazeroth" | "battle_for_azeroth" | "battleforazeroth" => Ok(WdlVersion::Bfa),
        "sl" | "shadowlands" => Ok(WdlVersion::Shadowlands),
        "df" | "dragonflight" => Ok(WdlVersion::Dragonflight),
        "latest" => Ok(WdlVersion::Latest),
        _ => {
            // Try parsing as numeric version for compatibility
            match version_str.split('.').next().unwrap_or("").parse::<u32>() {
                Ok(1) | Ok(2) => Ok(WdlVersion::Vanilla), // 1.x (Classic) and 2.x (TBC)
                Ok(3) => Ok(WdlVersion::Wotlk),           // 3.x (WotLK)
                Ok(4) => Ok(WdlVersion::Cataclysm),       // 4.x (Cataclysm)
                Ok(5) => Ok(WdlVersion::Mop),             // 5.x (MoP)
                Ok(6) => Ok(WdlVersion::Wod),             // 6.x (WoD)
                Ok(7) => Ok(WdlVersion::Legion),          // 7.x (Legion)
                Ok(8) => Ok(WdlVersion::Bfa),             // 8.x (BfA)
                Ok(9) => Ok(WdlVersion::Shadowlands),     // 9.x (Shadowlands)
                Ok(10) => Ok(WdlVersion::Dragonflight),   // 10.x (Dragonflight)
                _ => anyhow::bail!("Unknown version: {}", version_str),
            }
        }
    }
}

pub fn execute(command: WdlCommands) -> Result<()> {
    match command {
        WdlCommands::Validate { file, version } => execute_validate(file, version),
        WdlCommands::Convert {
            input,
            output,
            from,
            to,
        } => execute_convert(input, output, from, to),
        WdlCommands::Info { file } => execute_info(file),
        WdlCommands::Tree {
            file,
            version,
            depth,
            no_external_refs,
            no_color,
            compact,
        } => execute_tree(file, version, depth, !no_external_refs, no_color, compact),
    }
}

fn execute_validate(path: PathBuf, version: Option<String>) -> Result<()> {
    use console::style;

    let file =
        File::open(&path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    // Create parser with specified version if provided
    let parser = if let Some(version_str) = version {
        let version = parse_version(&version_str)?;
        WdlParser::with_version(version)
    } else {
        WdlParser::new()
    };

    // Parse the file
    let wdl_file = parser
        .parse(&mut reader)
        .with_context(|| format!("Failed to parse WDL file: {}", path.display()))?;

    // Validate the file
    match validate_wdl_file(&wdl_file) {
        Ok(_) => {
            println!(
                "✓ WDL file '{}' is valid (version: {})",
                style(path.display()).cyan(),
                style(&wdl_file.version).yellow()
            );
        }
        Err(err) => {
            anyhow::bail!("Validation failed: {}", err);
        }
    }

    Ok(())
}

fn execute_convert(
    input: PathBuf,
    output: PathBuf,
    from: Option<String>,
    to: String,
) -> Result<()> {
    use crate::utils::progress::create_progress_bar;
    use console::style;

    // Open the input file
    let input_file = File::open(&input)
        .with_context(|| format!("Failed to open input file: {}", input.display()))?;

    let mut reader = BufReader::new(input_file);

    // Create parser with specified source version if provided
    let parser = if let Some(version_str) = from {
        let version = parse_version(&version_str)?;
        WdlParser::with_version(version)
    } else {
        WdlParser::new()
    };

    // Parse the input file
    println!("Parsing input file...");
    let wdl_file = parser
        .parse(&mut reader)
        .with_context(|| format!("Failed to parse WDL file: {}", input.display()))?;

    // Parse the target version
    let target_version = parse_version(&to)?;

    // Show progress for conversion
    let pb = create_progress_bar(100, "Converting WDL file");
    pb.set_position(25);

    // Convert the file
    let converted_file = wow_wdl::conversion::convert_wdl_file(&wdl_file, target_version)
        .context("Failed to convert WDL file")?;

    pb.set_position(75);

    // Open the output file
    let output_file = File::create(&output)
        .with_context(|| format!("Failed to create output file: {}", output.display()))?;

    let mut writer = BufWriter::new(output_file);

    // Write the converted file
    let output_parser = WdlParser::with_version(target_version);
    output_parser
        .write(&mut writer, &converted_file)
        .context("Failed to write converted file")?;

    pb.finish_and_clear();

    println!(
        "✓ Successfully converted from {} to {}",
        style(&wdl_file.version).yellow(),
        style(&target_version).green()
    );

    Ok(())
}

fn execute_info(path: PathBuf) -> Result<()> {
    use crate::utils::table::create_table;
    use console::style;
    use prettytable::row;

    // Open the file
    let file =
        File::open(&path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    // Parse the file
    let parser = WdlParser::new();
    let wdl_file = parser
        .parse(&mut reader)
        .with_context(|| format!("Failed to parse WDL file: {}", path.display()))?;

    // Display basic information
    println!("\n{}", style("WDL File Information").bold().underlined());
    println!("File: {}", style(path.display()).cyan());
    println!("Version: {}", style(&wdl_file.version).yellow());
    println!("Total Chunks: {}", style(wdl_file.chunks.len()).green());

    // Count map tiles
    let mut tiles_count = 0;
    for &offset in &wdl_file.map_tile_offsets {
        if offset != 0 {
            tiles_count += 1;
        }
    }

    println!(
        "Map Tiles: {}/{}",
        style(tiles_count).green(),
        style(64 * 64).dim()
    );

    // Display WMO information if present
    if !wdl_file.wmo_filenames.is_empty() {
        println!("\n{}", style("WMO Data (Pre-Legion)").bold());
        println!(
            "WMO Models: {}",
            style(wdl_file.wmo_filenames.len()).green()
        );
        println!(
            "WMO Placements: {}",
            style(wdl_file.wmo_placements.len()).green()
        );
    }

    // Display Legion+ model data if present
    if !wdl_file.m2_placements.is_empty() || !wdl_file.wmo_legion_placements.is_empty() {
        println!("\n{}", style("Legion+ Model Data").bold());
        println!(
            "M2 Placements: {}",
            style(wdl_file.m2_placements.len()).green()
        );
        println!(
            "WMO Placements: {}",
            style(wdl_file.wmo_legion_placements.len()).green()
        );
    }

    // Create chunk summary table
    let mut chunk_counts = std::collections::HashMap::new();
    for chunk in &wdl_file.chunks {
        // Convert the 4-byte magic to a string - reverse the bytes to get readable magic
        let chunk_name = if chunk.magic.len() == 4 {
            let reversed = [
                chunk.magic[3],
                chunk.magic[2],
                chunk.magic[1],
                chunk.magic[0],
            ];
            String::from_utf8_lossy(&reversed).to_string()
        } else {
            format!("{:?}", chunk.magic)
        };
        *chunk_counts.entry(chunk_name).or_insert(0) += 1;
    }

    if !chunk_counts.is_empty() {
        println!("\n{}", style("Chunk Summary").bold());
        let mut table = create_table(vec!["Chunk Type", "Count", "Description"]);

        let chunk_descriptions = [
            ("MVER", "Version information"),
            ("MAOF", "Map area offset table"),
            ("MARE", "Map area heightmap data"),
            ("MAHO", "Map area hole data"),
            ("MWMO", "WMO filename strings"),
            ("MWID", "WMO filename indices"),
            ("MODF", "WMO placement data"),
            ("MLDD", "M2 model FileDataIDs"),
            ("MLDX", "M2 model indices"),
            ("MLMD", "M2 model placement data"),
            ("MLMX", "M2 model visibility bounds"),
        ];

        for (chunk_type, description) in &chunk_descriptions {
            if let Some(&count) = chunk_counts.get(*chunk_type) {
                table.add_row(row![
                    style(chunk_type).cyan(),
                    style(count).green(),
                    description
                ]);
            }
        }

        table.printstd();
    }

    Ok(())
}

fn execute_tree(
    path: PathBuf,
    version_str: String,
    depth: Option<usize>,
    show_external_refs: bool,
    no_color: bool,
    compact: bool,
) -> Result<()> {
    let version = parse_version(&version_str).context("Invalid version string")?;

    let file = File::open(&path).context("Failed to open WDL file")?;
    let mut reader = BufReader::new(file);

    let parser = WdlParser::with_version(version);
    let wdl = parser
        .parse(&mut reader)
        .context("Failed to parse WDL file")?;

    // Create root node
    let file_name = path.file_name().unwrap().to_string_lossy();
    let mut root = TreeNode::new(file_name.to_string(), NodeType::Root)
        .with_metadata("version", &wdl.version.to_string())
        .with_metadata("chunks", &wdl.chunks.len().to_string());

    // Count tiles with heightmap data
    let mut tiles_count = 0;
    for &offset in &wdl.map_tile_offsets {
        if offset != 0 {
            tiles_count += 1;
        }
    }
    root = root.with_metadata("tiles", &tiles_count.to_string());

    // Group chunks by type for easier display
    let mut chunk_groups: std::collections::BTreeMap<String, Vec<&wow_wdl::types::Chunk>> =
        std::collections::BTreeMap::new();

    for chunk in &wdl.chunks {
        // Convert the 4-byte magic to a string - reverse the bytes to get readable magic
        let chunk_name = if chunk.magic.len() == 4 {
            let reversed = [
                chunk.magic[3],
                chunk.magic[2],
                chunk.magic[1],
                chunk.magic[0],
            ];
            String::from_utf8_lossy(&reversed).to_string()
        } else {
            format!("{:?}", chunk.magic)
        };
        chunk_groups.entry(chunk_name).or_default().push(chunk);
    }

    // Add chunks to tree
    for (chunk_type, chunks) in chunk_groups {
        let (description, purpose) = match chunk_type.as_str() {
            "MVER" => ("Version Chunk", "Format version identifier"),
            "MAOF" => (
                "Map Area Offset Table",
                "Offsets to map tile heightmap data",
            ),
            "MARE" => (
                "Map Area Heightmap",
                "Low-resolution height data for map tiles",
            ),
            "MAHO" => ("Map Area Holes", "Hole information for map tiles"),
            "MWMO" => ("WMO Filenames", "World Map Object filename strings"),
            "MWID" => ("WMO Indices", "WMO filename indices"),
            "MODF" => ("WMO Placement", "WMO placement and positioning data"),
            "MLDD" => ("M2 FileDataIDs", "M2 model FileDataIDs (Legion+)"),
            "MLDX" => ("M2 Indices", "M2 model indices (Legion+)"),
            "MLMD" => ("M2 Placement", "M2 model placement data (Legion+)"),
            "MLMX" => ("M2 Visibility", "M2 model visibility bounds (Legion+)"),
            _ => ("Unknown Chunk", "Unknown chunk type"),
        };

        let total_size: u64 = chunks.iter().map(|c| c.data.len() as u64).sum();
        let mut chunk_node = TreeNode::new(chunk_type.clone(), NodeType::Chunk)
            .with_size(total_size)
            .with_metadata("count", &chunks.len().to_string())
            .with_metadata("description", description)
            .with_metadata("purpose", purpose);

        // Add external references for certain chunk types
        if show_external_refs {
            match chunk_type.as_str() {
                "MARE" => {
                    // MARE chunks reference the corresponding ADT heightmap data
                    let base_name = file_name.trim_end_matches(".wdl");
                    chunk_node = chunk_node.with_external_ref(
                        &format!("{}/*.adt", base_name),
                        detect_ref_type("file.adt"),
                    );
                }
                "MWMO" => {
                    // MWMO chunks reference WMO files
                    for filename in &wdl.wmo_filenames {
                        if !filename.is_empty() {
                            chunk_node =
                                chunk_node.with_external_ref(filename, detect_ref_type("file.wmo"));
                        }
                    }
                }
                _ => {}
            }
        }

        // For certain chunks, add detailed information
        match chunk_type.as_str() {
            "MVER" => {
                chunk_node = chunk_node.with_metadata("format_version", &wdl.version.to_string());
            }
            "MAOF" => {
                chunk_node = chunk_node.with_metadata("tiles_with_data", &tiles_count.to_string());
                chunk_node = chunk_node.with_metadata("total_tiles", &(64 * 64).to_string());

                // Add sample tile information if not compact
                if !compact && tiles_count > 0 {
                    let mut sample_count = 0;
                    for (i, &offset) in wdl.map_tile_offsets.iter().enumerate() {
                        if offset != 0 && sample_count < 5 {
                            let x = i % 64;
                            let y = i / 64;
                            let tile_node =
                                TreeNode::new(format!("[{:02},{:02}]", x, y), NodeType::Data)
                                    .with_metadata("offset", &format!("0x{:08X}", offset))
                                    .with_metadata("has_heightmap", "true");

                            chunk_node = chunk_node.add_child(tile_node);
                            sample_count += 1;
                        }
                    }

                    if tiles_count > 5 {
                        let summary_node = TreeNode::new(
                            format!("... and {} more tiles", tiles_count - 5),
                            NodeType::Data,
                        );
                        chunk_node = chunk_node.add_child(summary_node);
                    }
                }
            }
            "MARE" => {
                chunk_node =
                    chunk_node.with_metadata("heightmap_chunks", &chunks.len().to_string());
            }
            "MWMO" => {
                chunk_node =
                    chunk_node.with_metadata("wmo_filenames", &wdl.wmo_filenames.len().to_string());

                // Show WMO filenames if not compact
                if !compact {
                    for (i, filename) in wdl.wmo_filenames.iter().enumerate().take(5) {
                        let wmo_node =
                            TreeNode::new(format!("[{}] {}", i, filename), NodeType::File);
                        chunk_node = chunk_node.add_child(wmo_node);
                    }

                    if wdl.wmo_filenames.len() > 5 {
                        let summary_node = TreeNode::new(
                            format!("... and {} more WMO files", wdl.wmo_filenames.len() - 5),
                            NodeType::Data,
                        );
                        chunk_node = chunk_node.add_child(summary_node);
                    }
                }
            }
            "MODF" => {
                chunk_node = chunk_node
                    .with_metadata("wmo_placements", &wdl.wmo_placements.len().to_string());
            }
            "MLMD" => {
                chunk_node =
                    chunk_node.with_metadata("m2_placements", &wdl.m2_placements.len().to_string());
            }
            _ => {}
        }

        root = root.add_child(chunk_node);
    }

    // Add summary information
    if !wdl.wmo_filenames.is_empty() || !wdl.m2_placements.is_empty() {
        let mut model_summary = TreeNode::new("Model Summary".to_string(), NodeType::Data)
            .with_metadata("purpose", "Summary of all model references");

        if !wdl.wmo_filenames.is_empty() {
            model_summary =
                model_summary.with_metadata("wmo_models", &wdl.wmo_filenames.len().to_string());
            model_summary = model_summary
                .with_metadata("wmo_placements", &wdl.wmo_placements.len().to_string());
        }

        if !wdl.m2_placements.is_empty() {
            model_summary =
                model_summary.with_metadata("m2_placements", &wdl.m2_placements.len().to_string());
            model_summary = model_summary.with_metadata(
                "wmo_legion_placements",
                &wdl.wmo_legion_placements.len().to_string(),
            );
        }

        root = root.add_child(model_summary);
    }

    // Render the tree
    let options = TreeOptions {
        max_depth: depth,
        show_external_refs,
        no_color,
        show_metadata: true,
        compact,
    };

    println!("{}", render_tree(&root, &options));
    Ok(())
}
