use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wow_wmo::{ParsedWmo, parse_wmo_with_metadata};

/// Test all WMO files in our test data directories
#[test]
fn test_parse_all_vanilla_wmo_files() {
    let test_dir = Path::new("tests/data/vanilla");
    test_all_wmo_files_in_dir(test_dir, "vanilla");
}

#[test]
fn test_parse_all_wotlk_wmo_files() {
    let test_dir = Path::new("tests/data/wotlk");
    test_all_wmo_files_in_dir(test_dir, "wotlk");
}

#[test]
fn test_parse_all_cataclysm_wmo_files() {
    let test_dir = Path::new("tests/data/cataclysm");
    test_all_wmo_files_in_dir(test_dir, "cataclysm");
}

fn test_all_wmo_files_in_dir(dir: &Path, expansion: &str) {
    if !dir.exists() {
        eprintln!(
            "Skipping {} tests - directory not found: {:?}",
            expansion, dir
        );
        return;
    }

    let mut tested = 0;
    let mut root_files = 0;
    let mut group_files = 0;
    let mut failures = Vec::new();

    // Walk directory recursively
    visit_dirs(dir, &mut |path| {
        if path.extension().and_then(|e| e.to_str()) == Some("wmo") {
            let file_name = path.file_name().unwrap().to_str().unwrap();

            // Try to parse the file
            match File::open(path) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    match parse_wmo_with_metadata(&mut reader) {
                        Ok(result) => {
                            tested += 1;

                            // Track file types
                            match &result.wmo {
                                ParsedWmo::Root(root) => {
                                    root_files += 1;

                                    // Verify basic sanity
                                    assert_eq!(root.version, 17, "Expected version 17 for {}", file_name);

                                    // Log statistics for root files
                                    if root.n_groups > 0 || root.n_materials > 0 {
                                        println!(
                                            "  {} root: {} materials, {} groups, {} lights, {} doodads",
                                            file_name,
                                            root.n_materials,
                                            root.n_groups,
                                            root.n_lights,
                                            root.doodad_defs.len()
                                        );
                                    }
                                }
                                ParsedWmo::Group(group) => {
                                    group_files += 1;
                                    assert_eq!(group.version, 17, "Expected version 17 for {}", file_name);
                                }
                            }

                            // Check for malformed chunks
                            if let Some(metadata) = result.metadata() {
                                if metadata.has_malformed_chunks() {
                                    println!(
                                        "  WARNING: {} has {} malformed chunks",
                                        file_name,
                                        metadata.malformed_count()
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            failures.push(format!("{}: {}", path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    failures.push(format!("{}: Failed to open: {}", path.display(), e));
                }
            }
        }
    }).unwrap();

    // Report results
    println!("\n{} WMO parsing results:", expansion);
    println!("  Files tested: {}", tested);
    println!("  Root files: {}", root_files);
    println!("  Group files: {}", group_files);

    if !failures.is_empty() {
        println!("\n  Failures ({}):", failures.len());
        for failure in &failures {
            println!("    {}", failure);
        }
        panic!("{} files failed to parse!", failures.len());
    }

    // Ensure we tested at least some files
    assert!(tested > 0, "No WMO files found to test in {:?}", dir);
}

// Helper to walk directories
fn visit_dirs<F>(dir: &Path, cb: &mut F) -> std::io::Result<()>
where
    F: FnMut(&Path),
{
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&path);
            }
        }
    }
    Ok(())
}
