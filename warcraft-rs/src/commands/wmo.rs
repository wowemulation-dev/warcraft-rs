//! WMO world map object command implementations

use crate::utils::tree::{NodeType, TreeNode, TreeOptions};
use anyhow::{Context, Result};
use clap::Subcommand;
// use prettytable::{Cell, Row, Table, format};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wow_wmo::parse_wmo_with_metadata;

#[derive(Subcommand)]
pub enum WmoCommands {
    /// Show information about a WMO file
    Info {
        /// Path to the WMO file
        file: String,

        /// Show detailed information including all chunks
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validate a WMO file
    Validate {
        /// Path to the WMO file
        file: String,

        /// Show warnings in addition to errors
        #[arg(short, long)]
        warnings: bool,

        /// Show detailed validation report
        #[arg(short, long)]
        detailed: bool,
    },

    /// Convert WMO between different WoW versions
    Convert {
        /// Input WMO file
        input: String,

        /// Output WMO file
        output: String,

        /// Target WoW version number (17-27)
        #[arg(short, long)]
        to: u32,
    },

    /// Export WMO data
    Export {
        /// Path to the WMO file
        file: String,

        /// Output format (obj, gltf, json)
        #[arg(short, long, default_value = "obj")]
        format: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,

        /// Include textures
        #[arg(short, long)]
        textures: bool,

        /// Include doodads
        #[arg(short, long)]
        doodads: bool,
    },

    /// List WMO components
    List {
        /// Path to the WMO file
        file: String,

        /// Component to list (groups, doodads, portals, lights, materials)
        #[arg(short, long, default_value = "all")]
        component: String,
    },

    /// Extract WMO groups
    ExtractGroups {
        /// Path to the WMO file
        file: String,

        /// Group indices to extract (comma-separated, or "all")
        #[arg(short, long, default_value = "all")]
        groups: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Visualize WMO structure as a tree
    Tree {
        /// Path to the WMO file
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

        /// Detailed output with full field information
        #[arg(long)]
        detailed: bool,
    },
}

pub fn execute(command: WmoCommands) -> Result<()> {
    match command {
        WmoCommands::Info { file, detailed } => info(&file, detailed),
        WmoCommands::Validate {
            file,
            warnings,
            detailed,
        } => validate(&file, warnings, detailed),
        WmoCommands::Convert { input, output, to } => convert(&input, &output, to),
        WmoCommands::Export { .. } => {
            anyhow::bail!("WMO export functionality not yet implemented");
        }
        WmoCommands::List { file, component } => list(&file, &component),
        WmoCommands::ExtractGroups { .. } => {
            anyhow::bail!("WMO group extraction not yet implemented");
        }
        WmoCommands::Tree {
            file,
            depth,
            show_refs,
            no_color,
            no_metadata,
            compact,
            detailed,
        } => tree(
            &file,
            depth,
            show_refs,
            no_color,
            no_metadata,
            compact,
            detailed,
        ),
    }
}

fn info(path: &str, detailed: bool) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    // Use our new binrw-based parser with metadata
    let parse_result = parse_wmo_with_metadata(&mut reader)
        .with_context(|| format!("Failed to parse WMO file: {}", path.display()))?;

    println!("WMO Information");
    println!("===============");
    println!();

    // Display information based on file type
    match &parse_result.wmo {
        wow_wmo::ParsedWmo::Root(root) => {
            println!("File Type: Root WMO");
            println!("Version: {}", root.version);

            // Basic counts from MOHD
            println!("\nObject Counts:");
            println!("  Materials: {}", root.n_materials);
            println!("  Groups: {}", root.n_groups);
            println!("  Portals: {}", root.n_portals);
            println!("  Lights: {}", root.n_lights);

            // Extended chunk data
            if !root.materials.is_empty() {
                println!("\nExtended Data:");
                println!("  Materials (MOMT): {} entries", root.materials.len());
            }
            if !root.group_names.is_empty() {
                println!("  Group Names (MOGN): {} entries", root.group_names.len());
                // Show first few group names
                for (i, name) in root.group_names.iter().take(3).enumerate() {
                    println!("    [{}]: {}", i, name);
                }
                if root.group_names.len() > 3 {
                    println!("    ... and {} more", root.group_names.len() - 3);
                }
            }
            if !root.group_info.is_empty() {
                println!("  Group Info (MOGI): {} entries", root.group_info.len());
            }
            if !root.lights.is_empty() {
                println!("  Light Definitions (MOLT): {} entries", root.lights.len());
            }
            if !root.doodad_sets.is_empty() {
                println!("  Doodad Sets (MODS): {} entries", root.doodad_sets.len());
            }
            if !root.doodad_defs.is_empty() {
                println!(
                    "  Doodad Definitions (MODD): {} entries",
                    root.doodad_defs.len()
                );
            }
            if !root.fogs.is_empty() {
                println!("  Fog Definitions (MFOG): {} entries", root.fogs.len());
            }
            if !root.convex_volume_planes.is_empty() {
                println!(
                    "  Convex Volume Planes (MCVP): {} entries (Cataclysm+)",
                    root.convex_volume_planes.len()
                );
            }
            if !root.group_file_ids.is_empty() {
                println!(
                    "  Group File IDs (GFID): {} entries (Modern+)",
                    root.group_file_ids.len()
                );
            }
        }
        wow_wmo::ParsedWmo::Group(group) => {
            println!("File Type: Group WMO");
            println!("Version: {}", group.version);
            println!("Group Index: {}", group.group_index);
            println!("Group Name Index: {}", group.group_name_index);

            println!("\nGeometry:");
            println!("  Triangles: {}", group.n_triangles);
            println!("  Vertices: {}", group.n_vertices);

            if !group.vertex_positions.is_empty() {
                println!("\nExtended Data:");
                println!(
                    "  Vertex Positions (MOVT): {} vertices",
                    group.vertex_positions.len()
                );
            }
            if !group.vertex_indices.is_empty() {
                println!(
                    "  Vertex Indices (MOVI): {} indices",
                    group.vertex_indices.len()
                );
            }
            if !group.texture_coords.is_empty() {
                println!(
                    "  Texture Coordinates (MOTV): {} coords",
                    group.texture_coords.len()
                );
            }
            if !group.vertex_normals.is_empty() {
                println!(
                    "  Vertex Normals (MONR): {} normals",
                    group.vertex_normals.len()
                );
            }
            if !group.render_batches.is_empty() {
                println!(
                    "  Render Batches (MOBA): {} batches",
                    group.render_batches.len()
                );
            }
            if !group.vertex_colors.is_empty() {
                println!(
                    "  Vertex Colors (MOCV): {} colors",
                    group.vertex_colors.len()
                );
            }
            if group.liquid_header.is_some() {
                println!("  Liquid Data (MLIQ): Present");
            }
            if !group.triangle_strip_indices.is_empty() {
                println!(
                    "  Triangle Strip Indices (MORI): {} indices (Modern+)",
                    group.triangle_strip_indices.len()
                );
            }
            if !group.additional_render_batches.is_empty() {
                println!(
                    "  Additional Render Batches (MORB): {} batches (Modern+)",
                    group.additional_render_batches.len()
                );
            }
            if !group.tangent_arrays.is_empty() {
                println!(
                    "  Tangent Arrays (MOTA): {} tangents (Normal Mapping)",
                    group.tangent_arrays.len()
                );
            }
            if !group.shadow_batches.is_empty() {
                println!(
                    "  Shadow Batches (MOBS): {} batches (Shadow Rendering)",
                    group.shadow_batches.len()
                );
            }
        }
    }

    if detailed {
        // Show chunk metadata from our new parser
        if let Some(metadata) = &parse_result.metadata() {
            println!("\nChunk Information:");
            println!("  Total chunks: {}", metadata.total_chunks());
            if metadata.has_unknown_chunks() {
                println!("  Unknown chunks: {}", metadata.unknown_count());
            }
            if metadata.has_malformed_chunks() {
                println!("  Malformed chunks: {}", metadata.malformed_count());
            }

            // Show extended chunk data if available
            match &parse_result.wmo {
                wow_wmo::ParsedWmo::Root(root) => {
                    if !root.materials.is_empty() {
                        println!("\nMaterials (MOMT): {} entries", root.materials.len());
                    }
                    if !root.group_names.is_empty() {
                        println!("Group Names (MOGN): {} entries", root.group_names.len());
                    }
                    if !root.lights.is_empty() {
                        println!("Lights (MOLT): {} entries", root.lights.len());
                    }
                    if !root.fogs.is_empty() {
                        println!("Fog Definitions (MFOG): {} entries", root.fogs.len());
                    }
                    if !root.convex_volume_planes.is_empty() {
                        println!(
                            "Convex Volume Planes (MCVP): {} entries",
                            root.convex_volume_planes.len()
                        );
                    }
                    if !root.group_file_ids.is_empty() {
                        println!(
                            "Group File IDs (GFID): {} entries",
                            root.group_file_ids.len()
                        );
                    }
                }
                wow_wmo::ParsedWmo::Group(group) => {
                    if !group.vertex_positions.is_empty() {
                        println!(
                            "\nVertex Positions (MOVT): {} vertices",
                            group.vertex_positions.len()
                        );
                    }
                    if !group.vertex_indices.is_empty() {
                        println!(
                            "Vertex Indices (MOVI): {} indices",
                            group.vertex_indices.len()
                        );
                    }
                    if !group.render_batches.is_empty() {
                        println!(
                            "Render Batches (MOBA): {} batches",
                            group.render_batches.len()
                        );
                    }
                    if group.liquid_header.is_some() {
                        println!("Liquid Data (MLIQ): Present");
                    }
                    if !group.triangle_strip_indices.is_empty() {
                        println!(
                            "Triangle Strip Indices (MORI): {} indices",
                            group.triangle_strip_indices.len()
                        );
                    }
                    if !group.additional_render_batches.is_empty() {
                        println!(
                            "Additional Render Batches (MORB): {} batches",
                            group.additional_render_batches.len()
                        );
                    }
                    if !group.tangent_arrays.is_empty() {
                        println!(
                            "Tangent Arrays (MOTA): {} tangents",
                            group.tangent_arrays.len()
                        );
                    }
                    if !group.shadow_batches.is_empty() {
                        println!(
                            "Shadow Batches (MOBS): {} batches",
                            group.shadow_batches.len()
                        );
                    }
                }
            }
        }

        // Additional tables for detailed output could go here
        // For now, the extended data shown above is sufficient
    }

    Ok(())
}

fn validate(path: &str, _show_warnings: bool, _detailed: bool) -> Result<()> {
    // TODO: Implement validation with new parser
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    // For now, just check if it parses successfully
    match parse_wmo_with_metadata(&mut reader) {
        Ok(_) => {
            println!("✓ WMO file is valid (can be parsed)");
            Ok(())
        }
        Err(e) => {
            println!("✗ WMO file is invalid: {}", e);
            std::process::exit(1);
        }
    }
}

fn convert(_input_path: &str, _output_path: &str, _target_version: u32) -> Result<()> {
    // TODO: Implement conversion with new parser
    anyhow::bail!("Convert command needs updating for new parser");
}

fn list(_path: &str, _component: &str) -> Result<()> {
    // TODO: Update for new parser
    anyhow::bail!("List command needs updating for new parser");
    /*
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    // Use the new parser
    let parse_result = parse_wmo_with_metadata(&mut reader)
        .with_context(|| format!("Failed to parse WMO file: {}", path.display()))?;

    match component.to_lowercase().as_str() {
        "all" => {
            // Show summary
            println!("WMO Components Summary");
            println!("======================");
            println!("Groups: {}", wmo.groups.len());
            println!("Materials: {}", wmo.materials.len());
            println!("Doodads: {}", wmo.doodad_defs.len());
            println!("Portals: {}", wmo.portals.len());
            println!("Lights: {}", wmo.lights.len());
        }
        "groups" => {
            if wmo.groups.is_empty() {
                println!("No groups found");
            } else {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
                table.set_titles(Row::new(vec![
                    Cell::new("Index"),
                    Cell::new("Name"),
                    Cell::new("Flags"),
                ]));

                for (i, group) in wmo.groups.iter().enumerate() {
                    table.add_row(Row::new(vec![
                        Cell::new(&i.to_string()),
                        Cell::new(&group.name),
                        Cell::new(&format!("{:?}", group.flags)),
                    ]));
                }

                table.printstd();
            }
        }
        "materials" => {
            if wmo.materials.is_empty() {
                println!("No materials found");
            } else {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
                table.set_titles(Row::new(vec![
                    Cell::new("Index"),
                    Cell::new("Shader"),
                    Cell::new("Blend Mode"),
                    Cell::new("Texture 1"),
                ]));

                for (i, mat) in wmo.materials.iter().enumerate() {
                    table.add_row(Row::new(vec![
                        Cell::new(&i.to_string()),
                        Cell::new(&mat.shader.to_string()),
                        Cell::new(&mat.blend_mode.to_string()),
                        Cell::new(&mat.texture1.to_string()),
                    ]));
                }

                table.printstd();
            }
        }
        "doodads" => {
            if wmo.doodad_defs.is_empty() {
                println!("No doodads found");
            } else {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
                table.set_titles(Row::new(vec![
                    Cell::new("Index"),
                    Cell::new("Name Offset"),
                    Cell::new("Position"),
                    Cell::new("Scale"),
                ]));

                for (i, doodad) in wmo.doodad_defs.iter().enumerate() {
                    table.add_row(Row::new(vec![
                        Cell::new(&i.to_string()),
                        Cell::new(&doodad.name_offset.to_string()),
                        Cell::new(&format!(
                            "({:.2}, {:.2}, {:.2})",
                            doodad.position.x, doodad.position.y, doodad.position.z
                        )),
                        Cell::new(&format!("{:.2}", doodad.scale)),
                    ]));
                }

                table.printstd();
            }
        }
        "portals" => {
            if wmo.portals.is_empty() {
                println!("No portals found");
            } else {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
                table.set_titles(Row::new(vec![
                    Cell::new("Index"),
                    Cell::new("Vertices"),
                    Cell::new("Normal"),
                ]));

                for (i, portal) in wmo.portals.iter().enumerate() {
                    table.add_row(Row::new(vec![
                        Cell::new(&i.to_string()),
                        Cell::new(&portal.vertices.len().to_string()),
                        Cell::new(&format!(
                            "({:.2}, {:.2}, {:.2})",
                            portal.normal.x, portal.normal.y, portal.normal.z
                        )),
                    ]));
                }

                table.printstd();
            }
        }
        "lights" => {
            if wmo.lights.is_empty() {
                println!("No lights found");
            } else {
                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
                table.set_titles(Row::new(vec![
                    Cell::new("Index"),
                    Cell::new("Type"),
                    Cell::new("Color"),
                    Cell::new("Intensity"),
                ]));

                for (i, light) in wmo.lights.iter().enumerate() {
                    table.add_row(Row::new(vec![
                        Cell::new(&i.to_string()),
                        Cell::new(&format!("{:?}", light.light_type)),
                        Cell::new(&format!(
                            "({:.2}, {:.2}, {:.2})",
                            light.color.r, light.color.g, light.color.b
                        )),
                        Cell::new(&format!("{:.2}", light.intensity)),
                    ]));
                }

                table.printstd();
            }
        }
        _ => {
            anyhow::bail!(
                "Unknown component type: {}. Valid options are: all, groups, materials, doodads, portals, lights",
                component
            );
        }
    }

    Ok(())
    */
}

fn tree(
    path: &str,
    depth: Option<usize>,
    show_refs: bool,
    no_color: bool,
    no_metadata: bool,
    compact: bool,
    detailed: bool,
) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    // Parse using our new API
    let parse_result = wow_wmo::parse_wmo_with_metadata(&mut reader)
        .with_context(|| format!("Failed to parse WMO file: {}", path.display()))?;

