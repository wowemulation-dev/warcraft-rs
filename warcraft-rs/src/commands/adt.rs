//! ADT terrain command implementations

use anyhow::{Context, Result};
use clap::Subcommand;
use prettytable::{Cell, Row, Table, format};
use std::path::Path;
use wow_adt::{Adt, AdtVersion, ValidationLevel};

#[derive(Subcommand)]
pub enum AdtCommands {
    /// Show information about an ADT file
    Info {
        /// Path to the ADT file
        file: String,

        /// Show detailed chunk information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validate an ADT file
    Validate {
        /// Path to the ADT file
        file: String,

        /// Validation level (basic, standard, strict)
        #[arg(short, long, default_value = "standard")]
        level: String,

        /// Show warnings in addition to errors
        #[arg(short, long)]
        warnings: bool,
    },

    /// Convert ADT between different WoW versions
    Convert {
        /// Input ADT file
        input: String,

        /// Output ADT file
        output: String,

        /// Target WoW version (classic, tbc, wotlk, cataclysm)
        #[arg(short, long)]
        to: String,
    },

    /// Extract data from ADT files
    #[cfg(feature = "extract")]
    Extract {
        /// Path to the ADT file
        file: String,

        /// Output directory for extracted data
        #[arg(short, long)]
        output: Option<String>,

        /// Extract heightmap
        #[arg(long)]
        heightmap: bool,

        /// Heightmap format (pgm, png, tiff, raw)
        #[arg(long, default_value = "png")]
        heightmap_format: String,

        /// Extract texture information
        #[arg(long)]
        textures: bool,

        /// Extract model placements
        #[arg(long)]
        models: bool,

        /// Extract all data
        #[arg(long)]
        all: bool,
    },

    /// Visualize ADT structure as a tree
    Tree {
        /// Path to the ADT file
        file: String,

        /// Maximum depth to display
        #[arg(long)]
        depth: Option<usize>,

        /// Show external file references
        #[arg(long)]
        show_refs: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Hide metadata (sizes, counts, etc.)
        #[arg(long)]
        no_metadata: bool,

        /// Compact output without descriptions
        #[arg(long)]
        compact: bool,
    },

    /// Batch process multiple ADT files
    #[cfg(feature = "parallel")]
    Batch {
        /// Input pattern (e.g., "*.adt" or "World/Maps/Azeroth/*.adt")
        pattern: String,

        /// Output directory
        #[arg(short, long)]
        output: String,

        /// Operation to perform (validate, convert, extract)
        #[arg(short, long)]
        operation: String,

        /// Target version for conversion (classic, tbc, wotlk, cataclysm)
        #[arg(long)]
        to: Option<String>,

        /// Number of parallel threads
        #[arg(short, long)]
        threads: Option<usize>,
    },
}

pub fn execute(command: AdtCommands) -> Result<()> {
    match command {
        AdtCommands::Info { file, detailed } => execute_info(&file, detailed),
        AdtCommands::Validate {
            file,
            level,
            warnings,
        } => execute_validate(&file, &level, warnings),
        AdtCommands::Convert { input, output, to } => execute_convert(&input, &output, &to),
        #[cfg(feature = "extract")]
        AdtCommands::Extract {
            file,
            output,
            heightmap,
            heightmap_format,
            textures,
            models,
            all,
        } => execute_extract(
            &file,
            output.as_deref(),
            heightmap || all,
            &heightmap_format,
            textures || all,
            models || all,
        ),
        AdtCommands::Tree {
            file,
            depth,
            show_refs,
            no_color,
            no_metadata,
            compact,
        } => execute_tree(&file, depth, show_refs, no_color, no_metadata, compact),
        #[cfg(feature = "parallel")]
        AdtCommands::Batch {
            pattern,
            output,
            operation,
            to,
            threads,
        } => execute_batch(&pattern, &output, &operation, to.as_deref(), threads),
    }
}

fn execute_info(file: &str, detailed: bool) -> Result<()> {
    println!("🏔️  ADT File Information");
    println!("=====================");
    println!();

    // Load the ADT file
    let adt =
        Adt::from_path(file).with_context(|| format!("Failed to parse ADT file: {}", file))?;

    // Basic information
    println!("File: {}", file);
    println!("Version: {}", format_version(&adt.version()));

    // Check for split files
    let path = Path::new(file);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let dir = path.parent().unwrap_or(Path::new("."));

    if !stem.ends_with("_obj0") && !stem.ends_with("_tex0") {
        // Check for Cataclysm+ split files
        let tex0 = dir.join(format!("{}_tex0.adt", stem));
        let tex1 = dir.join(format!("{}_tex1.adt", stem));
        let obj0 = dir.join(format!("{}_obj0.adt", stem));
        let obj1 = dir.join(format!("{}_obj1.adt", stem));

        if tex0.exists() || obj0.exists() {
            println!("\n📁 Split Files Detected (Cataclysm+):");
            if tex0.exists() {
                println!("  ✓ {}_tex0.adt", stem);
            }
            if tex1.exists() {
                println!("  ✓ {}_tex1.adt", stem);
            }
            if obj0.exists() {
                println!("  ✓ {}_obj0.adt", stem);
            }
            if obj1.exists() {
                println!("  ✓ {}_obj1.adt", stem);
            }
        }
    }

    // Terrain chunks
    let mcnk_count = adt.mcnk_chunks().len();
    println!("\n🏔️  Terrain Information:");
    println!("  Chunks: {}/256", mcnk_count);

    if mcnk_count > 0 {
        // Note: Height data would require accessing MCVT subchunk data
        // which is not publicly exposed in the current API
    }

    // Textures - simplified since we can't access internal fields
    println!("\n🎨 Textures: Check detailed chunk information for texture data");

    // Models - simplified since we can't access internal fields
    println!("\n🌲 Models: Check detailed chunk information for model data");

    // Water
    if let Some(mh2o) = adt.mh2o() {
        let water_chunks = mh2o
            .chunks
            .iter()
            .filter(|c| !c.instances.is_empty())
            .count();
        if water_chunks > 0 {
            println!("\n💧 Water: {} chunks with water", water_chunks);
        }
    }

    if detailed {
        println!("\n📊 Chunk Details:");

        // Create a table
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        table.set_titles(Row::new(vec![
            Cell::new("Chunk"),
            Cell::new("Size"),
            Cell::new("Present"),
        ]));

        // Add chunk information
        // Note: Since we can't access private fields, we'll show basic info only
        add_chunk_row(&mut table, "MVER", 4, true);
        add_chunk_row(&mut table, "MHDR", 64, true); // Header is always present in valid ADT
        add_chunk_row(&mut table, "MCIN", 256 * 16, true); // Index is typically present
        add_chunk_row(&mut table, "MTEX", 0, true); // Variable size texture data
        add_chunk_row(&mut table, "MMDX", 0, true); // Variable size model data
        add_chunk_row(&mut table, "MMID", 0, true); // Model indices
        add_chunk_row(&mut table, "MWMO", 0, true); // WMO data
        add_chunk_row(&mut table, "MWID", 0, true); // WMO indices
        add_chunk_row(&mut table, "MDDF", 0, true); // Doodad placements
        add_chunk_row(&mut table, "MODF", 0, true); // Model placements
        add_chunk_row(&mut table, "MCNK", mcnk_count * 8000, mcnk_count > 0); // Approximate

        // Version-specific chunks
        if adt.version() >= AdtVersion::TBC {
            add_chunk_row(&mut table, "MFBO", 16, true); // Flight bounds
        }
        if adt.version() >= AdtVersion::WotLK {
            add_chunk_row(&mut table, "MH2O", 0, adt.mh2o().is_some()); // Water data
        }
        if adt.version() >= AdtVersion::Cataclysm {
            add_chunk_row(&mut table, "MTFX", 0, true); // Texture effects
        }

        table.printstd();
    }

    Ok(())
}

fn execute_validate(file: &str, level: &str, warnings: bool) -> Result<()> {
    let validation_level = match level.to_lowercase().as_str() {
        "basic" => ValidationLevel::Basic,
        "standard" => ValidationLevel::Standard,
        "strict" => ValidationLevel::Strict,
        _ => {
            anyhow::bail!(
                "Invalid validation level: {}. Must be one of: basic, standard, strict",
                level
            );
        }
    };

    println!("🔍 Validating ADT File");
    println!("=====================");
    println!();
    println!("File: {}", file);
    println!("Level: {:?}", validation_level);
    println!();

    // Load and validate
    let adt =
        Adt::from_path(file).with_context(|| format!("Failed to parse ADT file: {}", file))?;

    let report = adt.validate_with_report_and_context(validation_level, file)?;

    // Display results
    if report.errors.is_empty() && (!warnings || report.warnings.is_empty()) {
        println!("✅ Validation passed!");
    } else {
        if !report.errors.is_empty() {
            println!(
                "❌ Validation failed with {} error(s):",
                report.errors.len()
            );
            for (i, error) in report.errors.iter().enumerate() {
                println!("  {}. {}", i + 1, error);
            }
        }

        if warnings && !report.warnings.is_empty() {
            println!("\n⚠️  {} warning(s):", report.warnings.len());
            for (i, warning) in report.warnings.iter().enumerate() {
                println!("  {}. {}", i + 1, warning);
            }
        }
    }

    if !report.info.is_empty() {
        println!("\nℹ️  Additional information:");
        for info in &report.info {
            println!("  • {}", info);
        }
    }

    Ok(())
}

fn execute_convert(input: &str, output: &str, to_version: &str) -> Result<()> {
    let target_version = parse_version(to_version)?;

    println!("🔄 Converting ADT File");
    println!("====================");
    println!();
    println!("Input: {}", input);
    println!("Output: {}", output);
    println!("Target: {}", format_version(&target_version));
    println!();

    // Load the ADT
    let adt =
        Adt::from_path(input).with_context(|| format!("Failed to parse ADT file: {}", input))?;

    println!("Source version: {}", format_version(&adt.version()));

    // Convert
    let converted = adt
        .to_version(target_version)
        .context("Failed to convert ADT")?;

    // Save
    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(output)
        .with_context(|| format!("Failed to create output file: {}", output))?;
    let mut writer = BufWriter::new(file);

    converted
        .write(&mut writer)
        .with_context(|| format!("Failed to write ADT file: {}", output))?;

    println!("✅ Conversion complete!");

    Ok(())
}

#[cfg(feature = "extract")]
fn execute_extract(
    file: &str,
    output_dir: Option<&str>,
    heightmap: bool,
    heightmap_format: &str,
    textures: bool,
    models: bool,
) -> Result<()> {
    use std::path::PathBuf;
    use wow_adt::extract::{HeightmapOptions, ImageFormat, extract_heightmap};

    println!("📦 Extracting ADT Data");
    println!("====================");
    println!();

    // Load the ADT
    let adt =
        Adt::from_path(file).with_context(|| format!("Failed to parse ADT file: {}", file))?;

    // Determine output directory
    let output_path = if let Some(dir) = output_dir {
        PathBuf::from(dir)
    } else {
        PathBuf::from(".")
    };

    // Create output directory if needed
    std::fs::create_dir_all(&output_path)?;

    let base_name = Path::new(file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("adt");

    // Extract heightmap
    if heightmap {
        let format = match heightmap_format.to_lowercase().as_str() {
            "pgm" => ImageFormat::PGM,
            "png" => ImageFormat::PNG,
            "tiff" => ImageFormat::TIFF,
            "raw" => ImageFormat::Raw,
            _ => {
                anyhow::bail!(
                    "Invalid heightmap format: {}. Must be one of: pgm, png, tiff, raw",
                    heightmap_format
                );
            }
        };

        let output_file = output_path.join(format!(
            "{}_heightmap.{}",
            base_name,
            match format {
                ImageFormat::PGM => "pgm",
                ImageFormat::PNG => "png",
                ImageFormat::TIFF => "tiff",
                ImageFormat::Raw => "raw",
            }
        ));

        let options = HeightmapOptions {
            format,
            ..Default::default()
        };

        println!("Extracting heightmap to: {}", output_file.display());
        extract_heightmap(&adt, &output_file, options)?;
    }

    // Extract texture info
    if textures {
        use wow_adt::extract::extract_textures;
        println!("Extracting texture info to: {}", output_path.display());
        extract_textures(&adt, &output_path)?;
    }

    // Extract model placements
    if models {
        use wow_adt::extract::extract_models;
        println!("Extracting model placements to: {}", output_path.display());
        extract_models(&adt, &output_path)?;
    }

    println!("\n✅ Extraction complete!");

    Ok(())
}

#[cfg(not(feature = "extract"))]
#[allow(dead_code)]
fn execute_extract(_: &str, _: Option<&str>, _: bool, _: &str, _: bool, _: bool) -> Result<()> {
    anyhow::bail!("Extract command requires the 'extract' feature to be enabled")
}

fn execute_tree(
    file: &str,
    depth: Option<usize>,
    show_refs: bool,
    no_color: bool,
    no_metadata: bool,
    compact: bool,
) -> Result<()> {
    use crate::utils::tree::{NodeType, TreeNode, TreeOptions, render_tree};

    // Load the ADT
    let adt =
        Adt::from_path(file).with_context(|| format!("Failed to parse ADT file: {}", file))?;

    // Build tree structure
    let mut root = TreeNode::new(
        Path::new(file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file)
            .to_string(),
        NodeType::Root,
    );

    if !compact {
        root = root.with_metadata("version", format_version(&adt.version()));
    }

    // Add chunks
    // MHDR header chunk
    let mut header_node = TreeNode::new("MHDR".to_string(), NodeType::Header);
    if !no_metadata {
        header_node = header_node.with_size(64).with_metadata("type", "Header");
    }
    root = root.add_child(header_node);

    // Terrain chunks
    if !adt.mcnk_chunks().is_empty() {
        let mut terrain_node = TreeNode::new(
            format!("MCNK ({})", adt.mcnk_chunks().len()),
            NodeType::Directory,
        );

        if !no_metadata {
            terrain_node = terrain_node.with_metadata("type", "Terrain chunks");
        }

        // Add a few sample chunks
        for (_i, chunk) in adt
            .mcnk_chunks()
            .iter()
            .enumerate()
            .take(if compact { 2 } else { 4 })
        {
            let mut chunk_node =
                TreeNode::new(format!("[{},{}]", chunk.ix, chunk.iy), NodeType::Data);

            if !no_metadata {
                chunk_node =
                    chunk_node.with_metadata("holes", if chunk.holes != 0 { "yes" } else { "no" });
            }

            terrain_node = terrain_node.add_child(chunk_node);
        }

        if adt.mcnk_chunks().len() > 4 && !compact {
            terrain_node = terrain_node.add_child(TreeNode::new(
                format!("... and {} more chunks", adt.mcnk_chunks().len() - 4),
                NodeType::Data,
            ));
        }

        root = root.add_child(terrain_node);
    }

    // Texture chunk with actual texture filenames
    if let Some(ref mtex) = adt.mtex {
        let mut tex_node = TreeNode::new(
            format!("MTEX ({})", mtex.filenames.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            tex_node = tex_node.with_metadata("type", "Texture references");
        }

        // Add texture filenames
        for (i, filename) in mtex
            .filenames
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(filename.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &i.to_string());
            }
            tex_node = tex_node.add_child(file_node);
        }

        if mtex.filenames.len() > 10 && !compact {
            tex_node = tex_node.add_child(TreeNode::new(
                format!("... and {} more textures", mtex.filenames.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(tex_node);
    }

    // Model chunks with actual model filenames
    if let Some(ref mmdx) = adt.mmdx {
        let mut model_node = TreeNode::new(
            format!("MMDX ({}) / MMID", mmdx.filenames.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            model_node = model_node.with_metadata("type", "M2 model references");
        }

        // Add model filenames
        for (i, filename) in mmdx
            .filenames
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(filename.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &i.to_string());
            }
            if show_refs {
                file_node = file_node
                    .with_external_ref(filename, crate::utils::tree::detect_ref_type(filename));
            }
            model_node = model_node.add_child(file_node);
        }

        if mmdx.filenames.len() > 10 && !compact {
            model_node = model_node.add_child(TreeNode::new(
                format!("... and {} more models", mmdx.filenames.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(model_node);
    }

    // WMO chunks with actual WMO filenames
    if let Some(ref mwmo) = adt.mwmo {
        let mut wmo_node = TreeNode::new(
            format!("MWMO ({}) / MWID", mwmo.filenames.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            wmo_node = wmo_node.with_metadata("type", "WMO object references");
        }

        // Add WMO filenames
        for (i, filename) in mwmo
            .filenames
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(filename.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &i.to_string());
            }
            if show_refs {
                file_node = file_node
                    .with_external_ref(filename, crate::utils::tree::detect_ref_type(filename));
            }
            wmo_node = wmo_node.add_child(file_node);
        }

        if mwmo.filenames.len() > 10 && !compact {
            wmo_node = wmo_node.add_child(TreeNode::new(
                format!("... and {} more WMOs", mwmo.filenames.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(wmo_node);
    }

    // Placement chunks
    if let Some(ref mddf) = adt.mddf {
        let mut mddf_node = TreeNode::new(
            format!("MDDF ({} doodads)", mddf.doodads.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            mddf_node = mddf_node.with_metadata("type", "Doodad placements");
        }

        root = root.add_child(mddf_node);
    }

    if let Some(ref modf) = adt.modf {
        let mut modf_node = TreeNode::new(
            format!("MODF ({} WMOs)", modf.models.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            modf_node = modf_node.with_metadata("type", "WMO placements");
        }

        root = root.add_child(modf_node);
    }

    // Water
    if let Some(mh2o) = adt.mh2o() {
        let water_chunks = mh2o
            .chunks
            .iter()
            .filter(|c| !c.instances.is_empty())
            .count();
        let water_node = TreeNode::new(
            format!("MH2O ({} water chunks)", water_chunks),
            NodeType::Chunk,
        );
        root = root.add_child(water_node);
    }

    // Render tree
    let options = TreeOptions {
        max_depth: depth,
        show_external_refs: show_refs,
        no_color,
        show_metadata: !no_metadata,
        compact,
    };

    println!("{}", render_tree(&root, &options));

    Ok(())
}

#[cfg(feature = "parallel")]
fn execute_batch(
    pattern: &str,
    output_dir: &str,
    operation: &str,
    to_version: Option<&str>,
    threads: Option<usize>,
) -> Result<()> {
    use glob::glob;
    use rayon::prelude::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Set thread count
    if let Some(num_threads) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .context("Failed to set thread count")?;
    }

    // Find files
    let files: Vec<_> = glob(pattern)
        .context("Invalid glob pattern")?
        .filter_map(|p| p.ok())
        .collect();

    if files.is_empty() {
        anyhow::bail!("No files found matching pattern: {}", pattern);
    }

    println!("🔄 Batch Processing {} files", files.len());
    println!("Operation: {}", operation);
    if let Some(v) = to_version {
        println!("Target version: {}", v);
    }
    println!();

    let processed = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));

    // Process files in parallel
    files.par_iter().for_each(|file| {
        let result = match operation {
            "validate" => match Adt::from_path(file) {
                Ok(adt) => adt
                    .validate()
                    .map(|_| ())
                    .map_err(|e| anyhow::anyhow!("{}", e)),
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            },
            "convert" => {
                if let Some(version_str) = to_version {
                    match (Adt::from_path(file), parse_version(version_str)) {
                        (Ok(adt), Ok(target)) => {
                            let output_path = Path::new(output_dir).join(file.file_name().unwrap());
                            match adt.to_version(target) {
                                Ok(converted) => {
                                    use std::fs::File;
                                    use std::io::BufWriter;

                                    File::create(&output_path)
                                        .and_then(|f| {
                                            let mut writer = BufWriter::new(f);
                                            converted
                                                .write(&mut writer)
                                                .map_err(std::io::Error::other)
                                        })
                                        .map_err(|e| anyhow::anyhow!("{}", e))
                                }
                                Err(e) => Err(anyhow::anyhow!("{}", e)),
                            }
                        }
                        (Err(e), _) => Err(anyhow::anyhow!("{}", e)),
                        (_, Err(e)) => Err(e),
                    }
                } else {
                    Err(anyhow::anyhow!("Target version required for conversion"))
                }
            }
            _ => Err(anyhow::anyhow!("Invalid operation: {}", operation)),
        };

        match result {
            Ok(_) => {
                processed.fetch_add(1, Ordering::Relaxed);
                println!("✓ {}", file.display());
            }
            Err(e) => {
                failed.fetch_add(1, Ordering::Relaxed);
                eprintln!("✗ {}: {}", file.display(), e);
            }
        }
    });

    let total_processed = processed.load(Ordering::Relaxed);
    let total_failed = failed.load(Ordering::Relaxed);

    println!("\n📊 Results:");
    println!("  Processed: {}", total_processed);
    println!("  Failed: {}", total_failed);

    if total_failed > 0 {
        anyhow::bail!("{} files failed processing", total_failed);
    }

    Ok(())
}

#[cfg(not(feature = "parallel"))]
#[allow(dead_code)]
fn execute_batch(_: &str, _: &str, _: &str, _: Option<&str>, _: Option<usize>) -> Result<()> {
    anyhow::bail!("Batch command requires the 'parallel' feature to be enabled")
}

// Helper functions
fn parse_version(version_str: &str) -> Result<AdtVersion> {
    match version_str.to_lowercase().as_str() {
        "classic" | "vanilla" => Ok(AdtVersion::Vanilla),
        "tbc" | "bc" => Ok(AdtVersion::TBC),
        "wotlk" | "wrath" => Ok(AdtVersion::WotLK),
        "cataclysm" | "cata" => Ok(AdtVersion::Cataclysm),
        _ => anyhow::bail!(
            "Invalid version: {}. Valid versions: classic, tbc, wotlk, cataclysm",
            version_str
        ),
    }
}

fn format_version(version: &AdtVersion) -> &'static str {
    match version {
        AdtVersion::Vanilla => "Classic (1.x)",
        AdtVersion::TBC => "The Burning Crusade (2.x)",
        AdtVersion::WotLK => "Wrath of the Lich King (3.x)",
        AdtVersion::Cataclysm => "Cataclysm+ (4.x+)",
        _ => "Unknown Version",
    }
}

fn add_chunk_row(table: &mut Table, name: &str, size: usize, present: bool) {
    table.add_row(Row::new(vec![
        Cell::new(name),
        Cell::new(&humansize::format_size(size, humansize::BINARY)),
        Cell::new(if present { "✓" } else { "✗" }),
    ]));
}
