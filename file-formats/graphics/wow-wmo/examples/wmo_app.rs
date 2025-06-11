use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::process;

use wow_wmo::{WmoEditor, WmoVersion, WmoVisualizer, parse_wmo, parse_wmo_group};

// Import the rest of the necessary types
use wow_wmo::types::Vec3;

/// Main application for WMO manipulation
struct WmoApp {
    verbose: bool,
}

impl WmoApp {
    /// Create a new WMO application
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Process command line arguments and run the appropriate function
    fn run(&self, args: &[String]) -> i32 {
        if args.len() < 2 {
            print_usage();
            return 1;
        }

        let command = &args[1];

        match command.as_str() {
            "info" => {
                if args.len() < 3 {
                    println!("Error: Missing file path");
                    return 1;
                }

                let path = &args[2];
                if let Err(err) = self.show_info(path) {
                    println!("Error: {}", err);
                    return 1;
                }
            }
            "convert" => {
                if args.len() < 5 {
                    println!("Error: Missing arguments");
                    println!("Usage: wmo_app convert <input> <output> <version>");
                    return 1;
                }

                let input_path = &args[2];
                let output_path = &args[3];
                let version_str = &args[4];

                let version = match version_str.parse::<u32>() {
                    Ok(v) => match WmoVersion::from_raw(v) {
                        Some(version) => version,
                        None => {
                            println!("Error: Invalid WMO version: {}", v);
                            return 1;
                        }
                    },
                    Err(_) => {
                        println!("Error: Invalid version number: {}", version_str);
                        return 1;
                    }
                };

                if let Err(err) = self.convert_wmo(input_path, output_path, version) {
                    println!("Error: {}", err);
                    return 1;
                }
            }
            "extract" => {
                if args.len() < 4 {
                    println!("Error: Missing arguments");
                    println!("Usage: wmo_app extract <wmo_file> <output_dir>");
                    return 1;
                }

                let wmo_path = &args[2];
                let output_dir = &args[3];

                if let Err(err) = self.extract_resources(wmo_path, output_dir) {
                    println!("Error: {}", err);
                    return 1;
                }
            }
            "export" => {
                if args.len() < 4 {
                    println!("Error: Missing arguments");
                    println!("Usage: wmo_app export <wmo_file> <output_dir> [format]");
                    println!("Formats: obj (default), fbx, gltf");
                    return 1;
                }

                let wmo_path = &args[2];
                let output_dir = &args[3];

                let format = if args.len() >= 5 { &args[4] } else { "obj" };

                if let Err(err) = self.export_model(wmo_path, output_dir, format) {
                    println!("Error: {}", err);
                    return 1;
                }
            }
            "edit" => {
                if args.len() < 4 {
                    println!("Error: Missing arguments");
                    println!("Usage: wmo_app edit <wmo_file> <output_file> [operations...]");
                    println!("Operations:");
                    println!("  --scale <factor>                   Scale the model by a factor");
                    println!("  --translate <x> <y> <z>            Translate the model");
                    println!("  --remove-group <index>             Remove a group");
                    println!("  --rename-group <index> <name>      Rename a group");
                    println!("  --add-vertex <group> <x> <y> <z>   Add a vertex to a group");
                    return 1;
                }

                let wmo_path = &args[2];
                let output_path = &args[3];

                if let Err(err) = self.edit_wmo(wmo_path, output_path, &args[4..]) {
                    println!("Error: {}", err);
                    return 1;
                }
            }
            "validate" => {
                if args.len() < 3 {
                    println!("Error: Missing file path");
                    return 1;
                }

                let path = &args[2];

                if let Err(err) = self.validate_wmo(path) {
                    println!("Error: {}", err);
                    return 1;
                }
            }
            _ => {
                println!("Unknown command: {}", command);
                print_usage();
                return 1;
            }
        }

        0
    }