    let options = TreeOptions {
        max_depth: depth,
        show_external_refs: show_refs,
        no_color,
        show_metadata: !no_metadata,
        compact,
        verbose: detailed,
    };

    // Build tree structure based on WMO type
    match &parse_result.wmo {
        wow_wmo::ParsedWmo::Root(root_wmo) => display_root_tree(root_wmo, &parse_result, &options),
        wow_wmo::ParsedWmo::Group(group_wmo) => {
            display_group_tree(group_wmo, &parse_result, &options)
        }
    }
}

fn display_root_tree(
    wmo: &wow_wmo::root_parser::WmoRoot,
    parse_result: &wow_wmo::ParseResult,
    options: &TreeOptions,
) -> Result<()> {
    // Build tree structure for Root WMO
    let mut root = TreeNode::new(format!("WMO Root (v{})", wmo.version), NodeType::Root);

    // Header info from metadata
    if let Some(metadata) = parse_result.metadata() {
        let mut chunks_node = TreeNode::new(
            format!("Chunks ({})", metadata.total_chunks()),
            NodeType::Header,
        );
        if metadata.has_unknown_chunks() {
            chunks_node
                .metadata
                .insert("unknown".to_string(), metadata.unknown_count().to_string());
        }
        if metadata.has_malformed_chunks() {
            chunks_node.metadata.insert(
                "malformed".to_string(),
                metadata.malformed_count().to_string(),
            );
        }
        root.children.push(chunks_node);
    }

    // Texture Names (MOTX chunk)
    if !wmo.textures.is_empty() {
        let mut textures_node = TreeNode::new(
            format!("Texture Names (MOTX) [{}]", wmo.textures.len()),
            NodeType::Directory,
        );

        if !options.compact {
            for (i, texture) in wmo.textures.iter().take(5).enumerate() {
                let tex_node = TreeNode::new(format!("Texture {}: {}", i, texture), NodeType::File);
                textures_node.children.push(tex_node);
            }

            if wmo.textures.len() > 5 {
                textures_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.textures.len() - 5),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(textures_node);
    }

    // Materials
    if !wmo.materials.is_empty() {
        let mut materials_node = TreeNode::new(
            format!("Materials (MOMT) [{}]", wmo.materials.len()),
            NodeType::Directory,
        );

        if !options.compact {
            let display_count = if options.verbose {
                wmo.materials.len()
            } else {
                5
            };
            for (i, mat) in wmo.materials.iter().take(display_count).enumerate() {
                let mut mat_node = TreeNode::new(format!("Material {}", i), NodeType::Data);

                if options.verbose {
                    mat_node
                        .metadata
                        .insert("flags".to_string(), format!("0x{:08X}", mat.flags));
                    mat_node
                        .metadata
                        .insert("shader".to_string(), format!("0x{:08X}", mat.shader));
                    mat_node.metadata.insert(
                        "blend_mode".to_string(),
                        format!("0x{:08X}", mat.blend_mode),
                    );
                    mat_node
                        .metadata
                        .insert("texture_1".to_string(), mat.texture_1.to_string());
                    mat_node
                        .metadata
                        .insert("texture_2".to_string(), mat.texture_2.to_string());
                    mat_node
                        .metadata
                        .insert("texture_3".to_string(), mat.texture_3.to_string());
                }

                materials_node.children.push(mat_node);
            }

            if !options.verbose && wmo.materials.len() > 5 {
                materials_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.materials.len() - 5),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(materials_node);
    }

    // Group File IDs (GFID chunk - modern WoW)
    if !wmo.group_file_ids.is_empty() {
        let mut gfid_node = TreeNode::new(
            format!("Group File IDs (GFID) [{}]", wmo.group_file_ids.len()),
            NodeType::Directory,
        );

        if !options.compact {
            for (i, file_id) in wmo.group_file_ids.iter().take(5).enumerate() {
                let id_node = TreeNode::new(format!("Group {}: ID {}", i, file_id), NodeType::Data);
                gfid_node.children.push(id_node);
            }

            if wmo.group_file_ids.len() > 5 {
                gfid_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.group_file_ids.len() - 5),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(gfid_node);
    }

    // Group Names and Group Info
    if !wmo.group_names.is_empty() {
        let mut groups_node = TreeNode::new(
            format!("Group Names (MOGN) [{}]", wmo.group_names.len()),
            NodeType::Directory,
        );

        if !options.compact {
            for (i, name) in wmo.group_names.iter().take(5).enumerate() {
                let group_node = TreeNode::new(format!("Group {}: {}", i, name), NodeType::File);
                groups_node.children.push(group_node);
            }

            if wmo.group_names.len() > 5 {
                groups_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.group_names.len() - 5),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(groups_node);
    }

    // Group Info
    if !wmo.group_info.is_empty() {
        let groups_info_node = TreeNode::new(
            format!("Group Info (MOGI) [{}]", wmo.group_info.len()),
            NodeType::Directory,
        );
        root.children.push(groups_info_node);
    }

    // Doodad Sets
    if !wmo.doodad_sets.is_empty() {
        let mut doodad_sets_node = TreeNode::new(
            format!("Doodad Sets (MODS) [{}]", wmo.doodad_sets.len()),
            NodeType::Table,
        );

        if !options.compact {
            for (i, set) in wmo.doodad_sets.iter().enumerate() {
                let name_str = String::from_utf8_lossy(&set.name)
                    .trim_end_matches('\0')
                    .to_string();
                let mut set_node =
                    TreeNode::new(format!("Set {}: {}", i, name_str), NodeType::Data);
                set_node
                    .metadata
                    .insert("count".to_string(), format!("{} doodads", set.count));

                if options.verbose {
                    set_node
                        .metadata
                        .insert("start_index".to_string(), set.start_index.to_string());
                    set_node
                        .metadata
                        .insert("padding".to_string(), format!("0x{:08X}", set.padding));
                }

                doodad_sets_node.children.push(set_node);
            }
        }

        root.children.push(doodad_sets_node);
    }

    // Portals (only have count, not list)
    if wmo.n_portals > 0 {
        root.children.push(TreeNode::new(
            format!("Portals [{}]", wmo.n_portals),
            NodeType::Table,
        ));
    }

    // Lights
    if !wmo.lights.is_empty() {
        root.children.push(TreeNode::new(
            format!("Lights ({})", wmo.lights.len()),
            NodeType::Table,
        ));
    }

    // Doodad Definitions
    if !wmo.doodad_defs.is_empty() {
        root.children.push(TreeNode::new(
            format!("Doodad Definitions (MODD) [{}]", wmo.doodad_defs.len()),
            NodeType::Table,
        ));
    }

    // Doodad Names (MODN)
    if !wmo.doodad_names.is_empty() {
        let mut doodad_names_node = TreeNode::new(
            format!("Doodad Names (MODN) [{}]", wmo.doodad_names.len()),
            NodeType::Directory,
        );

        if !options.compact {
            for (i, name) in wmo.doodad_names.iter().take(5).enumerate() {
                let name_node = TreeNode::new(format!("Doodad {}: {}", i, name), NodeType::File);
                doodad_names_node.children.push(name_node);
            }

            if wmo.doodad_names.len() > 5 {
                doodad_names_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.doodad_names.len() - 5),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(doodad_names_node);
    }

    // Skybox (MOSB)
    if let Some(skybox) = &wmo.skybox {
        root.children.push(TreeNode::new(
            format!("Skybox (MOSB): {}", skybox),
            NodeType::File,
        ));
    }

    // Portal Vertices (MOPV)
    if !wmo.portal_vertices.is_empty() {
        root.children.push(TreeNode::new(
            format!("Portal Vertices (MOPV) [{}]", wmo.portal_vertices.len()),
            NodeType::Table,
        ));
    }

    // Portal References (MOPR)
    if !wmo.portal_refs.is_empty() {
        root.children.push(TreeNode::new(
            format!("Portal References (MOPR) [{}]", wmo.portal_refs.len()),
            NodeType::Table,
        ));
    }

    // Visible Block Vertices (MOVV)
    if !wmo.visible_vertices.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "Visible Block Vertices (MOVV) [{}]",
                wmo.visible_vertices.len()
            ),
            NodeType::Table,
        ));
    }

    // Visible Block List (MOVB)
    if !wmo.visible_blocks.is_empty() {
        root.children.push(TreeNode::new(
            format!("Visible Block List (MOVB) [{}]", wmo.visible_blocks.len()),
            NodeType::Table,
        ));
    }

    // UV Transformations (MOUV - Legion+)
    if !wmo.uv_transforms.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "UV Transformations (MOUV) [{}] (Legion+)",
                wmo.uv_transforms.len()
            ),
            NodeType::Table,
        ));
    }

    // Portal Extra Information (MOPE - WarWithin+)
    if !wmo.portal_extras.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "Portal Extras (MOPE) [{}] (WarWithin+)",
                wmo.portal_extras.len()
            ),
            NodeType::Table,
        ));
    }

    // Light Extensions (MOLV - Shadowlands+)
    if !wmo.light_extensions.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "Light Extensions (MOLV) [{}] (Shadowlands+)",
                wmo.light_extensions.len()
            ),
            NodeType::Table,
        ));
    }

    // Doodad File IDs (MODI - Battle for Azeroth+)
    if !wmo.doodad_ids.is_empty() {
        root.children.push(TreeNode::new(
            format!("Doodad File IDs (MODI) [{}] (BfA+)", wmo.doodad_ids.len()),
            NodeType::Table,
        ));
    }

    // New Materials (MOM3 - WarWithin+)
    if !wmo.new_materials.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "New Materials (MOM3) [{}] (WarWithin+)",
                wmo.new_materials.len()
            ),
            NodeType::Table,
        ));
    }

    let output = crate::utils::tree::render_tree(&root, options);
    print!("{output}");

    Ok(())
}

