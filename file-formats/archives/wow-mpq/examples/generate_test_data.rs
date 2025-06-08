//! Generate test data for MPQ archive testing
//!
//! This example replaces the generate_test_data.py script.
//!
//! Usage:
//!     cargo run --example generate_test_data -- simple
//!     cargo run --example generate_test_data -- all

use rand::{Rng, SeedableRng, rngs::StdRng};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// Re-implement the test data generation logic here
// (In a real project, this would be in a separate crate or workspace member)

#[derive(Debug, Clone)]
struct TestDataConfig {
    name: String,
    description: String,
    directories: Vec<String>,
    files: Vec<FileConfig>,
}

#[derive(Debug, Clone)]
struct FileConfig {
    path: String,
    file_type: FileType,
    size_kb: usize,
}

#[derive(Debug, Clone, Copy)]
enum FileType {
    Text,
    Binary,
    Empty,
}

#[derive(Debug)]
struct GenerationResult {
    base_path: PathBuf,
    files_created: Vec<String>,
}

impl TestDataConfig {
    fn simple() -> Self {
        Self {
            name: "simple".to_string(),
            description: "Simple flat structure with text files".to_string(),
            directories: vec![],
            files: vec![
                FileConfig {
                    path: "readme.txt".to_string(),
                    file_type: FileType::Text,
                    size_kb: 2,
                },
                FileConfig {
                    path: "data.txt".to_string(),
                    file_type: FileType::Text,
                    size_kb: 5,
                },
                FileConfig {
                    path: "config.ini".to_string(),
                    file_type: FileType::Text,
                    size_kb: 1,
                },
            ],
        }
    }

    fn game_assets() -> Self {
        Self {
            name: "game_assets".to_string(),
            description: "Game-like asset structure".to_string(),
            directories: vec![
                "textures".to_string(),
                "models".to_string(),
                "sounds".to_string(),
                "scripts".to_string(),
            ],
            files: vec![
                FileConfig {
                    path: "textures/player.dds".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 256,
                },
                FileConfig {
                    path: "textures/terrain.dds".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 512,
                },
                FileConfig {
                    path: "models/player.mdx".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 128,
                },
                FileConfig {
                    path: "models/building.mdx".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 64,
                },
                FileConfig {
                    path: "sounds/music/theme.mp3".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 1024,
                },
                FileConfig {
                    path: "sounds/effects/click.wav".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 32,
                },
                FileConfig {
                    path: "scripts/main.lua".to_string(),
                    file_type: FileType::Text,
                    size_kb: 10,
                },
                FileConfig {
                    path: "scripts/utils.lua".to_string(),
                    file_type: FileType::Text,
                    size_kb: 5,
                },
            ],
        }
    }

    fn nested() -> Self {
        Self {
            name: "nested".to_string(),
            description: "Deeply nested directory structure".to_string(),
            directories: vec![],
            files: vec![
                FileConfig {
                    path: "level1/readme.txt".to_string(),
                    file_type: FileType::Text,
                    size_kb: 1,
                },
                FileConfig {
                    path: "level1/level2/data.bin".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 10,
                },
                FileConfig {
                    path: "level1/level2/level3/config.xml".to_string(),
                    file_type: FileType::Text,
                    size_kb: 2,
                },
                FileConfig {
                    path: "level1/level2/level3/level4/deep.txt".to_string(),
                    file_type: FileType::Text,
                    size_kb: 1,
                },
            ],
        }
    }

    fn mixed_sizes() -> Self {
        Self {
            name: "mixed_sizes".to_string(),
            description: "Mix of file sizes from tiny to large".to_string(),
            directories: vec![],
            files: vec![
                FileConfig {
                    path: "tiny.txt".to_string(),
                    file_type: FileType::Empty,
                    size_kb: 0,
                },
                FileConfig {
                    path: "small.dat".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 1,
                },
                FileConfig {
                    path: "medium.bin".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 100,
                },
                FileConfig {
                    path: "large.pak".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 1024,
                },
                FileConfig {
                    path: "config.json".to_string(),
                    file_type: FileType::Text,
                    size_kb: 5,
                },
            ],
        }
    }

