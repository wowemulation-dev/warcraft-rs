//! ADT terrain command implementations

use anyhow::{Context, Result};
use clap::Subcommand;
use prettytable::{Cell, Row, Table, format};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wow_adt::{AdtVersion, parse_adt_with_metadata, ParsedAdt};

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
    println!("ADT File Information");
    println!("====================");
    println!();

    // Parse the ADT file with metadata
    let file_handle = File::open(file)
        .with_context(|| format!("Failed to open ADT file: {file}"))?;
    let mut reader = BufReader::new(file_handle);
    let (adt, metadata) = parse_adt_with_metadata(&mut reader)
        .with_context(|| format!("Failed to parse ADT file: {file}"))?;

    // Basic information
    println!("File: {file}");
    println!("Type: {:?}", metadata.file_type);
    println!("Version: {}", format_version(&metadata.version));

    // Check for split files
    let path = Path::new(file);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let dir = path.parent().unwrap_or(Path::new("."));

    if !stem.ends_with("_obj0") && !stem.ends_with("_tex0") {
        // Check for Cataclysm+ split files
        let tex0 = dir.join(format!("{stem}_tex0.adt"));
        let obj0 = dir.join(format!("{stem}_obj0.adt"));
        let lod = dir.join(format!("{stem}_lod.adt"));

        if tex0.exists() || obj0.exists() {
            println!("\nSplit Files Detected (Cataclysm+):");
            if tex0.exists() {
                println!("  - {stem}_tex0.adt");
            }
            if obj0.exists() {
                println!("  - {stem}_obj0.adt");
            }
            if lod.exists() {
                println!("  - {stem}_lod.adt");
            }
        }
    }

    // Display information based on file type
    match adt {
        ParsedAdt::Root(root) => {
            println!("\nTerrain Information:");
            println!("  Chunks: {}/256", root.mcnk_chunks.len());

            println!("\nTextures: {}", root.textures.len());
            if !root.textures.is_empty() && detailed {
                for (i, texture) in root.textures.iter().take(5).enumerate() {
                    println!("  [{}]: {}", i, texture);
                }
                if root.textures.len() > 5 {
                    println!("  ... and {} more", root.textures.len() - 5);
                }
            }

            println!("\nModels (M2): {}", root.models.len());
            if !root.models.is_empty() && detailed {
                for (i, model) in root.models.iter().take(5).enumerate() {
                    println!("  [{}]: {}", i, model);
                }
                if root.models.len() > 5 {
                    println!("  ... and {} more", root.models.len() - 5);
                }
            }

            println!("WMOs: {}", root.wmos.len());
            if !root.wmos.is_empty() && detailed {
                for (i, wmo) in root.wmos.iter().take(5).enumerate() {
                    println!("  [{}]: {}", i, wmo);
                }
                if root.wmos.len() > 5 {
                    println!("  ... and {} more", root.wmos.len() - 5);
                }
            }

            println!("\nPlacements:");
            println!("  M2 Doodads: {}", root.doodad_placements.len());
            println!("  WMO Objects: {}", root.wmo_placements.len());

            // Water
            if let Some(water) = &root.water_data {
                let water_chunks = water
                    .entries
                    .iter()
                    .filter(|e| e.header.has_liquid())
                    .count();
                if water_chunks > 0 {
                    println!("\nWater: {} chunks with water (WotLK+)", water_chunks);
                }
            }

            // Flight bounds
            if root.flight_bounds.is_some() {
                println!("\nFlight Boundaries: Present (TBC+)");
            }

            // Blend mesh system (MoP+)
            if root.blend_mesh_headers.is_some() {
                let header_count = root.blend_mesh_headers.as_ref().unwrap().entries.len();
                println!("\nBlend Mesh System (MoP+):");
                println!("  Headers: {}", header_count);
                if let Some(vertices) = &root.blend_mesh_vertices {
                    println!("  Vertices: {}", vertices.vertices.len());
                }
                if let Some(indices) = &root.blend_mesh_indices {
                    println!("  Indices: {}", indices.indices.len());
                }
            }
        }
        ParsedAdt::Tex0(tex) | ParsedAdt::Tex1(tex) => {
            println!("\nTexture File Information:");
            println!("  Textures: {}", tex.textures.len());
            println!("  MCNK chunks with texture data: {}", tex.mcnk_textures.len());

            if detailed && !tex.textures.is_empty() {
                println!("\nTexture List:");
                for (i, texture) in tex.textures.iter().take(10).enumerate() {
                    println!("  [{}]: {}", i, texture);
                }
                if tex.textures.len() > 10 {
                    println!("  ... and {} more", tex.textures.len() - 10);
                }
            }
        }
        ParsedAdt::Obj0(obj) | ParsedAdt::Obj1(obj) => {
            println!("\nObject File Information:");
            println!("  M2 Models: {}", obj.models.len());
            println!("  WMO Objects: {}", obj.wmos.len());
            println!("  M2 Placements: {}", obj.doodad_placements.len());
            println!("  WMO Placements: {}", obj.wmo_placements.len());
            println!("  MCNK chunks with object refs: {}", obj.mcnk_objects.len());

            if detailed && !obj.models.is_empty() {
                println!("\nM2 Model List:");
                for (i, model) in obj.models.iter().take(10).enumerate() {
                    println!("  [{}]: {}", i, model);
                }
                if obj.models.len() > 10 {
                    println!("  ... and {} more", obj.models.len() - 10);
                }
            }

            if detailed && !obj.wmos.is_empty() {
                println!("\nWMO List:");
                for (i, wmo) in obj.wmos.iter().take(10).enumerate() {
                    println!("  [{}]: {}", i, wmo);
                }
                if obj.wmos.len() > 10 {
                    println!("  ... and {} more", obj.wmos.len() - 10);
                }
            }
        }
        ParsedAdt::Lod(_) => {
            println!("\nLOD File Information:");
            println!("  Level-of-detail file (Cataclysm+)");
        }
    }

    if detailed {
        println!("\nChunk Metadata:");
        println!("  Total chunks discovered: {}", metadata.chunk_count);
        println!("  Discovery time: {:?}", metadata.discovery_duration);
        println!("  Parse time: {:?}", metadata.parse_duration);

        if !metadata.warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &metadata.warnings {
                println!("  - {warning}");
            }
        }

        // Create a table for chunk information
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        table.set_titles(Row::new(vec![
            Cell::new("Chunk Type"),
            Cell::new("Count"),
            Cell::new("Present"),
        ]));

        // Add rows based on discovered chunks
        for chunk_type in metadata.discovery.chunk_types() {
            let count = metadata.discovery.chunks.get(&chunk_type).map(|v| v.len()).unwrap_or(0);
            add_chunk_row(&mut table, &chunk_type.as_str(), count);
        }

        table.printstd();
    }

    Ok(())
}

