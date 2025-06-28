//! WMO world map object command implementations

use crate::utils::tree::{NodeType, RefType, TreeNode, TreeOptions};
use anyhow::{Context, Result};
use clap::Subcommand;
use prettytable::{Cell, Row, Table, format};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wow_wmo::{WmoVersion, convert_wmo, parse_wmo, validate_wmo, validate_wmo_detailed};

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
        } => tree(&file, depth, show_refs, no_color, no_metadata, compact),
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

    let wmo = parse_wmo(&mut reader)
        .with_context(|| format!("Failed to parse WMO file: {}", path.display()))?;

    println!("WMO Information");
    println!("===============");
    println!();

    // Basic info
    println!(
        "Version: {} ({})",
        wmo.version.to_raw(),
        wmo.version.expansion_name()
    );
    println!("Materials: {}", wmo.materials.len());
    println!("Groups: {}", wmo.groups.len());
    println!("Portals: {}", wmo.portals.len());
    println!("Lights: {}", wmo.lights.len());
    println!("Doodad Definitions: {}", wmo.doodad_defs.len());
    println!("Doodad Sets: {}", wmo.doodad_sets.len());

    // Bounding box
    println!("\nBounding Box:");
    println!(
        "  Min: ({:.2}, {:.2}, {:.2})",
        wmo.bounding_box.min.x, wmo.bounding_box.min.y, wmo.bounding_box.min.z
    );
    println!(
        "  Max: ({:.2}, {:.2}, {:.2})",
        wmo.bounding_box.max.x, wmo.bounding_box.max.y, wmo.bounding_box.max.z
    );

    // Header flags
    println!("\nHeader Flags: {:?}", wmo.header.flags);

    if let Some(ref skybox) = wmo.skybox {
        println!("Skybox: {skybox}");
    }

    if detailed {
        // Doodad sets
        if !wmo.doodad_sets.is_empty() {
            println!("\nDoodad Sets:");
            let mut table = Table::new();
            table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
            table.set_titles(Row::new(vec![
                Cell::new("Index"),
                Cell::new("Name"),
                Cell::new("Doodads"),
                Cell::new("Start Index"),
            ]));

            for (i, set) in wmo.doodad_sets.iter().enumerate() {
                table.add_row(Row::new(vec![
                    Cell::new(&i.to_string()),
                    Cell::new(&set.name),
                    Cell::new(&set.n_doodads.to_string()),
                    Cell::new(&set.start_doodad.to_string()),
                ]));
            }

            table.printstd();
        }

        // Groups
        if !wmo.groups.is_empty() {
            println!("\nGroups:");
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

    Ok(())
}

fn validate(path: &str, show_warnings: bool, detailed: bool) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    if detailed {
        let report = validate_wmo_detailed(&mut reader)
            .with_context(|| format!("Failed to validate WMO file: {}", path.display()))?;

        println!("WMO Validation Report");
        println!("=====================");
        println!();

        if report.errors.is_empty() && (!show_warnings || report.warnings.is_empty()) {
            println!("✓ WMO file is valid");
        } else {
            if !report.errors.is_empty() {
                println!("Errors:");
                for error in &report.errors {
                    println!("  ✗ {error}");
                }
            }

            if show_warnings && !report.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &report.warnings {
                    println!("  ⚠ {warning}");
                }
            }
        }
    } else {
        let is_valid = validate_wmo(&mut reader)
            .with_context(|| format!("Failed to validate WMO file: {}", path.display()))?;

        if is_valid {
            println!("✓ WMO file is valid");
        } else {
            println!("✗ WMO file is invalid");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn convert(input_path: &str, output_path: &str, target_version: u32) -> Result<()> {
    let input_path = Path::new(input_path);
    let output_path = Path::new(output_path);

    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", input_path.display());
    }

    let version = WmoVersion::from_raw(target_version)
        .ok_or_else(|| anyhow::anyhow!("Invalid WMO version: {}", target_version))?;

    println!("Converting WMO file...");
    println!("  Input: {}", input_path.display());
    println!("  Output: {}", output_path.display());
    println!(
        "  Target version: {} ({})",
        version.to_raw(),
        version.expansion_name()
    );

    let input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path.display()))?;

    let output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;

    let mut reader = BufReader::new(input_file);
    let mut writer = std::io::BufWriter::new(output_file);

    convert_wmo(&mut reader, &mut writer, version).with_context(|| "Failed to convert WMO file")?;

    println!("✓ WMO file converted successfully");

    Ok(())
}