    fn special_names() -> Self {
        Self {
            name: "special_names".to_string(),
            description: "Files with special characters and spaces".to_string(),
            directories: vec![],
            files: vec![
                FileConfig {
                    path: "file with spaces.txt".to_string(),
                    file_type: FileType::Text,
                    size_kb: 1,
                },
                FileConfig {
                    path: "special-chars_$#@.dat".to_string(),
                    file_type: FileType::Binary,
                    size_kb: 5,
                },
                FileConfig {
                    path: "unicode_文件.txt".to_string(),
                    file_type: FileType::Text,
                    size_kb: 2,
                },
                FileConfig {
                    path: ".hidden_file".to_string(),
                    file_type: FileType::Text,
                    size_kb: 1,
                },
            ],
        }
    }

    fn all_configs() -> Vec<Self> {
        vec![
            Self::simple(),
            Self::game_assets(),
            Self::nested(),
            Self::mixed_sizes(),
            Self::special_names(),
        ]
    }
}

fn generate_test_data(
    base_path: &Path,
    config: &TestDataConfig,
) -> Result<GenerationResult, std::io::Error> {
    let output_dir = base_path.join(&config.name);

    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)?;
    }

    fs::create_dir_all(&output_dir)?;

    for dir in &config.directories {
        fs::create_dir_all(output_dir.join(dir))?;
    }

    let mut files_created = Vec::new();

    for file_config in &config.files {
        let file_path = output_dir.join(&file_config.path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        match file_config.file_type {
            FileType::Text => {
                let content = generate_text_content(file_config.size_kb);
                fs::write(&file_path, content)?;
            }
            FileType::Binary => {
                let content = generate_binary_content(file_config.size_kb);
                fs::write(&file_path, content)?;
            }
            FileType::Empty => {
                fs::File::create(&file_path)?;
            }
        }

        files_created.push(file_config.path.clone());
    }

    Ok(GenerationResult {
        base_path: output_dir,
        files_created,
    })
}

fn generate_text_content(size_kb: usize) -> Vec<u8> {
    let target_size = size_kb * 1024;
    let pattern = b"The quick brown fox jumps over the lazy dog. ";
    let mut content = Vec::with_capacity(target_size);

    while content.len() < target_size {
        let remaining = target_size - content.len();
        let to_copy = remaining.min(pattern.len());
        content.extend_from_slice(&pattern[..to_copy]);
    }

    content
}

fn generate_binary_content(size_kb: usize) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(42);
    let size_bytes = size_kb * 1024;
    let mut content = vec![0u8; size_bytes];
    rng.fill(&mut content[..]);
    content
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <config> [--output-dir <dir>]", args[0]);
        eprintln!("\nAvailable configs:");
        eprintln!("  simple       - Simple flat structure with text files");
        eprintln!("  game_assets  - Game-like asset structure");
        eprintln!("  nested       - Deeply nested directory structure");
        eprintln!("  mixed_sizes  - Mix of file sizes from tiny to large");
        eprintln!("  special_names - Files with special characters and spaces");
        eprintln!("  all          - Generate all configurations");
        std::process::exit(1);
    }

    let config_name = &args[1];
    let output_dir = if args.len() >= 4 && args[2] == "--output-dir" {
        &args[3]
    } else {
        "test-data/raw-data"
    };

    let output_base = Path::new(output_dir);

    let configs = if config_name == "all" {
        TestDataConfig::all_configs()
    } else {
        match config_name.as_str() {
            "simple" => vec![TestDataConfig::simple()],
            "game_assets" => vec![TestDataConfig::game_assets()],
            "nested" => vec![TestDataConfig::nested()],
            "mixed_sizes" => vec![TestDataConfig::mixed_sizes()],
            "special_names" => vec![TestDataConfig::special_names()],
            _ => {
                eprintln!("Unknown configuration: {}", config_name);
                std::process::exit(1);
            }
        }
    };

    println!("Generating test data in: {}", output_base.display());
    println!();

    for config in configs {
        println!("Creating '{}': {}", config.name, config.description);
        let result = generate_test_data(output_base, &config)?;
        println!(
            "  Created {} files in {}",
            result.files_created.len(),
            result.base_path.display()
        );
        for file in &result.files_created {
            println!("    - {}", file);
        }
        println!();
    }

    println!("Test data generation complete!");
    println!();
    println!("Example usage:");
    println!("  storm-cli archive create test.mpq {}/simple", output_dir);
    println!(
        "  storm-cli archive create game.mpq {}/game_assets --compression bzip2",
        output_dir
    );
    println!(
        "  storm-cli archive create nested.mpq {}/nested --recursive",
        output_dir
    );

    Ok(())
}