fn display_group_tree(
    wmo: &wow_wmo::group_parser::WmoGroup,
    parse_result: &wow_wmo::ParseResult,
    options: &TreeOptions,
) -> Result<()> {
    // Build tree structure for Group WMO
    let mut root = TreeNode::new(
        format!("WMO Group {} (v{})", wmo.group_index, wmo.version),
        NodeType::Root,
    );

    // Metadata from parser
    if let Some(metadata) = parse_result.metadata() {
        let mut chunks_node = TreeNode::new(
            format!("Chunks ({})", metadata.total_chunks()),
            NodeType::Header,
        );
        if metadata.has_unknown_chunks() {
            chunks_node
                .metadata
                .insert("unknown".to_string(), metadata.unknown_count().to_string());
        }
        if metadata.has_malformed_chunks() {
            chunks_node.metadata.insert(
                "malformed".to_string(),
                metadata.malformed_count().to_string(),
            );
        }
        root.children.push(chunks_node);
    }

    // Geometry information
    let mut geometry_node = TreeNode::new("Geometry".to_string(), NodeType::Directory);
    geometry_node
        .metadata
        .insert("triangles".to_string(), wmo.n_triangles.to_string());
    geometry_node
        .metadata
        .insert("vertices".to_string(), wmo.n_vertices.to_string());
    root.children.push(geometry_node);

    // Material Info per Triangle (MOPY)
    if !wmo.material_info.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "Material Info per Triangle (MOPY) [{}]",
                wmo.material_info.len()
            ),
            NodeType::Table,
        ));
    }

    // Vertex data
    if !wmo.vertex_positions.is_empty() {
        root.children.push(TreeNode::new(
            format!("Vertex Positions (MOVT) [{}]", wmo.vertex_positions.len()),
            NodeType::Table,
        ));
    }

    if !wmo.vertex_indices.is_empty() {
        root.children.push(TreeNode::new(
            format!("Vertex Indices (MOVI) [{}]", wmo.vertex_indices.len()),
            NodeType::Table,
        ));
    }

    if !wmo.vertex_normals.is_empty() {
        root.children.push(TreeNode::new(
            format!("Vertex Normals (MONR) [{}]", wmo.vertex_normals.len()),
            NodeType::Table,
        ));
    }

    if !wmo.texture_coords.is_empty() {
        root.children.push(TreeNode::new(
            format!("Texture Coords (MOTV) [{}]", wmo.texture_coords.len()),
            NodeType::Table,
        ));
    }

    // Render batches
    if !wmo.render_batches.is_empty() {
        let mut batches_node = TreeNode::new(
            format!("Render Batches (MOBA) [{}]", wmo.render_batches.len()),
            NodeType::Directory,
        );

        if !options.compact {
            for (i, batch) in wmo.render_batches.iter().take(3).enumerate() {
                let mut batch_node = TreeNode::new(format!("Batch {}", i), NodeType::Data);
                batch_node
                    .metadata
                    .insert("material".to_string(), batch.material_id.to_string());
                batches_node.children.push(batch_node);
            }

            if wmo.render_batches.len() > 3 {
                batches_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.render_batches.len() - 3),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(batches_node);
    }

    // Triangle Strip Indices (MORI chunk - modern WoW)
    if !wmo.triangle_strip_indices.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "Triangle Strip Indices (MORI) [{}]",
                wmo.triangle_strip_indices.len()
            ),
            NodeType::Table,
        ));
    }

    // Additional Render Batches (MORB chunk - extended batching)
    if !wmo.additional_render_batches.is_empty() {
        let mut morb_node = TreeNode::new(
            format!(
                "Additional Render Batches (MORB) [{}]",
                wmo.additional_render_batches.len()
            ),
            NodeType::Directory,
        );

        if !options.compact {
            for (i, batch) in wmo.additional_render_batches.iter().take(3).enumerate() {
                let mut batch_node =
                    TreeNode::new(format!("Additional Batch {}", i), NodeType::Data);
                batch_node
                    .metadata
                    .insert("material".to_string(), batch.material_id.to_string());
                batch_node.metadata.insert(
                    "indices".to_string(),
                    if batch.index_count > 0 {
                        let end_index = batch
                            .start_index
                            .saturating_add(batch.index_count)
                            .saturating_sub(1);
                        format!(
                            "{}-{} ({})",
                            batch.start_index, end_index, batch.index_count
                        )
                    } else {
                        format!("empty ({})", batch.index_count)
                    },
                );
                morb_node.children.push(batch_node);
            }

            if wmo.additional_render_batches.len() > 3 {
                morb_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.additional_render_batches.len() - 3),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(morb_node);
    }

    // Tangent Arrays (MOTA chunk - normal mapping)
    if !wmo.tangent_arrays.is_empty() {
        root.children.push(TreeNode::new(
            format!("Tangent Arrays (MOTA) [{}]", wmo.tangent_arrays.len()),
            NodeType::Table,
        ));
    }

    // Shadow Batches (MOBS chunk - shadow rendering)
    if !wmo.shadow_batches.is_empty() {
        let mut mobs_node = TreeNode::new(
            format!("Shadow Batches (MOBS) [{}]", wmo.shadow_batches.len()),
            NodeType::Directory,
        );

        if !options.compact {
            for (i, batch) in wmo.shadow_batches.iter().take(3).enumerate() {
                let mut batch_node = TreeNode::new(format!("Shadow Batch {}", i), NodeType::Data);
                batch_node
                    .metadata
                    .insert("material".to_string(), batch.material_id.to_string());
                batch_node.metadata.insert(
                    "indices".to_string(),
                    if batch.index_count > 0 {
                        let end_index = batch
                            .start_index
                            .saturating_add(batch.index_count as u16)
                            .saturating_sub(1);
                        format!(
                            "{}-{} ({})",
                            batch.start_index, end_index, batch.index_count
                        )
                    } else {
                        format!("empty/negative ({})", batch.index_count)
                    },
                );
                mobs_node.children.push(batch_node);
            }

            if wmo.shadow_batches.len() > 3 {
                mobs_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.shadow_batches.len() - 3),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(mobs_node);
    }

    // Light References (MOLR)
    if !wmo.light_refs.is_empty() {
        root.children.push(TreeNode::new(
            format!("Light References (MOLR) [{}]", wmo.light_refs.len()),
            NodeType::Table,
        ));
    }

    // Doodad References (MODR)
    if !wmo.doodad_refs.is_empty() {
        root.children.push(TreeNode::new(
            format!("Doodad References (MODR) [{}]", wmo.doodad_refs.len()),
            NodeType::Table,
        ));
    }

    // BSP Tree Nodes (MOBN)
    if !wmo.bsp_nodes.is_empty() {
        root.children.push(TreeNode::new(
            format!("BSP Tree Nodes (MOBN) [{}]", wmo.bsp_nodes.len()),
            NodeType::Table,
        ));
    }

    // BSP Face Indices (MOBR)
    if !wmo.bsp_face_indices.is_empty() {
        root.children.push(TreeNode::new(
            format!("BSP Face Indices (MOBR) [{}]", wmo.bsp_face_indices.len()),
            NodeType::Table,
        ));
    }

    // Extended Vertex Indices (MOVX - Shadowlands+)
    if !wmo.extended_vertex_indices.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "Extended Vertex Indices (MOVX) [{}] (Shadowlands+)",
                wmo.extended_vertex_indices.len()
            ),
            NodeType::Table,
        ));
    }

    // Query Face Start (MOGX - Dragonflight+)
    if let Some(query_start) = wmo.query_face_start {
        root.children.push(TreeNode::new(
            format!("Query Face Start (MOGX): {} (Dragonflight+)", query_start),
            NodeType::Data,
        ));
    }

    // Query Faces (MOQG - Dragonflight+)
    if !wmo.query_faces.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "Query Faces (MOQG) [{}] (Dragonflight+)",
                wmo.query_faces.len()
            ),
            NodeType::Table,
        ));
    }

    // Extended Materials (MPY2 - Dragonflight+)
    if !wmo.extended_materials.is_empty() {
        root.children.push(TreeNode::new(
            format!(
                "Extended Materials (MPY2) [{}] (Dragonflight+)",
                wmo.extended_materials.len()
            ),
            NodeType::Table,
        ));
    }

    // Additional data
    if !wmo.vertex_colors.is_empty() {
        root.children.push(TreeNode::new(
            format!("Vertex Colors (MOCV) [{}]", wmo.vertex_colors.len()),
            NodeType::Table,
        ));
    }

    if wmo.liquid_header.is_some() {
        root.children.push(TreeNode::new(
            "Liquid Data (MLIQ) [Present]".to_string(),
            NodeType::Table,
        ));
    }

    let output = crate::utils::tree::render_tree(&root, options);
    print!("{}", output);

    Ok(())
}