fn list(path: &str, component: &str) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    let wmo = parse_wmo(&mut reader)
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
}

fn tree(
    path: &str,
    depth: Option<usize>,
    show_refs: bool,
    no_color: bool,
    no_metadata: bool,
    compact: bool,
) -> Result<()> {
    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);

    let wmo = parse_wmo(&mut reader)
        .with_context(|| format!("Failed to parse WMO file: {}", path.display()))?;

    let options = TreeOptions {
        max_depth: depth,
        show_external_refs: show_refs,
        no_color,
        show_metadata: !no_metadata,
        compact,
    };

    // Build tree structure for WMO
    let mut root = TreeNode::new(
        format!("WMO Root (v{})", wmo.version.to_raw()),
        NodeType::Root,
    );
    root.metadata.insert(
        "expansion".to_string(),
        wmo.version.expansion_name().to_string(),
    );

    // Header
    let mut header_node = TreeNode::new("Header".to_string(), NodeType::Header);
    header_node
        .metadata
        .insert("flags".to_string(), format!("{:?}", wmo.header.flags));
    root.children.push(header_node);

    // Materials
    if !wmo.materials.is_empty() {
        let mut materials_node = TreeNode::new(
            format!("Materials ({})", wmo.materials.len()),
            NodeType::Directory,
        );

        if !compact {
            for (i, mat) in wmo.materials.iter().take(5).enumerate() {
                let mut mat_node = TreeNode::new(format!("Material {i}"), NodeType::Data);
                mat_node
                    .metadata
                    .insert("shader".to_string(), mat.shader.to_string());
                materials_node.children.push(mat_node);
            }

            if wmo.materials.len() > 5 {
                materials_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.materials.len() - 5),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(materials_node);
    }

    // Groups
    if !wmo.groups.is_empty() {
        let mut groups_node = TreeNode::new(
            format!("Groups ({})", wmo.groups.len()),
            NodeType::Directory,
        );

        if !compact {
            for (i, group) in wmo.groups.iter().take(5).enumerate() {
                let mut group_node =
                    TreeNode::new(format!("Group {}: {}", i, group.name), NodeType::File);
                group_node
                    .metadata
                    .insert("flags".to_string(), format!("{:?}", group.flags));
                groups_node.children.push(group_node);
            }

            if wmo.groups.len() > 5 {
                groups_node.children.push(TreeNode::new(
                    format!("... {} more", wmo.groups.len() - 5),
                    NodeType::Data,
                ));
            }
        }

        root.children.push(groups_node);
    }

    // Doodad Sets
    if !wmo.doodad_sets.is_empty() {
        let mut doodad_sets_node = TreeNode::new(
            format!("Doodad Sets ({})", wmo.doodad_sets.len()),
            NodeType::Table,
        );

        if !compact {
            for (i, set) in wmo.doodad_sets.iter().enumerate() {
                let mut set_node =
                    TreeNode::new(format!("Set {}: {}", i, set.name), NodeType::Data);
                set_node
                    .metadata
                    .insert("count".to_string(), format!("{} doodads", set.n_doodads));
                doodad_sets_node.children.push(set_node);
            }
        }

        root.children.push(doodad_sets_node);
    }

    // Portals
    if !wmo.portals.is_empty() {
        root.children.push(TreeNode::new(
            format!("Portals ({})", wmo.portals.len()),
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
            format!("Doodad Definitions ({})", wmo.doodad_defs.len()),
            NodeType::Table,
        ));
    }

    // Textures
    if !wmo.textures.is_empty() && show_refs {
        let mut textures_node = TreeNode::new(
            format!("Textures ({})", wmo.textures.len()),
            NodeType::Directory,
        );

        if !compact {
            for (i, name) in wmo.textures.iter().take(5).enumerate() {
                let mut tex_node = TreeNode::new(format!("Texture {i}"), NodeType::File);
                tex_node
                    .external_refs
                    .push(crate::utils::tree::ExternalRef {
                        path: name.clone(),
                        ref_type: RefType::Texture,
                        exists: None,
                    });
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

    let output = crate::utils::tree::render_tree(&root, &options);
    print!("{output}");

    Ok(())
}