fn execute_validate(file: &str, level: &str, warnings: bool) -> Result<()> {
    println!("Validating ADT File");
    println!("===================");
    println!();
    println!("File: {file}");
    println!("Level: {level}");
    println!();

    // Parse the file (parsing validates structure)
    let file_handle = File::open(file)
        .with_context(|| format!("Failed to open ADT file: {file}"))?;
    let mut reader = BufReader::new(file_handle);
    let (adt, metadata) = parse_adt_with_metadata(&mut reader)
        .with_context(|| format!("Failed to parse ADT file: {file}"))?;

    // Basic validation is built into the parser
    println!("Validation passed!");
    println!();
    println!("File Type: {:?}", metadata.file_type);
    println!("Version: {}", format_version(&metadata.version));
    println!("Chunks: {}", metadata.chunk_count);

    // Display warnings if requested
    if warnings && !metadata.warnings.is_empty() {
        println!("\nWarnings ({}):", metadata.warnings.len());
        for (i, warning) in metadata.warnings.iter().enumerate() {
            println!("  {}. {}", i + 1, warning);
        }
    }

    // Additional validation based on file type
    match adt {
        ParsedAdt::Root(root) => {
            if root.mcnk_chunks.is_empty() {
                println!("\nWarning: No MCNK terrain chunks found");
            }
            if root.mcnk_chunks.len() > 256 {
                println!("\nError: Too many MCNK chunks ({})", root.mcnk_chunks.len());
            }
        }
        ParsedAdt::Tex0(tex) | ParsedAdt::Tex1(tex) => {
            if tex.textures.is_empty() {
                println!("\nWarning: No textures found in texture file");
            }
        }
        ParsedAdt::Obj0(obj) | ParsedAdt::Obj1(obj) => {
            if obj.models.is_empty() && obj.wmos.is_empty() {
                println!("\nWarning: No objects found in object file");
            }
        }
        ParsedAdt::Lod(_) => {}
    }

    Ok(())
}