    /// Show information about a WMO file
    fn show_info(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Loading WMO file: {}", path);

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let wmo = parse_wmo(&mut reader)?;

        println!("WMO Information:");
        println!("----------------");
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

        if !wmo.doodad_sets.is_empty() {
            println!("\nDoodad Sets:");
            for (i, set) in wmo.doodad_sets.iter().enumerate() {
                println!("  {}. {} ({} doodads)", i + 1, set.name, set.n_doodads);
            }
        }

        if !wmo.groups.is_empty() {
            println!("\nGroups:");
            for (i, group) in wmo.groups.iter().enumerate() {
                println!("  {}. {} (flags: {:?})", i + 1, group.name, group.flags);

                // For verbose mode, also show group details
                if self.verbose {
                    // Load group file
                    let group_filename = Self::get_group_filename(path, i);
                    if let Ok(group_file) = File::open(&group_filename) {
                        println!("    Loading group file: {}", group_filename);
                        let mut group_reader = BufReader::new(group_file);

                        if let Ok(group_data) = parse_wmo_group(&mut group_reader, i as u32) {
                            println!("    Vertices: {}", group_data.vertices.len());
                            println!("    Indices: {}", group_data.indices.len());
                            println!("    Batches: {}", group_data.batches.len());

                            if let Some(colors) = &group_data.vertex_colors {
                                println!("    Vertex Colors: {}", colors.len());
                            }

                            if let Some(liquid) = &group_data.liquid {
                                println!(
                                    "    Liquid: {}x{} (Type: 0x{:X})",
                                    liquid.width, liquid.height, liquid.liquid_type
                                );
                            }
                        }
                    }
                }
            }
        }

        if !wmo.textures.is_empty() {
            println!("\nTextures:");
            for (i, texture) in wmo.textures.iter().enumerate() {
                println!("  {}. {}", i + 1, texture);
            }
        }

        println!("\nBounding Box:");
        println!(
            "  Min: ({:.2}, {:.2}, {:.2})",
            wmo.bounding_box.min.x, wmo.bounding_box.min.y, wmo.bounding_box.min.z
        );
        println!(
            "  Max: ({:.2}, {:.2}, {:.2})",
            wmo.bounding_box.max.x, wmo.bounding_box.max.y, wmo.bounding_box.max.z
        );

        println!("\nHeader Flags: {:?}", wmo.header.flags);

        if let Some(ref skybox) = wmo.skybox {
            println!("\nSkybox: {}", skybox);
        }

        Ok(())
    }

    /// Convert a WMO file to a different version
    fn convert_wmo(
        &self,
        input_path: &str,
        output_path: &str,
        target_version: WmoVersion,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Converting WMO: {} -> {} (version {})",
            input_path,
            output_path,
            target_version.to_raw()
        );

        // Open input file
        let input_file = File::open(input_path)?;
        let mut reader = BufReader::new(input_file);

        // Parse WMO
        let wmo = parse_wmo(&mut reader)?;
        let current_version = wmo.version;

        println!(
            "Current version: {} ({})",
            current_version.to_raw(),
            current_version.expansion_name()
        );
        println!(
            "Target version: {} ({})",
            target_version.to_raw(),
            target_version.expansion_name()
        );

        // Create WMO editor
        let mut editor = WmoEditor::new(wmo);

        // Convert to target version
        editor.convert_to_version(target_version)?;

        // Save root file
        let output_file = File::create(output_path)?;
        let mut writer = BufWriter::new(output_file);
        editor.save_root(&mut writer)?;

        println!("Conversion complete!");

        // If groups need to be converted too, find and process them
        let base_input_path = Path::new(input_path).with_extension("");
        let base_output_path = Path::new(output_path).with_extension("");

        for i in 0..editor.group_count() {
            // Try to find and load group file
            let group_filename = format!("{}_{:03}.wmo", base_input_path.display(), i);

            if let Ok(group_file) = File::open(&group_filename) {
                println!("Converting group file: {}", group_filename);

                let mut group_reader = BufReader::new(group_file);
                let group = parse_wmo_group(&mut group_reader, i as u32)?;

                // Add to editor
                editor.add_group(group)?;

                // Save converted group
                let output_group_filename = format!("{}_{:03}.wmo", base_output_path.display(), i);
                let output_group_file = File::create(output_group_filename)?;
                let mut group_writer = BufWriter::new(output_group_file);

                editor.save_group(&mut group_writer, i)?;
            }
        }

