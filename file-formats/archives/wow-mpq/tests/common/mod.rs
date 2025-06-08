//! Common test utilities and fixtures

#![allow(dead_code)]

pub mod test_helpers;

use std::path::Path;
use tempfile::TempDir;

/// Create a temporary directory for tests
pub fn temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

/// Generate test data of a specific size
pub fn generate_test_data(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

/// Generate repetitive test data (good for compression tests)
pub fn generate_repetitive_data(pattern: &[u8], total_size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(total_size);
    while data.len() < total_size {
        let chunk_size = (total_size - data.len()).min(pattern.len());
        data.extend_from_slice(&pattern[..chunk_size]);
    }
    data
}

/// Create a test file with specific content
pub fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> std::path::PathBuf {
    use std::fs;
    let path = dir.join(name);
    fs::write(&path, content).expect("Failed to write test file");
    path
}