fn execute_convert(_input: &str, _output: &str, _to_version: &str) -> Result<()> {
    anyhow::bail!("ADT conversion not yet implemented for binrw-based API");
}

#[cfg(feature = "extract")]
fn execute_extract(
    _file: &str,
    _output_dir: Option<&str>,
    _heightmap: bool,
    _heightmap_format: &str,
    _textures: bool,
    _models: bool,
) -> Result<()> {
    anyhow::bail!("ADT extraction not yet implemented for binrw-based API");
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

    // Parse the ADT file
    let file_handle = File::open(file)
        .with_context(|| format!("Failed to open ADT file: {file}"))?;
    let mut reader = BufReader::new(file_handle);
    let (adt, metadata) = parse_adt_with_metadata(&mut reader)
        .with_context(|| format!("Failed to parse ADT file: {file}"))?;

    // Build tree structure based on file type
    let mut root = TreeNode::new(
        Path::new(file)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(file)
            .to_string(),
        NodeType::Root,
    );

    if !compact {
        root = root.with_metadata("type", &format!("{:?}", metadata.file_type));
        root = root.with_metadata("version", format_version(&metadata.version));
    }

    match adt {
        ParsedAdt::Root(root_adt) => {
            root = build_root_adt_tree(root, &root_adt, show_refs, no_metadata, compact);
        }
        ParsedAdt::Tex0(tex) | ParsedAdt::Tex1(tex) => {
            root = build_tex_adt_tree(root, &tex, no_metadata, compact);
        }
        ParsedAdt::Obj0(obj) | ParsedAdt::Obj1(obj) => {
            root = build_obj_adt_tree(root, &obj, show_refs, no_metadata, compact);
        }
        ParsedAdt::Lod(_) => {
            root = root.add_child(TreeNode::new(
                "LOD file (minimal structure)".to_string(),
                NodeType::Data,
            ));
        }
    }

    // Add metadata node if detailed
    if !no_metadata {
        let mut meta_node = TreeNode::new(
            format!("Metadata ({} chunks)", metadata.chunk_count),
            NodeType::Header,
        );
        meta_node = meta_node.with_metadata("discovery", &format!("{:?}", metadata.discovery_duration));
        meta_node = meta_node.with_metadata("parse", &format!("{:?}", metadata.parse_duration));
        root = root.add_child(meta_node);
    }

    // Render tree
    let options = TreeOptions {
        verbose: false,
        max_depth: depth,
        show_external_refs: show_refs,
        no_color,
        show_metadata: !no_metadata,
        compact,
    };

    println!("{}", render_tree(&root, &options));

    Ok(())
}

