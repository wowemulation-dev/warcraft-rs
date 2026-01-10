//! M2 model file command implementations

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;

use wow_blp::parser::load_blp;
use wow_m2::{
    AnimFile, M2Converter, M2Model, M2Version, SkinFile,
    skin::{OldSkinHeader, SkinG, SkinHeaderT},
};

use crate::utils::{NodeType, TreeNode, TreeOptions, render_tree};

#[derive(Subcommand)]
pub enum M2Commands {
    /// Display information about an M2 model file
    Info {
        /// Path to the M2 file
        file: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Validate an M2 model file
    Validate {
        /// Path to the M2 file
        file: PathBuf,

        /// Show all warnings (not just errors)
        #[arg(short, long)]
        warnings: bool,
    },

    /// Convert an M2 model to a different version
    Convert {
        /// Input M2 file
        input: PathBuf,

        /// Output M2 file
        output: PathBuf,

        /// Target version (e.g., "3.3.5a", "WotLK", "MoP")
        #[arg(long)]
        version: String,
    },

    /// Display M2 file structure as a tree
    Tree {
        /// Path to the M2 file
        file: PathBuf,

        /// Maximum depth to display
        #[arg(short, long, default_value = "5")]
        depth: usize,

        /// Include size information
        #[arg(short, long)]
        size: bool,

        /// Include references
        #[arg(short, long)]
        refs: bool,
    },

    /// Display information about a Skin file
    SkinInfo {
        /// Path to the Skin file
        file: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Parse old format
        #[arg(short, long)]
        old_format: bool,
    },

    /// Convert a Skin file to a different version
    SkinConvert {
        /// Input Skin file
        input: PathBuf,

        /// Output Skin file
        output: PathBuf,

        /// Target version (e.g., "3.3.5a", "WotLK", "MoP")
        #[arg(long)]
        version: String,
    },

    /// Display information about an ANIM file
    AnimInfo {
        /// Path to the ANIM file
        file: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Convert an ANIM file to a different version
    AnimConvert {
        /// Input ANIM file
        input: PathBuf,

        /// Output ANIM file
        output: PathBuf,

        /// Target version (e.g., "3.3.5a", "WotLK", "MoP")
        #[arg(long)]
        version: String,
    },

    /// Display information about a BLP texture file
    BlpInfo {
        /// Path to the BLP file
        file: PathBuf,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },
}

pub fn execute(cmd: M2Commands) -> Result<()> {
    match cmd {
        M2Commands::Info { file, detailed } => handle_info(file, detailed),
        M2Commands::Convert {
            input,
            output,
            version,
        } => handle_convert(input, output, version),
        M2Commands::Validate { file, warnings } => handle_validate(file, warnings),
        M2Commands::Tree {
            file,
            depth,
            size,
            refs,
        } => handle_tree(file, depth, size, refs),
        M2Commands::SkinInfo {
            file,
            detailed,
            old_format,
        } => {
            // Use auto-detection by default, with old_format as an override
            handle_skin_info_auto(file, detailed, old_format)
        }
        M2Commands::SkinConvert {
            input,
            output,
            version,
        } => handle_skin_convert(input, output, version),
        M2Commands::AnimInfo { file, detailed } => handle_anim_info(file, detailed),
        M2Commands::AnimConvert {
            input,
            output,
            version,
        } => handle_anim_convert(input, output, version),
        M2Commands::BlpInfo { file, detailed } => handle_blp_info(file, detailed),
    }
}

fn handle_info(path: PathBuf, detailed: bool) -> Result<()> {
    println!("Loading M2 model: {}", path.display());

    let m2_format = M2Model::load(&path)
        .with_context(|| format!("Failed to load M2 model from {}", path.display()))?;

    let model = m2_format.model();
    println!("\n=== M2 Model Information ===");

    // Display version information
    println!("Version: {}", model.header.version);
    if let Some(version) = model.header.version() {
        println!("Expansion: {:?}", version);
        println!(
            "Format: {}",
            if version >= M2Version::Legion {
                "Chunked (MD21)"
            } else {
                "Legacy (MD20)"
            }
        );
    } else {
        println!("Expansion: Unknown");
        println!("Format: Unknown");
    }

    // Display model name if available
    if let Some(ref name) = model.name {
        println!("Model Name: {}", name);
    }

    // Display basic counts
    println!("Vertices: {}", model.header.vertices.count);
    println!("Bones: {}", model.header.bones.count);
    println!("Animations: {}", model.header.animations.count);
    println!("Textures: {}", model.header.textures.count);

    // Display skin information
    if let Some(version) = model.header.version() {
        if version <= M2Version::TBC {
            println!("Skin Format: Embedded (Pre-WotLK)");
            println!("Skin Profiles: {}", model.header.views.count);
        } else {
            println!("Skin Format: External (WotLK+)");
            if let Some(count) = model.header.num_skin_profiles {
                println!("Skin Profiles: {}", count);
            }
        }
    }

    if detailed {
        println!("\n=== Detailed Information ===");
        println!("Global Sequences: {}", model.header.global_sequences.count);
        println!("Color Animations: {}", model.header.color_animations.count);
        println!(
            "Texture Animations: {}",
            model.header.texture_animations.count
        );
        println!("Key Bone Lookups: {}", model.header.key_bone_lookup.count);
        println!("Texture Units: {}", model.header.texture_units.count);

        // Bounding box information
        println!("\n--- Bounding Information ---");
        let bbox_min = model.header.bounding_box_min;
        let bbox_max = model.header.bounding_box_max;
        println!(
            "Bounding Box: [{:.2}, {:.2}, {:.2}] to [{:.2}, {:.2}, {:.2}]",
            bbox_min[0], bbox_min[1], bbox_min[2], bbox_max[0], bbox_max[1], bbox_max[2]
        );
        println!(
            "Bounding Sphere Radius: {:.2}",
            model.header.bounding_sphere_radius
        );

        // Collision information
        let col_min = model.header.collision_box_min;
        let col_max = model.header.collision_box_max;
        println!(
            "Collision Box: [{:.2}, {:.2}, {:.2}] to [{:.2}, {:.2}, {:.2}]",
            col_min[0], col_min[1], col_min[2], col_max[0], col_max[1], col_max[2]
        );
        println!(
            "Collision Sphere Radius: {:.2}",
            model.header.collision_sphere_radius
        );

        // Model flags
        println!("\n--- Model Flags ---");
        println!("Flags: {:?}", model.header.flags);
    }

    Ok(())
}

fn handle_convert(input: PathBuf, output: PathBuf, version_str: String) -> Result<()> {
    println!("Loading M2 model: {}", input.display());

    let m2_format = M2Model::load(&input)
        .with_context(|| format!("Failed to load M2 model from {}", input.display()))?;
    let model = m2_format.model();

    let target_version = M2Version::from_expansion_name(&version_str)
        .with_context(|| format!("Invalid target version: {version_str}"))?;

    println!("Converting to {target_version:?}");

    let converter = M2Converter::new();
    let converted = converter
        .convert(model, target_version)
        .with_context(|| "Failed to convert model")?;

    println!("Saving converted model to: {}", output.display());
    converted
        .save(&output)
        .with_context(|| format!("Failed to save converted model to {}", output.display()))?;

    println!("Conversion complete!");
    Ok(())
}

fn handle_validate(path: PathBuf, show_warnings: bool) -> Result<()> {
    println!("Validating M2 model: {}", path.display());

    let m2_format = M2Model::load(&path)
        .with_context(|| format!("Failed to load M2 model from {}", path.display()))?;
    let model = m2_format.model();

    // Validate the model
    match model.validate() {
        Ok(_) => {
            println!("✓ Model validation passed!");
        }
        Err(e) => {
            println!("❌ Model validation failed: {e}");
            if !show_warnings {
                println!("Use --warnings to show additional details");
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

fn handle_tree(path: PathBuf, max_depth: usize, show_size: bool, show_refs: bool) -> Result<()> {
    println!("Loading M2 model: {}", path.display());

    let m2_format = M2Model::load(&path)
        .with_context(|| format!("Failed to load M2 model from {}", path.display()))?;
    let model = m2_format.model();

    // Load the original file data for embedded skin parsing
    let original_m2_data = std::fs::read(&path).with_context(|| {
        format!(
            "Failed to read original M2 file data from {}",
            path.display()
        )
    })?;

    // Determine version and format details
    let version_info = if let Some(version) = model.header.version() {
        format!("{:?} ({})", version, model.header.version)
    } else {
        format!("Unknown ({})", model.header.version)
    };

    let format_type = match model.header.version() {
        Some(v) if v >= M2Version::Legion => "Chunked (MD21)",
        Some(_) => "Legacy (MD20)",
        None => "Unknown",
    };

    // Build the main model tree
    let mut root = TreeNode::new(
        format!(
            "M2 Model: {}",
            path.file_name().unwrap_or_default().to_string_lossy()
        ),
        NodeType::Root,
    )
    .with_metadata("version", &version_info)
    .with_metadata("format", format_type)
    .with_metadata("path", &path.to_string_lossy());

    // Add header information
    let mut header_node = TreeNode::new("Header".to_string(), NodeType::Header)
        .with_metadata("magic", &String::from_utf8_lossy(&model.header.magic))
        .with_metadata("version", &model.header.version.to_string())
        .with_metadata("flags", &format!("{:?}", model.header.flags));

    if let Some(ref name) = model.name {
        header_node = header_node.with_metadata("model_name", name);
    }

    // Add bounding information
    let bounds_node = TreeNode::new("Bounding Data".to_string(), NodeType::Data)
        .with_metadata(
            "bounding_box_min",
            &format!("{:.2?}", model.header.bounding_box_min),
        )
        .with_metadata(
            "bounding_box_max",
            &format!("{:.2?}", model.header.bounding_box_max),
        )
        .with_metadata(
            "bounding_radius",
            &format!("{:.2}", model.header.bounding_sphere_radius),
        )
        .with_metadata(
            "collision_box_min",
            &format!("{:.2?}", model.header.collision_box_min),
        )
        .with_metadata(
            "collision_box_max",
            &format!("{:.2?}", model.header.collision_box_max),
        )
        .with_metadata(
            "collision_radius",
            &format!("{:.2}", model.header.collision_sphere_radius),
        );

    header_node = header_node.add_child(bounds_node);
    root = root.add_child(header_node);

    // Add geometry information
    let mut geometry_node = TreeNode::new("Geometry".to_string(), NodeType::Data)
        .with_metadata("vertices", &model.header.vertices.count.to_string())
        .with_metadata("bones", &model.header.bones.count.to_string());

    if show_size {
        geometry_node = geometry_node
            .with_metadata(
                "vertex_data_offset",
                &format!("0x{:x}", model.header.vertices.offset),
            )
            .with_metadata(
                "bone_data_offset",
                &format!("0x{:x}", model.header.bones.offset),
            );
    }

    root = root.add_child(geometry_node);

    // Add animation information
    let mut anim_node = TreeNode::new("Animations".to_string(), NodeType::Data)
        .with_metadata("sequences", &model.header.animations.count.to_string())
        .with_metadata(
            "global_sequences",
            &model.header.global_sequences.count.to_string(),
        )
        .with_metadata(
            "key_bone_lookups",
            &model.header.key_bone_lookup.count.to_string(),
        );

    if show_size {
        anim_node = anim_node
            .with_metadata(
                "animation_offset",
                &format!("0x{:x}", model.header.animations.offset),
            )
            .with_metadata(
                "global_seq_offset",
                &format!("0x{:x}", model.header.global_sequences.offset),
            );
    }

    root = root.add_child(anim_node);

    // Add texture information
    let mut texture_node = TreeNode::new("Textures".to_string(), NodeType::Data)
        .with_metadata("textures", &model.header.textures.count.to_string())
        .with_metadata(
            "texture_units",
            &model.header.texture_units.count.to_string(),
        )
        .with_metadata(
            "texture_lookups",
            &model.header.texture_lookup_table.count.to_string(),
        )
        .with_metadata(
            "texture_animations",
            &model.header.texture_animations.count.to_string(),
        );

    if show_size {
        texture_node = texture_node
            .with_metadata(
                "texture_offset",
                &format!("0x{:x}", model.header.textures.offset),
            )
            .with_metadata(
                "texture_unit_offset",
                &format!("0x{:x}", model.header.texture_units.offset),
            );
    }

    root = root.add_child(texture_node);

    // Add version-specific skin information
    if let Some(version) = model.header.version() {
        let skin_node = if version <= M2Version::TBC {
            // Pre-WotLK: Embedded skins
            let mut node = TreeNode::new("Skin Data (Embedded)".to_string(), NodeType::Data)
                .with_metadata("format", "Embedded (Pre-WotLK)")
                .with_metadata("profiles", &model.header.views.count.to_string());

            if show_size {
                node = node.with_metadata(
                    "views_offset",
                    &format!("0x{:x}", model.header.views.offset),
                );
            }

            // Actually parse embedded skin profiles and show detailed information
            for i in 0..model.header.views.count {
                let skin_profile_name = format!("Skin Profile {}", i);
                let mut profile_node = TreeNode::new(skin_profile_name, NodeType::Data)
                    .with_metadata("index", &i.to_string());

                // Try to parse the embedded skin
                match model.parse_embedded_skin(&original_m2_data, i as usize) {
                    Ok(skin) => {
                        profile_node = profile_node
                            .with_metadata("status", "✅ Parsed successfully")
                            .with_metadata("indices_count", &skin.indices().len().to_string())
                            .with_metadata("triangles_count", &skin.triangles().len().to_string())
                            .with_metadata("submeshes_count", &skin.submeshes().len().to_string());

                        // Add submesh details
                        if !skin.submeshes().is_empty() {
                            for (submesh_idx, submesh) in skin.submeshes().iter().enumerate() {
                                let submesh_name = format!("Submesh {}", submesh_idx);
                                let submesh_node = TreeNode::new(submesh_name, NodeType::Data)
                                    .with_metadata("id", &submesh.id.to_string())
                                    .with_metadata(
                                        "vertex_start",
                                        &submesh.vertex_start.to_string(),
                                    )
                                    .with_metadata(
                                        "vertex_count",
                                        &submesh.vertex_count.to_string(),
                                    )
                                    .with_metadata(
                                        "triangle_start",
                                        &submesh.triangle_start.to_string(),
                                    )
                                    .with_metadata(
                                        "triangle_count",
                                        &submesh.triangle_count.to_string(),
                                    )
                                    .with_metadata("bone_count", &submesh.bone_count.to_string())
                                    .with_metadata("bone_start", &submesh.bone_start.to_string());

                                profile_node = profile_node.add_child(submesh_node);
                            }
                        }
                    }
                    Err(e) => {
                        profile_node = profile_node
                            .with_metadata("status", &format!("❌ Parse failed: {}", e))
                            .with_metadata(
                                "note",
                                if i == 0 {
                                    "Main skin should be valid"
                                } else {
                                    "Secondary skins often contain invalid data"
                                },
                            );
                    }
                }

                node = node.add_child(profile_node);
            }

            node
        } else {
            // WotLK+: External skins
            let mut node = TreeNode::new("Skin Data (External)".to_string(), NodeType::Data)
                .with_metadata("format", "External (.skin files)");

            if let Some(count) = model.header.num_skin_profiles {
                node = node.with_metadata("profiles", &count.to_string());
            }

            // Try to parse external skin files
            let base_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("model");
            let skin_count = model.header.num_skin_profiles.unwrap_or(1);

            for i in 0..skin_count {
                let skin_filename = format!("{}{:02}.skin", base_name, i);
                let mut skin_path = path.clone();
                skin_path.set_file_name(&skin_filename);

                let mut skin_ref_node = TreeNode::new(skin_filename.clone(), NodeType::Reference)
                    .with_metadata("type", "External Skin File")
                    .with_metadata("index", &i.to_string());

                // Try to load and parse the external skin file
                match wow_m2::SkinFile::load(&skin_path) {
                    Ok(skin) => {
                        skin_ref_node = skin_ref_node
                            .with_metadata("status", "✅ Loaded successfully")
                            .with_metadata("indices_count", &skin.indices().len().to_string())
                            .with_metadata("triangles_count", &skin.triangles().len().to_string())
                            .with_metadata("submeshes_count", &skin.submeshes().len().to_string());

                        // Add submesh details for external skins too
                        if !skin.submeshes().is_empty() {
                            for (submesh_idx, submesh) in skin.submeshes().iter().enumerate() {
                                let submesh_name = format!("Submesh {}", submesh_idx);
                                let submesh_node = TreeNode::new(submesh_name, NodeType::Data)
                                    .with_metadata("id", &submesh.id.to_string())
                                    .with_metadata(
                                        "vertex_start",
                                        &submesh.vertex_start.to_string(),
                                    )
                                    .with_metadata(
                                        "vertex_count",
                                        &submesh.vertex_count.to_string(),
                                    )
                                    .with_metadata(
                                        "triangle_start",
                                        &submesh.triangle_start.to_string(),
                                    )
                                    .with_metadata(
                                        "triangle_count",
                                        &submesh.triangle_count.to_string(),
                                    )
                                    .with_metadata("bone_count", &submesh.bone_count.to_string())
                                    .with_metadata("bone_start", &submesh.bone_start.to_string());

                                skin_ref_node = skin_ref_node.add_child(submesh_node);
                            }
                        }
                    }
                    Err(e) => {
                        skin_ref_node = skin_ref_node
                            .with_metadata("status", &format!("❌ Not found/failed: {}", e))
                            .with_metadata("path", skin_path.to_string_lossy().as_ref());
                    }
                }

                node = node.add_child(skin_ref_node);
            }

            node
        };

        root = root.add_child(skin_node);
    }

    // Add material and rendering information
    let mut material_node = TreeNode::new("Materials & Rendering".to_string(), NodeType::Data)
        .with_metadata("render_flags", &model.header.render_flags.count.to_string())
        .with_metadata(
            "color_animations",
            &model.header.color_animations.count.to_string(),
        );

    if show_size {
        material_node = material_node
            .with_metadata(
                "render_flags_offset",
                &format!("0x{:x}", model.header.render_flags.offset),
            )
            .with_metadata(
                "color_anim_offset",
                &format!("0x{:x}", model.header.color_animations.offset),
            );
    }

    root = root.add_child(material_node);

    // Add version-specific features
    if let Some(version) = model.header.version() {
        if version >= M2Version::Cataclysm {
            if let Some(ref combos) = model.header.texture_combiner_combos {
                let combo_node = TreeNode::new(
                    "Texture Combiner Combos (Cataclysm+)".to_string(),
                    NodeType::Data,
                )
                .with_metadata("count", &combos.count.to_string())
                .with_metadata("offset", &format!("0x{:x}", combos.offset));
                root = root.add_child(combo_node);
            }
        }

        if version >= M2Version::WotLK {
            // Add chunked format features for newer versions
            if version >= M2Version::Legion {
                let chunks_node = TreeNode::new(
                    "Chunked Format Features (Legion+)".to_string(),
                    NodeType::Data,
                )
                .with_metadata("format", "MD21 Chunked")
                .with_metadata("note", "Additional chunks may be present");
                root = root.add_child(chunks_node);
            }
        }
    }

    // Configure tree rendering options
    let options = TreeOptions {
        verbose: false,
        max_depth: Some(max_depth),
        show_external_refs: show_refs,
        no_color: false,
        show_metadata: true,
        compact: false,
    };

    let tree_output = render_tree(&root, &options);
    println!("{}", tree_output);

    Ok(())
}

fn handle_skin_info_auto(path: PathBuf, detailed: bool, force_old_format: bool) -> Result<()> {
    println!("Loading Skin file: {}", path.display());

    // Get file size for reference
    let file_size = std::fs::metadata(&path)
        .map(|m| m.len())
        .unwrap_or(0);

    // If force_old_format is specified, use the old parser directly
    if force_old_format {
        let skin = SkinG::<OldSkinHeader>::load(&path)
            .with_context(|| format!("Failed to load Skin file from {}", path.display()))?;

        println!("\n=== Skin Information ===");
        println!("Format: Old (forced via --old-format)");
        println!("File size: {} bytes", file_size);
        println!("Indices: {}", skin.indices.len());
        println!("Triangles: {}", skin.triangles.len());
        println!("Bone Indices: {}", skin.bone_indices.len());
        println!("Submeshes: {}", skin.submeshes.len());
        println!("Batches: {}", skin.batches.len());

        if detailed {
            print_skin_details(&skin.submeshes, &skin.batches);
            print_skin_samples(&skin.indices, &skin.triangles, &skin.bone_indices);
        }

        return Ok(());
    }

    // Use auto-detection
    let skin = SkinFile::load(&path)
        .with_context(|| format!("Failed to load Skin file from {}", path.display()))?;

    println!("\n=== Skin Information ===");

    let format_name = if skin.is_new_format() {
        "New (Cataclysm+)"
    } else {
        "Old (WotLK and earlier)"
    };
    println!("Format: {}", format_name);
    println!("File size: {} bytes", file_size);

    println!("Indices: {}", skin.indices().len());
    println!("Triangles: {}", skin.triangles().len());
    println!("Bone Indices: {}", skin.bone_indices().len());
    println!("Submeshes: {}", skin.submeshes().len());
    println!("Batches: {}", skin.batches().len());

    if detailed {
        print_skin_details(skin.submeshes(), skin.batches());
        print_skin_samples(skin.indices(), skin.triangles(), skin.bone_indices());
    }

    Ok(())
}

fn print_skin_details(
    submeshes: &[wow_m2::skin::SkinSubmesh],
    batches: &[wow_m2::skin::SkinBatch],
) {
    if !submeshes.is_empty() {
        println!("\n=== Submeshes ===");
        for (i, submesh) in submeshes.iter().enumerate() {
            println!(
                "  [{}] ID: {}, Vertices: {} (start: {}), Triangles: {} (start: {})",
                i,
                submesh.id,
                submesh.vertex_count,
                submesh.vertex_start,
                submesh.triangle_count,
                submesh.triangle_start
            );
            println!(
                "       Bones: {} (start: {}), Center: [{:.2}, {:.2}, {:.2}]",
                submesh.bone_count,
                submesh.bone_start,
                submesh.center[0],
                submesh.center[1],
                submesh.center[2]
            );
        }
    }

    if !batches.is_empty() {
        println!("\n=== Batches ===");
        for (i, batch) in batches.iter().enumerate() {
            println!(
                "  [{}] Submesh: {}, Shader: {}, Textures: {}, Material: {}",
                i,
                batch.skin_section_index,
                batch.shader_id,
                batch.texture_count,
                batch.material_index
            );
        }
    }
}

fn print_skin_samples(indices: &[u16], triangles: &[u16], bone_indices: &[u8]) {
    println!("\n=== Data Samples ===");

    // Show first few indices
    if !indices.is_empty() {
        let sample: Vec<_> = indices.iter().take(10).collect();
        println!("Indices (first 10): {:?}", sample);
        if indices.len() > 10 {
            println!("  ... and {} more", indices.len() - 10);
        }
    }

    // Show first few triangles (as groups of 3)
    if !triangles.is_empty() {
        println!("Triangles (first 3 faces):");
        for i in 0..3.min(triangles.len() / 3) {
            let base = i * 3;
            println!(
                "  Face {}: [{}, {}, {}]",
                i, triangles[base], triangles[base + 1], triangles[base + 2]
            );
        }
        if triangles.len() > 9 {
            println!("  ... and {} more faces", triangles.len() / 3 - 3);
        }
    }

    // Show first few bone indices (as groups of 4 if possible)
    if !bone_indices.is_empty() {
        println!("Bone indices (first 5 vertices):");
        for i in 0..5.min(bone_indices.len() / 4) {
            let base = i * 4;
            if base + 3 < bone_indices.len() {
                println!(
                    "  Vertex {}: [{}, {}, {}, {}]",
                    i,
                    bone_indices[base],
                    bone_indices[base + 1],
                    bone_indices[base + 2],
                    bone_indices[base + 3]
                );
            }
        }
        println!(
            "  Total: {} bytes ({} if 4 bytes/vertex = {} vertices)",
            bone_indices.len(),
            bone_indices.len(),
            bone_indices.len() / 4
        );
    }
}

#[allow(dead_code)]
fn handle_skin_info<H: SkinHeaderT + Clone>(path: PathBuf, detailed: bool) -> Result<()> {
    println!("Loading Skin file: {}", path.display());

    let _skin = SkinG::<H>::load(&path)
        .with_context(|| format!("Failed to load Skin file from {}", path.display()))?;

    println!("\n=== Skin Information ===");
    println!("File loaded successfully!");

    if detailed {
        println!("\n=== Detailed Information ===");
        println!("(Detailed information requires additional public API methods)");
    }

    Ok(())
}

fn handle_skin_convert(input: PathBuf, output: PathBuf, version_str: String) -> Result<()> {
    println!("Loading Skin file: {}", input.display());

    // Use SkinFile::load() for automatic format detection
    let skin = SkinFile::load(&input)
        .with_context(|| format!("Failed to load Skin file from {}", input.display()))?;

    let source_format = if skin.is_new_format() {
        "new format"
    } else {
        "old format"
    };
    println!("Detected source format: {}", source_format);

    let target_version = M2Version::from_expansion_name(&version_str)
        .with_context(|| format!("Invalid target version: {version_str}"))?;

    let target_format = if target_version.uses_new_skin_format() {
        "new format"
    } else {
        "old format"
    };
    println!("Converting to {:?} ({})...", target_version, target_format);

    // Actually perform the conversion
    let converted = skin
        .convert(target_version)
        .with_context(|| format!("Failed to convert skin to {:?}", target_version))?;

    println!("Saving converted Skin file to: {}", output.display());
    converted
        .save(&output)
        .with_context(|| format!("Failed to save converted Skin file to {}", output.display()))?;

    println!("Conversion complete!");
    Ok(())
}

fn handle_anim_info(path: PathBuf, detailed: bool) -> Result<()> {
    println!("Loading ANIM file: {}", path.display());

    let anim = AnimFile::load(&path)
        .with_context(|| format!("Failed to load ANIM file from {}", path.display()))?;

    println!("\n=== ANIM Information ===");
    println!("Format: {:?}", anim.format);
    println!("Animation Sections: {}", anim.animation_count());

    if anim.is_legacy_format() {
        println!("Legacy Format: True");
    } else {
        println!("Modern Format: True");
    }

    // Show memory usage stats
    let usage = anim.memory_usage();
    println!("Total Keyframes: {}", usage.total_keyframes());
    println!("Memory Usage: ~{} bytes", usage.approximate_bytes);

    if detailed {
        println!("\n=== Detailed Information ===");

        // Show format-specific metadata
        match &anim.metadata {
            wow_m2::AnimMetadata::Legacy {
                file_size,
                animation_count,
                structure_hints,
            } => {
                println!("File Size: {} bytes", file_size);
                println!("Animation Count (metadata): {}", animation_count);
                println!("Structure Valid: {}", structure_hints.appears_valid);
                println!("Estimated Blocks: {}", structure_hints.estimated_blocks);
                println!("Has Timestamps: {}", structure_hints.has_timestamps);
            }
            wow_m2::AnimMetadata::Modern { header, entries } => {
                println!("ANIM Version: {}", header.version);
                println!("ID Count: {}", header.id_count);
                println!("Entry Offset: {}", header.anim_entry_offset);
                println!("Entry Count: {}", entries.len());

                if !entries.is_empty() {
                    println!("\n=== Animation Entries ===");
                    for (i, entry) in entries.iter().enumerate() {
                        println!(
                            "Entry {}: ID={}, Offset={}, Size={}",
                            i, entry.id, entry.offset, entry.size
                        );
                    }
                }
            }
        }

        // Show memory breakdown
        println!("\n=== Memory Usage Breakdown ===");
        println!("Sections: {}", usage.sections);
        println!("Bone Animations: {}", usage.bone_animations);
        println!("Translation Keyframes: {}", usage.translation_keyframes);
        println!("Rotation Keyframes: {}", usage.rotation_keyframes);
        println!("Scaling Keyframes: {}", usage.scaling_keyframes);

        // Show sections summary
        if !anim.sections.is_empty() {
            println!("\n=== Animation Sections ===");
            for (i, section) in anim.sections.iter().enumerate() {
                println!(
                    "Section {}: ID={}, Start={}, End={}, Bones={}",
                    i,
                    section.header.id,
                    section.header.start,
                    section.header.end,
                    section.bone_animations.len()
                );
            }
        }
    }

    Ok(())
}

fn handle_anim_convert(input: PathBuf, output: PathBuf, version_str: String) -> Result<()> {
    println!("Loading ANIM file: {}", input.display());

    let anim = AnimFile::load(&input)
        .with_context(|| format!("Failed to load ANIM file from {}", input.display()))?;

    let target_version = M2Version::from_expansion_name(&version_str)
        .with_context(|| format!("Invalid target version: {version_str}"))?;

    println!("Source Format: {:?}", anim.format);
    println!("Converting to {target_version:?}");

    let converted = anim.convert(target_version);
    println!("Target Format: {:?}", converted.format);

    if converted.format == anim.format {
        println!("Note: No format conversion needed - same format for target version");
    }

    println!("Saving converted ANIM file to: {}", output.display());
    converted
        .save(&output)
        .with_context(|| format!("Failed to save converted ANIM file to {}", output.display()))?;

    println!("Conversion complete!");
    Ok(())
}

fn handle_blp_info(path: PathBuf, detailed: bool) -> Result<()> {
    println!("Loading BLP texture: {}", path.display());

    let _blp = load_blp(&path)
        .with_context(|| format!("Failed to load BLP texture from {}", path.display()))?;

    println!("\n=== BLP Texture Information ===");
    println!("File loaded successfully!");

    if detailed {
        println!("\n=== Detailed Information ===");
        println!("(Detailed information requires additional public API methods)");
    }

    Ok(())
}
