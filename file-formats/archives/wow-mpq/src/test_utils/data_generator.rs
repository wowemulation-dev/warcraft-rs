//! Test data generation utilities
//!
//! Replaces the functionality of generate_test_data.py

use rand::{Rng, SeedableRng, rngs::StdRng};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for test data generation
#[derive(Debug, Clone)]
pub struct TestDataConfig {
    /// Name of the test configuration
    pub name: String,
    /// Human-readable description of what this configuration creates
    pub description: String,
    /// List of directories to create in the test structure
    pub directories: Vec<String>,
    /// List of files to create with their configurations
    pub files: Vec<FileConfig>,
}

/// Configuration for a single test file
#[derive(Debug, Clone)]
pub struct FileConfig {
    /// Relative path where the file should be created
    pub path: String,
    /// Type of content to generate for this file
    pub file_type: FileType,
    /// Size of the file in kilobytes
    pub size_kb: usize,
}

/// Type of file content to generate
#[derive(Debug, Clone, Copy)]
pub enum FileType {
    /// Generate text content with repeating patterns
    Text,
    /// Generate pseudo-random binary content
    Binary,
    /// Create an empty file
    Empty,
}

/// Content compressibility level for generated data
#[derive(Debug, Clone, Copy)]
pub enum Compressibility {
    /// Highly compressible data (repeated characters)
    High,
    /// Medium compressibility (repeating patterns)
    Medium,
    /// Low compressibility (pseudo-random data)
    Low,
}

/// Result of test data generation
#[derive(Debug)]
pub struct GenerationResult {
    /// Path to the base directory where test data was created
    pub base_path: PathBuf,
    /// List of files that were successfully created
    pub files_created: Vec<String>,
}

impl TestDataConfig {
    /// Create a simple flat structure with text files
    pub fn simple() -> Self {
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

    /// Create a game-like asset structure
    pub fn game_assets() -> Self {
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

    /// Create a deeply nested directory structure
    pub fn nested() -> Self {
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

    /// Create files with various sizes
    pub fn mixed_sizes() -> Self {
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

    /// Create files with special characters and spaces
    pub fn special_names() -> Self {
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

    /// Get all predefined configurations
    pub fn all_configs() -> Vec<Self> {
        vec![
            Self::simple(),
            Self::game_assets(),
            Self::nested(),
            Self::mixed_sizes(),
            Self::special_names(),
        ]
    }
}

/// Generate test data with specified configuration
pub fn generate_test_data(
    base_path: &Path,
    config: &TestDataConfig,
) -> Result<GenerationResult, std::io::Error> {
    let output_dir = base_path.join(&config.name);

    // Clean up if exists
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)?;
    }

    // Create base directory
    fs::create_dir_all(&output_dir)?;

    // Create subdirectories
    for dir in &config.directories {
        fs::create_dir_all(output_dir.join(dir))?;
    }

    let mut files_created = Vec::new();

    // Create files
    for file_config in &config.files {
        let file_path = output_dir.join(&file_config.path);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        match file_config.file_type {
            FileType::Text => {
                let content = generate_text_content(file_config.size_kb, Compressibility::Medium);
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

/// Generate text content with specified size and compressibility
fn generate_text_content(size_kb: usize, compressibility: Compressibility) -> Vec<u8> {
    let target_size = size_kb * 1024;
    let mut content = Vec::with_capacity(target_size);
    let mut rng = StdRng::seed_from_u64(42);

    match compressibility {
        Compressibility::High => {
            // Highly compressible - repeated character
            content.resize(target_size, b'A');
        }
        Compressibility::Medium => {
            // Repeating pattern
            let pattern = b"The quick brown fox jumps over the lazy dog. ";
            while content.len() < target_size {
                let remaining = target_size - content.len();
                let to_copy = remaining.min(pattern.len());
                content.extend_from_slice(&pattern[..to_copy]);
            }
        }
        Compressibility::Low => {
            // Mix of lorem ipsum and structured data
            let words = [
                "lorem",
                "ipsum",
                "dolor",
                "sit",
                "amet",
                "consectetur",
                "adipiscing",
                "elit",
                "sed",
                "do",
                "eiusmod",
                "tempor",
                "incididunt",
                "ut",
                "labore",
                "et",
                "dolore",
                "magna",
            ];

            while content.len() < target_size {
                // Alternate between text paragraphs and structured data
                if rng.random_bool(0.5) {
                    // Generate paragraph
                    let word_count = rng.random_range(50..200);
                    for _ in 0..word_count {
                        let word = words[rng.random_range(0..words.len())];
                        content.extend_from_slice(word.as_bytes());
                        content.push(b' ');
                    }
                    content.extend_from_slice(b"\n\n");
                } else {
                    // Generate structured data (JSON-like or CSV-like)
                    if rng.random_bool(0.5) {
                        let id = rng.random_range(1..1000);
                        let value: String = (0..10)
                            .map(|_| (b'a' + rng.random::<u8>() % 26) as char)
                            .collect();
                        let json = format!("{{\"id\": {id}, \"value\": \"{value}\"}}\n");
                        content.extend_from_slice(json.as_bytes());
                    } else {
                        let csv = format!(
                            "{},{},{:.3}\n",
                            rng.random_range(1..100),
                            (b'A' + rng.random::<u8>() % 26) as char,
                            rng.random::<f32>()
                        );
                        content.extend_from_slice(csv.as_bytes());
                    }
                }
            }
        }
    }

    content.truncate(target_size);
    content
}

/// Generate binary content with specified size
fn generate_binary_content(size_kb: usize) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(42);
    let size_bytes = size_kb * 1024;
    let mut content = vec![0u8; size_bytes];
    rng.fill(&mut content[..]);
    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_simple_data_generation() {
        let temp_dir = TempDir::new().unwrap();
        let config = TestDataConfig::simple();
        let result = generate_test_data(temp_dir.path(), &config).unwrap();

        assert_eq!(result.files_created.len(), 3);
        assert!(result.base_path.join("readme.txt").exists());
        assert!(result.base_path.join("data.txt").exists());
        assert!(result.base_path.join("config.ini").exists());
    }

    #[test]
    fn test_nested_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config = TestDataConfig::nested();
        let result = generate_test_data(temp_dir.path(), &config).unwrap();

        assert!(
            result
                .base_path
                .join("level1/level2/level3/level4/deep.txt")
                .exists()
        );
    }
}