fn build_root_adt_tree(
    mut root: crate::utils::tree::TreeNode,
    adt: &wow_adt::api::RootAdt,
    show_refs: bool,
    no_metadata: bool,
    compact: bool,
) -> crate::utils::tree::TreeNode {
    use crate::utils::tree::{NodeType, TreeNode};

    // Header chunk
    let mut header_node = TreeNode::new("MHDR (Header)".to_string(), NodeType::Header);
    if !no_metadata {
        header_node = header_node.with_size(64);
    }
    root = root.add_child(header_node);

    // MCIN chunk index
    let mcin_node = TreeNode::new("MCIN (Chunk Index)".to_string(), NodeType::Header)
        .with_size(256 * 16);
    root = root.add_child(mcin_node);

    // Terrain chunks
    if !adt.mcnk_chunks.is_empty() {
        let mut terrain_node = TreeNode::new(
            format!("MCNK ({} chunks)", adt.mcnk_chunks.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            terrain_node = terrain_node.with_metadata("type", "Terrain chunks");
        }

        // Add sample chunks
        for (_i, chunk) in adt
            .mcnk_chunks
            .iter()
            .enumerate()
            .take(if compact { 2 } else { 4 })
        {
            let mut chunk_node = TreeNode::new(
                format!("Chunk [{},{}]", chunk.header.index_x, chunk.header.index_y),
                NodeType::Data,
            );

            if !no_metadata {
                chunk_node = chunk_node.with_metadata(
                    "flags",
                    &format!("0x{:08X}", chunk.header.flags.value),
                );
                chunk_node = chunk_node.with_metadata(
                    "holes",
                    if chunk.header.holes_low_res != 0 {
                        "yes"
                    } else {
                        "no"
                    },
                );

                // Add counts for subchunks
                let mut subchunks = vec![];
                if chunk.heights.is_some() {
                    subchunks.push("MCVT");
                }
                if chunk.normals.is_some() {
                    subchunks.push("MCNR");
                }
                if chunk.layers.is_some() {
                    subchunks.push("MCLY");
                }
                if chunk.alpha.is_some() {
                    subchunks.push("MCAL");
                }
                if chunk.shadow.is_some() {
                    subchunks.push("MCSH");
                }
                if chunk.refs.is_some() {
                    subchunks.push("MCRF");
                }
                if chunk.liquid.is_some() {
                    subchunks.push("MCLQ");
                }
                if chunk.sound_emitters.is_some() {
                    subchunks.push("MCSE");
                }
                if chunk.vertex_colors.is_some() {
                    subchunks.push("MCCV");
                }
                if chunk.vertex_lighting.is_some() {
                    subchunks.push("MCLV");
                }
                if chunk.doodad_refs.is_some() {
                    subchunks.push("MCRD");
                }
                if chunk.wmo_refs.is_some() {
                    subchunks.push("MCRW");
                }
                if chunk.materials.is_some() {
                    subchunks.push("MCMT");
                }
                if chunk.doodad_disable.is_some() {
                    subchunks.push("MCDD");
                }
                if chunk.blend_batches.is_some() {
                    subchunks.push("MCBB");
                }

                if !subchunks.is_empty() {
                    chunk_node = chunk_node.with_metadata("subchunks", &subchunks.join(", "));
                }
            }

            terrain_node = terrain_node.add_child(chunk_node);
        }

        if adt.mcnk_chunks.len() > 4 && !compact {
            terrain_node = terrain_node.add_child(TreeNode::new(
                format!("... and {} more chunks", adt.mcnk_chunks.len() - 4),
                NodeType::Data,
            ));
        }

        root = root.add_child(terrain_node);
    }

    // Textures
    if !adt.textures.is_empty() {
        let mut tex_node = TreeNode::new(
            format!("MTEX ({} textures)", adt.textures.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            tex_node = tex_node.with_metadata("type", "Texture filenames");
        }

        for (i, texture) in adt
            .textures
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(texture.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &&i.to_string());
            }
            tex_node = tex_node.add_child(file_node);
        }

        if adt.textures.len() > 10 && !compact {
            tex_node = tex_node.add_child(TreeNode::new(
                format!("... and {} more textures", adt.textures.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(tex_node);
    }

    // M2 Models
    if !adt.models.is_empty() {
        let mut model_node = TreeNode::new(
            format!("MMDX/MMID ({} models)", adt.models.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            model_node = model_node.with_metadata("type", "M2 model references");
        }

        for (i, model) in adt
            .models
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(model.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &&i.to_string());
            }
            if show_refs {
                file_node = file_node
                    .with_external_ref(model, crate::utils::tree::detect_ref_type(model));
            }
            model_node = model_node.add_child(file_node);
        }

        if adt.models.len() > 10 && !compact {
            model_node = model_node.add_child(TreeNode::new(
                format!("... and {} more models", adt.models.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(model_node);
    }

    // WMOs
    if !adt.wmos.is_empty() {
        let mut wmo_node = TreeNode::new(
            format!("MWMO/MWID ({} WMOs)", adt.wmos.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            wmo_node = wmo_node.with_metadata("type", "WMO object references");
        }

        for (i, wmo) in adt
            .wmos
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(wmo.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &&i.to_string());
            }
            if show_refs {
                file_node = file_node
                    .with_external_ref(wmo, crate::utils::tree::detect_ref_type(wmo));
            }
            wmo_node = wmo_node.add_child(file_node);
        }

        if adt.wmos.len() > 10 && !compact {
            wmo_node = wmo_node.add_child(TreeNode::new(
                format!("... and {} more WMOs", adt.wmos.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(wmo_node);
    }

    // Placements
    if !adt.doodad_placements.is_empty() {
        let mddf_node = TreeNode::new(
            format!("MDDF ({} doodad placements)", adt.doodad_placements.len()),
            NodeType::Directory,
        );
        root = root.add_child(mddf_node);
    }

    if !adt.wmo_placements.is_empty() {
        let modf_node = TreeNode::new(
            format!("MODF ({} WMO placements)", adt.wmo_placements.len()),
            NodeType::Directory,
        );
        root = root.add_child(modf_node);
    }

    // Version-specific chunks
    if let Some(_flight_bounds) = &adt.flight_bounds {
        let mut fb_node = TreeNode::new("MFBO (Flight Boundaries)".to_string(), NodeType::Chunk);
        if !no_metadata {
            fb_node = fb_node.with_metadata("expansion", "TBC+");
            fb_node = fb_node.with_size(36); // 9 i16 values * 2 * 2 bytes
        }
        root = root.add_child(fb_node);
    }

    if let Some(water) = &adt.water_data {
        let water_chunks = water
            .entries
            .iter()
            .filter(|e| e.header.has_liquid())
            .count();

        // Count total instances and vertex data stats
        let mut total_instances = 0;
        let mut lvf_counts = [0_usize; 4]; // LVF 0-3
        let mut has_vertex_data = 0;
        let mut has_exists_bitmap = 0;

        for entry in &water.entries {
            if entry.header.has_liquid() {
                total_instances += entry.instances.len();

                for (idx, _instance) in entry.instances.iter().enumerate() {
                    // Count vertex data by format
                    if let Some(vertex_data) = entry.vertex_data.get(idx).and_then(|v| v.as_ref()) {
                        has_vertex_data += 1;
                        let format_idx = vertex_data.format() as usize;
                        if format_idx < 4 {
                            lvf_counts[format_idx] += 1;
                        }
                    }

                    // Count exists bitmaps
                    if entry.exists_bitmaps.get(idx).and_then(|b| b.as_ref()).is_some() {
                        has_exists_bitmap += 1;
                    }
                }
            }
        }

        let mut water_node = TreeNode::new(
            format!("MH2O ({} chunks with water)", water_chunks),
            NodeType::Chunk,
        );
        if !no_metadata {
            water_node = water_node.with_metadata("expansion", "WotLK+");
            water_node = water_node.with_metadata("instances", &total_instances.to_string());

            if has_vertex_data > 0 {
                water_node = water_node.with_metadata("vertex_data", &format!("{} instances", has_vertex_data));
            }
            if has_exists_bitmap > 0 {
                water_node = water_node.with_metadata("exists_bitmaps", &format!("{} instances", has_exists_bitmap));
            }
        }

        // Add LVF breakdown if we have vertex data
        if !compact && has_vertex_data > 0 {
            let mut lvf_node = TreeNode::new(
                "Vertex Data Formats".to_string(),
                NodeType::Directory,
            );

            if lvf_counts[0] > 0 {
                let mut lvf0_node = TreeNode::new(
                    format!("LVF 0: HeightDepth ({} instances)", lvf_counts[0]),
                    NodeType::Data,
                );
                if !no_metadata {
                    lvf0_node = lvf0_node.with_metadata("size", "5 bytes/vertex");
                }
                lvf_node = lvf_node.add_child(lvf0_node);
            }

            if lvf_counts[1] > 0 {
                let mut lvf1_node = TreeNode::new(
                    format!("LVF 1: HeightUv ({} instances)", lvf_counts[1]),
                    NodeType::Data,
                );
                if !no_metadata {
                    lvf1_node = lvf1_node.with_metadata("size", "8 bytes/vertex");
                }
                lvf_node = lvf_node.add_child(lvf1_node);
            }

            if lvf_counts[2] > 0 {
                let mut lvf2_node = TreeNode::new(
                    format!("LVF 2: DepthOnly ({} instances)", lvf_counts[2]),
                    NodeType::Data,
                );
                if !no_metadata {
                    lvf2_node = lvf2_node.with_metadata("size", "1 byte/vertex");
                }
                lvf_node = lvf_node.add_child(lvf2_node);
            }

            if lvf_counts[3] > 0 {
                let mut lvf3_node = TreeNode::new(
                    format!("LVF 3: HeightUvDepth ({} instances)", lvf_counts[3]),
                    NodeType::Data,
                );
                if !no_metadata {
                    lvf3_node = lvf3_node.with_metadata("size", "9 bytes/vertex");
                }
                lvf_node = lvf_node.add_child(lvf3_node);
            }

            water_node = water_node.add_child(lvf_node);
        }

        root = root.add_child(water_node);
    }

    if let Some(texture_flags) = &adt.texture_flags {
        let mut tf_node = TreeNode::new(
            format!("MTXF ({} texture flags)", texture_flags.flags.len()),
            NodeType::Chunk,
        );
        if !no_metadata {
            tf_node = tf_node.with_metadata("expansion", "WotLK 3.x+");
        }
        root = root.add_child(tf_node);
    }

    if let Some(amp) = &adt.texture_amplifier {
        let mut amp_node = TreeNode::new(
            format!("MAMP ({} amplifiers)", amp.amplifier),
            NodeType::Chunk,
        );
        if !no_metadata {
            amp_node = amp_node.with_metadata("expansion", "Cataclysm+");
        }
        root = root.add_child(amp_node);
    }

    if let Some(params) = &adt.texture_params {
        let mut params_node = TreeNode::new(
            format!("MTXP ({} texture params)", params.height_params.len()),
            NodeType::Chunk,
        );
        if !no_metadata {
            params_node = params_node.with_metadata("expansion", "MoP+");
        }
        root = root.add_child(params_node);
    }

    // Blend mesh system (MoP+)
    if let Some(headers) = &adt.blend_mesh_headers {
        let mut blend_node =
            TreeNode::new("Blend Mesh System (MoP+)".to_string(), NodeType::Directory);

        let mbmh_node = TreeNode::new(
            format!("MBMH ({} headers)", headers.entries.len()),
            NodeType::Chunk,
        );
        blend_node = blend_node.add_child(mbmh_node);

        if let Some(bounds) = &adt.blend_mesh_bounds {
            let mbbb_node = TreeNode::new(
                format!("MBBB ({} bounding boxes)", bounds.entries.len()),
                NodeType::Chunk,
            );
            blend_node = blend_node.add_child(mbbb_node);
        }

        if let Some(vertices) = &adt.blend_mesh_vertices {
            let mbnv_node = TreeNode::new(
                format!("MBNV ({} vertices)", vertices.vertices.len()),
                NodeType::Chunk,
            );
            blend_node = blend_node.add_child(mbnv_node);
        }

        if let Some(indices) = &adt.blend_mesh_indices {
            let mbmi_node = TreeNode::new(
                format!("MBMI ({} indices)", indices.indices.len()),
                NodeType::Chunk,
            );
            blend_node = blend_node.add_child(mbmi_node);
        }

        root = root.add_child(blend_node);
    }

    root
}

fn build_tex_adt_tree(
    mut root: crate::utils::tree::TreeNode,
    adt: &wow_adt::api::Tex0Adt,
    no_metadata: bool,
    compact: bool,
) -> crate::utils::tree::TreeNode {
    use crate::utils::tree::{NodeType, TreeNode};

    // Textures
    if !adt.textures.is_empty() {
        let mut tex_node = TreeNode::new(
            format!("MTEX ({} textures)", adt.textures.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            tex_node = tex_node.with_metadata("type", "Texture filenames");
        }

        for (i, texture) in adt
            .textures
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(texture.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &&i.to_string());
            }
            tex_node = tex_node.add_child(file_node);
        }

        if adt.textures.len() > 10 && !compact {
            tex_node = tex_node.add_child(TreeNode::new(
                format!("... and {} more textures", adt.textures.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(tex_node);
    }

    // MCNK texture chunks
    if !adt.mcnk_textures.is_empty() {
        let mut mcnk_node = TreeNode::new(
            format!("MCNK Texture Data ({} chunks)", adt.mcnk_textures.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            mcnk_node = mcnk_node.with_metadata("type", "Per-chunk texture layers");
        }

        // Show first few chunks
        for chunk in adt.mcnk_textures.iter().take(if compact { 2 } else { 4 }) {
            let row = chunk.index / 16;
            let col = chunk.index % 16;
            let mut chunk_node = TreeNode::new(
                format!("Chunk [{},{}]", row, col),
                NodeType::Data,
            );

            if let Some(layers) = &chunk.layers {
                chunk_node = chunk_node.with_metadata("layers", &layers.layers.len().to_string());
            }
            if chunk.alpha_maps.is_some() {
                chunk_node = chunk_node.with_metadata("alpha", "present");
            }

            mcnk_node = mcnk_node.add_child(chunk_node);
        }

        if adt.mcnk_textures.len() > 4 && !compact {
            mcnk_node = mcnk_node.add_child(TreeNode::new(
                format!("... and {} more chunks", adt.mcnk_textures.len() - 4),
                NodeType::Data,
            ));
        }

        root = root.add_child(mcnk_node);
    }

    // Texture parameters
    if let Some(params) = &adt.texture_params {
        let mut params_node = TreeNode::new(
            format!("MTXP ({} params)", params.height_params.len()),
            NodeType::Chunk,
        );
        if !no_metadata {
            params_node = params_node.with_metadata("expansion", "MoP+");
        }
        root = root.add_child(params_node);
    }

    root
}

fn build_obj_adt_tree(
    mut root: crate::utils::tree::TreeNode,
    adt: &wow_adt::api::Obj0Adt,
    show_refs: bool,
    no_metadata: bool,
    compact: bool,
) -> crate::utils::tree::TreeNode {
    use crate::utils::tree::{NodeType, TreeNode};

    // M2 Models
    if !adt.models.is_empty() {
        let mut model_node = TreeNode::new(
            format!("MMDX/MMID ({} models)", adt.models.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            model_node = model_node.with_metadata("type", "M2 model references");
        }

        for (i, model) in adt
            .models
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(model.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &&i.to_string());
            }
            if show_refs {
                file_node = file_node
                    .with_external_ref(model, crate::utils::tree::detect_ref_type(model));
            }
            model_node = model_node.add_child(file_node);
        }

        if adt.models.len() > 10 && !compact {
            model_node = model_node.add_child(TreeNode::new(
                format!("... and {} more models", adt.models.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(model_node);
    }

    // WMOs
    if !adt.wmos.is_empty() {
        let mut wmo_node = TreeNode::new(
            format!("MWMO/MWID ({} WMOs)", adt.wmos.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            wmo_node = wmo_node.with_metadata("type", "WMO object references");
        }

        for (i, wmo) in adt
            .wmos
            .iter()
            .enumerate()
            .take(if compact { 3 } else { 10 })
        {
            let mut file_node = TreeNode::new(wmo.clone(), NodeType::File);
            if !no_metadata {
                file_node = file_node.with_metadata("index", &&i.to_string());
            }
            if show_refs {
                file_node = file_node
                    .with_external_ref(wmo, crate::utils::tree::detect_ref_type(wmo));
            }
            wmo_node = wmo_node.add_child(file_node);
        }

        if adt.wmos.len() > 10 && !compact {
            wmo_node = wmo_node.add_child(TreeNode::new(
                format!("... and {} more WMOs", adt.wmos.len() - 10),
                NodeType::Data,
            ));
        }

        root = root.add_child(wmo_node);
    }

    // Placements
    if !adt.doodad_placements.is_empty() {
        let mddf_node = TreeNode::new(
            format!("MDDF ({} doodad placements)", adt.doodad_placements.len()),
            NodeType::Directory,
        );
        root = root.add_child(mddf_node);
    }

    if !adt.wmo_placements.is_empty() {
        let modf_node = TreeNode::new(
            format!("MODF ({} WMO placements)", adt.wmo_placements.len()),
            NodeType::Directory,
        );
        root = root.add_child(modf_node);
    }

    // MCNK object references
    if !adt.mcnk_objects.is_empty() {
        let mut mcnk_node = TreeNode::new(
            format!("MCNK Object References ({} chunks)", adt.mcnk_objects.len()),
            NodeType::Directory,
        );

        if !no_metadata {
            mcnk_node = mcnk_node.with_metadata("type", "Per-chunk object indices");
        }

        // Show first few chunks
        for chunk in adt.mcnk_objects.iter().take(if compact { 2 } else { 4 }) {
            let row = chunk.index / 16;
            let col = chunk.index % 16;
            let mut chunk_node = TreeNode::new(
                format!("Chunk [{},{}]", row, col),
                NodeType::Data,
            );

            if !chunk.doodad_refs.is_empty() {
                chunk_node = chunk_node.with_metadata("doodads", &chunk.doodad_refs.len().to_string());
            }
            if !chunk.wmo_refs.is_empty() {
                chunk_node = chunk_node.with_metadata("wmos", &chunk.wmo_refs.len().to_string());
            }

            mcnk_node = mcnk_node.add_child(chunk_node);
        }

        if adt.mcnk_objects.len() > 4 && !compact {
            mcnk_node = mcnk_node.add_child(TreeNode::new(
                format!("... and {} more chunks", adt.mcnk_objects.len() - 4),
                NodeType::Data,
            ));
        }

        root = root.add_child(mcnk_node);
    }

    root
}

#[cfg(feature = "parallel")]
fn execute_batch(
    pattern: &str,
    output_dir: &str,
    operation: &str,
    _to_version: Option<&str>,
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

    println!("Batch Processing {} files", files.len());
    println!("Operation: {operation}");
    println!();

    let processed = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));

    // Process files in parallel
    files.par_iter().for_each(|file| {
        let result = match operation {
            "validate" => {
                let file_handle = File::open(file);
                match file_handle {
                    Ok(f) => {
                        let mut reader = BufReader::new(f);
                        parse_adt(&mut reader)
                            .map(|_| ())
                            .map_err(|e| anyhow::anyhow!("{}", e))
                    }
                    Err(e) => Err(anyhow::anyhow!("{}", e)),
                }
            }
            _ => Err(anyhow::anyhow!("Invalid operation: {}", operation)),
        };

        match result {
            Ok(()) => {
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

    println!("\nResults:");
    println!("  Processed: {total_processed}");
    println!("  Failed: {total_failed}");

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
fn format_version(version: &AdtVersion) -> &'static str {
    match version {
        AdtVersion::VanillaEarly => "Classic Early (1.0-1.8)",
        AdtVersion::VanillaLate => "Classic Late (1.9+)",
        AdtVersion::TBC => "The Burning Crusade (2.x)",
        AdtVersion::WotLK => "Wrath of the Lich King (3.x)",
        AdtVersion::Cataclysm => "Cataclysm (4.x)",
        AdtVersion::MoP => "Mists of Pandaria (5.x)",
    }
}

fn add_chunk_row(table: &mut Table, name: &str, count: usize) {
    table.add_row(Row::new(vec![
        Cell::new(name),
        Cell::new(&count.to_string()),
        Cell::new(if count > 0 { "✓" } else { "✗" }),
    ]));
}