        Ok(())
    }

    /// Extract resources (textures, doodads) from a WMO file
    fn extract_resources(
        &self,
        wmo_path: &str,
        output_dir: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Extracting resources from: {} to {}", wmo_path, output_dir);

        // Ensure output directory exists
        fs::create_dir_all(output_dir)?;

        // Open input file
        let input_file = File::open(wmo_path)?;
        let mut reader = BufReader::new(input_file);

        // Parse WMO
        let wmo = parse_wmo(&mut reader)?;

        // Create subdirectories
        let textures_dir = format!("{}/textures", output_dir);
        let doodads_dir = format!("{}/doodads", output_dir);
        let groups_dir = format!("{}/groups", output_dir);

        fs::create_dir_all(&textures_dir)?;
        fs::create_dir_all(&doodads_dir)?;
        fs::create_dir_all(&groups_dir)?;

        // Create a list of textures
        if !wmo.textures.is_empty() {
            println!("\nTextures:");
            let texture_list_path = format!("{}/textures.txt", output_dir);
            let mut texture_list = File::create(texture_list_path)?;

            for (i, texture) in wmo.textures.iter().enumerate() {
                writeln!(texture_list, "{}. {}", i, texture)?;
                println!("  {}. {}", i, texture);
            }
        }

        // Create a list of doodads
        if !wmo.doodad_defs.is_empty() {
            println!("\nDoodads:");
            let doodad_list_path = format!("{}/doodads.txt", output_dir);
            let mut doodad_list = File::create(doodad_list_path)?;

            for (i, doodad) in wmo.doodad_defs.iter().enumerate() {
                let name = format!("Doodad_{}", doodad.name_offset);
                writeln!(
                    doodad_list,
                    "{}. {} at ({:.2}, {:.2}, {:.2}) scale {:.2}",
                    i, name, doodad.position.x, doodad.position.y, doodad.position.z, doodad.scale
                )?;
                println!(
                    "  {}. {} at ({:.2}, {:.2}, {:.2}) scale {:.2}",
                    i, name, doodad.position.x, doodad.position.y, doodad.position.z, doodad.scale
                );
            }
        }

        // Create a list of groups
        if !wmo.groups.is_empty() {
            println!("\nGroups:");
            let group_list_path = format!("{}/groups.txt", output_dir);
            let mut group_list = File::create(group_list_path)?;

            for (i, group) in wmo.groups.iter().enumerate() {
                writeln!(group_list, "{}. {}", i, group.name)?;
                println!("  {}. {}", i, group.name);

                // Extract group data if file exists
                let group_filename = Self::get_group_filename(wmo_path, i);
                if let Ok(group_file) = File::open(&group_filename) {
                    let mut group_reader = BufReader::new(group_file);

                    if let Ok(group_data) = parse_wmo_group(&mut group_reader, i as u32) {
                        // Save group info
                        let group_info_path = format!("{}/group_{}.txt", groups_dir, i);
                        let mut group_info = File::create(group_info_path)?;

                        writeln!(group_info, "Group: {}", group.name)?;
                        writeln!(group_info, "Vertices: {}", group_data.vertices.len())?;
                        writeln!(group_info, "Indices: {}", group_data.indices.len())?;
                        writeln!(group_info, "Batches: {}", group_data.batches.len())?;

                        if let Some(colors) = &group_data.vertex_colors {
                            writeln!(group_info, "Vertex Colors: {}", colors.len())?;
                        }

                        if let Some(liquid) = &group_data.liquid {
                            writeln!(
                                group_info,
                                "Liquid: {}x{} (Type: 0x{:X})",
                                liquid.width, liquid.height, liquid.liquid_type
                            )?;
                        }
                    }
                }
            }
        }

        // Export as OBJ for visualization
        let visualizer = WmoVisualizer::new();

        // Load groups
        let mut groups = Vec::new();

        for i in 0..wmo.groups.len() {
            let group_filename = Self::get_group_filename(wmo_path, i);
            if let Ok(group_file) = File::open(&group_filename) {
                let mut group_reader = BufReader::new(group_file);

                if let Ok(group_data) = parse_wmo_group(&mut group_reader, i as u32) {
                    groups.push(group_data);
                }
            }
        }

        // Export OBJ if we have groups
        if !groups.is_empty() {
            println!("\nExporting to OBJ...");
            let obj = visualizer.export_to_obj(&wmo, &groups);
            let mtl = visualizer.export_to_mtl(&wmo);

            let obj_path = format!("{}/model.obj", output_dir);
            let mtl_path = format!("{}/materials.mtl", output_dir);

            fs::write(obj_path, obj)?;
            fs::write(mtl_path, mtl)?;

            println!("OBJ/MTL export complete");
        }

        println!("\nResource extraction complete!");

        Ok(())
    }

    /// Export a WMO model to a 3D format
    fn export_model(
        &self,
        wmo_path: &str,
        output_dir: &str,
        format: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Exporting WMO: {} to {} format in {}",
            wmo_path, format, output_dir
        );

        // Ensure output directory exists
        fs::create_dir_all(output_dir)?;

        // Open input file
        let input_file = File::open(wmo_path)?;
        let mut reader = BufReader::new(input_file);

        // Parse WMO
        let wmo = parse_wmo(&mut reader)?;

        // Load groups
        let mut groups = Vec::new();

        for i in 0..wmo.groups.len() {
            let group_filename = Self::get_group_filename(wmo_path, i);
            if let Ok(group_file) = File::open(&group_filename) {
                println!("Loading group file: {}", group_filename);
                let mut group_reader = BufReader::new(group_file);

                if let Ok(group_data) = parse_wmo_group(&mut group_reader, i as u32) {
                    groups.push(group_data);
                }
            }
        }

        if groups.is_empty() {
            println!("No group files found, export will be incomplete");
        }

        // Export based on format
        let visualizer = WmoVisualizer::new();

        match format.to_lowercase().as_str() {
            "obj" => {
                println!("Exporting to OBJ...");
                let obj = visualizer.export_to_obj(&wmo, &groups);
                let mtl = visualizer.export_to_mtl(&wmo);

                let obj_path = format!("{}/model.obj", output_dir);
                let mtl_path = format!("{}/materials.mtl", output_dir);

                fs::write(obj_path, obj)?;
                fs::write(mtl_path, mtl)?;

                println!("OBJ/MTL export complete");
            }
            "fbx" | "gltf" => {
                println!("Export to {} format is not yet implemented", format);
                return Err(format!("Export to {} format is not yet implemented", format).into());
            }
            _ => {
                println!("Unknown format: {}", format);
                return Err(format!("Unknown format: {}", format).into());
            }
        }

        Ok(())
    }

    /// Edit a WMO file
    fn edit_wmo(
        &self,
        wmo_path: &str,
        output_path: &str,
        operations: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Editing WMO: {} -> {}", wmo_path, output_path);

        // Open input file
        let input_file = File::open(wmo_path)?;
        let mut reader = BufReader::new(input_file);

        // Parse WMO
        let wmo = parse_wmo(&mut reader)?;

        // Create WMO editor
        let mut editor = WmoEditor::new(wmo);

        // Process each operation
        let mut i = 0;
        while i < operations.len() {
            let operation = &operations[i];

            match operation.as_str() {
                "--scale" => {
                    if i + 1 >= operations.len() {
                        return Err("Missing scale factor".into());
                    }

                    let factor = operations[i + 1].parse::<f32>()?;

                    println!("Scaling by factor: {}", factor);

                    // We need to load and scale each group
                    for group_idx in 0..editor.group_count() {
                        // Try to load group file if not already loaded
                        if !editor.is_group_loaded(group_idx) {
                            let group_filename = Self::get_group_filename(wmo_path, group_idx);
                            if let Ok(group_file) = File::open(&group_filename) {
                                let mut group_reader = BufReader::new(group_file);

                                if let Ok(group_data) =
                                    parse_wmo_group(&mut group_reader, group_idx as u32)
                                {
                                    editor.add_group(group_data)?;
                                }
                            }
                        }

                        // Scale vertices if group is loaded
                        if let Some(group) = editor.group_mut(group_idx) {
                            for vertex in &mut group.vertices {
                                vertex.x *= factor;
                                vertex.y *= factor;
                                vertex.z *= factor;
                            }

                            // Update bounding box
                            editor.recalculate_group_bounding_box(group_idx)?;
                        }
                    }

                    // Update global bounding box
                    editor.recalculate_global_bounding_box()?;

                    i += 2;
                }
                "--translate" => {
                    if i + 3 >= operations.len() {
                        return Err("Missing translation coordinates".into());
                    }

                    let x = operations[i + 1].parse::<f32>()?;
                    let y = operations[i + 2].parse::<f32>()?;
                    let z = operations[i + 3].parse::<f32>()?;

                    println!("Translating by ({}, {}, {})", x, y, z);

                    // We need to load and translate each group
                    for group_idx in 0..editor.group_count() {
                        // Try to load group file if not already loaded
                        if !editor.is_group_loaded(group_idx) {
                            let group_filename = Self::get_group_filename(wmo_path, group_idx);
                            if let Ok(group_file) = File::open(&group_filename) {
                                let mut group_reader = BufReader::new(group_file);

                                if let Ok(group_data) =
                                    parse_wmo_group(&mut group_reader, group_idx as u32)
                                {
                                    editor.add_group(group_data)?;
                                }
                            }
                        }

                        // Translate vertices if group is loaded
                        if let Some(group) = editor.group_mut(group_idx) {
                            for vertex in &mut group.vertices {
                                vertex.x += x;
                                vertex.y += y;
                                vertex.z += z;
                            }

                            // Update bounding box
                            editor.recalculate_group_bounding_box(group_idx)?;
                        }
                    }

                    // Also translate doodads
                    for doodad in &mut editor.root_mut().doodad_defs {
                        doodad.position.x += x;
                        doodad.position.y += y;
                        doodad.position.z += z;
                    }

                    // Update global bounding box
                    editor.recalculate_global_bounding_box()?;

                    i += 4;
                }
                "--remove-group" => {
                    if i + 1 >= operations.len() {
                        return Err("Missing group index".into());
                    }

                    let group_idx = operations[i + 1].parse::<usize>()?;

                    println!("Removing group: {}", group_idx);

                    // Remove group
                    editor.remove_group(group_idx)?;

                    i += 2;
                }
                "--rename-group" => {
                    if i + 2 >= operations.len() {
                        return Err("Missing group index or name".into());
                    }

                    let group_idx = operations[i + 1].parse::<usize>()?;
                    let new_name = &operations[i + 2];

                    println!("Renaming group {} to: {}", group_idx, new_name);

                    // Rename group
                    if let Some(group) = editor.root_mut().groups.get_mut(group_idx) {
                        group.name = new_name.clone();
                    } else {
                        return Err(format!("Group {} does not exist", group_idx).into());
                    }

                    i += 3;
                }
                "--add-vertex" => {
                    if i + 4 >= operations.len() {
                        return Err("Missing group index or vertex coordinates".into());
                    }

                    let group_idx = operations[i + 1].parse::<usize>()?;
                    let x = operations[i + 2].parse::<f32>()?;
                    let y = operations[i + 3].parse::<f32>()?;
                    let z = operations[i + 4].parse::<f32>()?;

                    println!(
                        "Adding vertex to group {}: ({}, {}, {})",
                        group_idx, x, y, z
                    );

                    // Try to load group file if not already loaded
                    if !editor.is_group_loaded(group_idx) {
                        let group_filename = Self::get_group_filename(wmo_path, group_idx);
                        if let Ok(group_file) = File::open(&group_filename) {
                            let mut group_reader = BufReader::new(group_file);

                            if let Ok(group_data) =
                                parse_wmo_group(&mut group_reader, group_idx as u32)
                            {
                                editor.add_group(group_data)?;
                            }
                        } else {
                            return Err(format!("Group {} file not found", group_idx).into());
                        }
                    }

                    // Add vertex
                    editor.add_vertex(group_idx, Vec3 { x, y, z })?;

                    i += 5;
                }
                _ => {
                    return Err(format!("Unknown operation: {}", operation).into());
                }
            }
        }

        // Save root file
        let output_file = File::create(output_path)?;
        let mut writer = BufWriter::new(output_file);
        editor.save_root(&mut writer)?;

        // Save group files
        let base_output_path = Path::new(output_path).with_extension("");

        for i in 0..editor.group_count() {
            if editor.is_group_modified(i) {
                let output_group_filename = format!("{}_{:03}.wmo", base_output_path.display(), i);
                let output_group_file = File::create(output_group_filename)?;
                let mut group_writer = BufWriter::new(output_group_file);

                editor.save_group(&mut group_writer, i)?;
            }
        }

        println!("Editing complete!");

        Ok(())
    }

    /// Validate a WMO file
    fn validate_wmo(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Validating WMO file: {}", path);

        // Open input file
        let input_file = File::open(path)?;
        let mut reader = BufReader::new(input_file);

        // Validate WMO
        let wmo = parse_wmo(&mut reader)?;

        // Create validator
        let validator = wow_wmo::WmoValidator::new();
        let report = validator.validate_root(&wmo)?;

        // Print validation results
        println!("\nValidation Report:");
        println!("-----------------");

        if report.has_errors() {
            println!("Errors: {}", report.error_count());
            for error in &report.errors {
                println!("  - {}", error);
            }
        } else {
            println!("No errors found!");
        }

        if report.has_warnings() {
            println!("\nWarnings: {}", report.warning_count());
            for warning in &report.warnings {
                println!("  - {}", warning);
            }
        } else {
            println!("No warnings found!");
        }

        // Validate groups if in verbose mode
        if self.verbose {
            for i in 0..wmo.groups.len() {
                let group_filename = Self::get_group_filename(path, i);
                if let Ok(group_file) = File::open(&group_filename) {
                    println!("\nValidating group file: {}", group_filename);
                    let mut group_reader = BufReader::new(group_file);

                    if let Ok(group_data) = parse_wmo_group(&mut group_reader, i as u32) {
                        let group_report = validator.validate_group(&group_data)?;

                        if group_report.has_errors() {
                            println!("Errors: {}", group_report.error_count());
                            for error in &group_report.errors {
                                println!("  - {}", error);
                            }
                        } else {
                            println!("No errors found!");
                        }

                        if group_report.has_warnings() {
                            println!("Warnings: {}", group_report.warning_count());
                            for warning in &group_report.warnings {
                                println!("  - {}", warning);
                            }
                        } else {
                            println!("No warnings found!");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the filename for a group file
    fn get_group_filename(wmo_path: &str, group_index: usize) -> String {
        let base_path = Path::new(wmo_path).with_extension("");
        format!("{}_{:03}.wmo", base_path.display(), group_index)
    }
}

/// Print usage information
fn print_usage() {
    println!("WMO File Manipulation Tool");
    println!("-------------------------");
    println!("Usage: wmo_app <command> [args...]");
    println!();
    println!("Commands:");
    println!("  info <file>                    Show information about a WMO file");
    println!("  convert <input> <output> <ver> Convert a WMO file to a different version");
    println!("  extract <wmo> <outdir>         Extract resources from a WMO file");
    println!("  export <wmo> <outdir> [format] Export a WMO model to a 3D format");
    println!("  edit <wmo> <output> [ops...]   Edit a WMO file");
    println!("  validate <file>                Validate a WMO file");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");

    let app = WmoApp::new(verbose);
    let result = app.run(&args);

    process::exit(result);
}
