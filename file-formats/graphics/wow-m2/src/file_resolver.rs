//! FileDataID resolution system for M2 chunked format
//!
//! This module provides traits and implementations for resolving FileDataIDs
//! to file paths and loading external files referenced by M2 models.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{M2Error, Result};

/// Trait for resolving FileDataIDs to file paths and loading external files
pub trait FileResolver {
    /// Resolve a FileDataID to a file path
    fn resolve_file_data_id(&self, id: u32) -> Result<String>;

    /// Load file data by FileDataID
    fn load_file_by_id(&self, id: u32) -> Result<Vec<u8>>;

    /// Load a skin file by FileDataID
    fn load_skin_by_id(&self, id: u32) -> Result<Vec<u8>> {
        self.load_file_by_id(id)
    }

    /// Load an animation file by FileDataID
    fn load_animation_by_id(&self, id: u32) -> Result<Vec<u8>> {
        self.load_file_by_id(id)
    }

    /// Load a texture file by FileDataID
    fn load_texture_by_id(&self, id: u32) -> Result<Vec<u8>> {
        self.load_file_by_id(id)
    }

    /// Load a physics file by FileDataID
    fn load_physics_by_id(&self, id: &u32) -> Result<Vec<u8>> {
        self.load_file_by_id(*id)
    }

    /// Load a skeleton file by FileDataID
    fn load_skeleton_by_id(&self, id: &u32) -> Result<Vec<u8>> {
        self.load_file_by_id(*id)
    }

    /// Load a bone file by FileDataID
    fn load_bone_by_id(&self, id: &u32) -> Result<Vec<u8>> {
        self.load_file_by_id(*id)
    }
}

/// A file resolver that uses a listfile mapping from FileDataID to file path
/// This is the most common implementation for WoW file resolution
#[derive(Debug)]
pub struct ListfileResolver {
    /// Mapping from FileDataID to file path
    id_to_path: HashMap<u32, String>,
    /// Base path for resolving file paths
    base_path: Option<PathBuf>,
}

impl ListfileResolver {
    /// Create a new empty listfile resolver
    pub fn new() -> Self {
        Self {
            id_to_path: HashMap::new(),
            base_path: None,
        }
    }

    /// Create a new listfile resolver with a base path
    pub fn with_base_path<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            id_to_path: HashMap::new(),
            base_path: Some(base_path.as_ref().to_path_buf()),
        }
    }

    /// Load mappings from a CSV file (FileDataID;filepath format)
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let contents = fs::read_to_string(path)
            .map_err(|e| M2Error::ExternalFileError(format!("Failed to read listfile: {}", e)))?;

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse CSV format: FileDataID;filepath
            let parts: Vec<&str> = line.split(';').collect();
            if parts.len() >= 2
                && let Ok(id) = parts[0].parse::<u32>()
            {
                let path = parts[1].to_string();
                self.id_to_path.insert(id, path);
            }
        }

        Ok(())
    }

    /// Load mappings from a simple text file (one "ID filepath" per line)
    pub fn load_from_text<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let contents = fs::read_to_string(path)
            .map_err(|e| M2Error::ExternalFileError(format!("Failed to read listfile: {}", e)))?;

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse text format: ID filepath
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2
                && let Ok(id) = parts[0].parse::<u32>()
            {
                let path = parts[1..].join(" "); // Handle paths with spaces
                self.id_to_path.insert(id, path);
            }
        }

        Ok(())
    }

    /// Add a manual mapping from FileDataID to file path
    pub fn add_mapping<S: Into<String>>(&mut self, id: u32, path: S) {
        self.id_to_path.insert(id, path.into());
    }

    /// Remove a mapping by FileDataID
    pub fn remove_mapping(&mut self, id: u32) -> Option<String> {
        self.id_to_path.remove(&id)
    }

    /// Get the number of mappings
    pub fn len(&self) -> usize {
        self.id_to_path.len()
    }

    /// Check if the resolver is empty
    pub fn is_empty(&self) -> bool {
        self.id_to_path.is_empty()
    }

    /// Set the base path for file resolution
    pub fn set_base_path<P: AsRef<Path>>(&mut self, base_path: P) {
        self.base_path = Some(base_path.as_ref().to_path_buf());
    }

    /// Get the resolved absolute path for a file path
    fn resolve_path(&self, file_path: &str) -> PathBuf {
        match &self.base_path {
            Some(base) => base.join(file_path),
            None => PathBuf::from(file_path),
        }
    }
}

impl Default for ListfileResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl FileResolver for ListfileResolver {
    fn resolve_file_data_id(&self, id: u32) -> Result<String> {
        self.id_to_path
            .get(&id)
            .cloned()
            .ok_or(M2Error::UnknownFileDataId(id))
    }

    fn load_file_by_id(&self, id: u32) -> Result<Vec<u8>> {
        let file_path = self.resolve_file_data_id(id)?;
        let absolute_path = self.resolve_path(&file_path);

        fs::read(&absolute_path).map_err(|e| {
            M2Error::ExternalFileError(format!(
                "Failed to load file {} (ID {}): {}",
                absolute_path.display(),
                id,
                e
            ))
        })
    }
}

/// A simple file resolver that only works with file paths (no FileDataID resolution)
/// This is useful for testing or when working with extracted files
#[derive(Debug)]
pub struct PathResolver {
    base_path: PathBuf,
}

impl PathResolver {
    /// Create a new path resolver with a base directory
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Load a file by relative path
    pub fn load_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let full_path = self.base_path.join(path.as_ref());
        fs::read(&full_path).map_err(|e| {
            M2Error::ExternalFileError(format!(
                "Failed to load file {}: {}",
                full_path.display(),
                e
            ))
        })
    }
}

impl FileResolver for PathResolver {
    fn resolve_file_data_id(&self, id: u32) -> Result<String> {
        // This resolver doesn't support FileDataID resolution
        Err(M2Error::UnknownFileDataId(id))
    }

    fn load_file_by_id(&self, id: u32) -> Result<Vec<u8>> {
        // This resolver doesn't support FileDataID-based loading
        Err(M2Error::UnknownFileDataId(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_listfile_resolver_csv_format() {
        let mut resolver = ListfileResolver::new();

        // Create a temporary CSV listfile
        let mut listfile = NamedTempFile::new().unwrap();
        writeln!(listfile, "# Comment line").unwrap();
        writeln!(listfile, "123456;World\\Maps\\Azeroth\\Azeroth.wdt").unwrap();
        writeln!(listfile, "789012;Creature\\Human\\Male\\HumanMale.m2").unwrap();
        writeln!(listfile).unwrap(); // Empty line
        listfile.flush().unwrap();

        resolver.load_from_csv(listfile.path()).unwrap();

        assert_eq!(resolver.len(), 2);
        assert_eq!(
            resolver.resolve_file_data_id(123456).unwrap(),
            "World\\Maps\\Azeroth\\Azeroth.wdt"
        );
        assert_eq!(
            resolver.resolve_file_data_id(789012).unwrap(),
            "Creature\\Human\\Male\\HumanMale.m2"
        );

        // Test unknown ID
        assert!(resolver.resolve_file_data_id(999999).is_err());
    }

    #[test]
    fn test_listfile_resolver_text_format() {
        let mut resolver = ListfileResolver::new();

        // Create a temporary text listfile
        let mut listfile = NamedTempFile::new().unwrap();
        writeln!(listfile, "# Comment line").unwrap();
        writeln!(listfile, "123456 World/Maps/Azeroth/Azeroth.wdt").unwrap();
        writeln!(listfile, "789012 Creature/Human/Male/HumanMale.m2").unwrap();
        writeln!(listfile, "555666 Path with spaces/file.blp").unwrap();
        listfile.flush().unwrap();

        resolver.load_from_text(listfile.path()).unwrap();

        assert_eq!(resolver.len(), 3);
        assert_eq!(
            resolver.resolve_file_data_id(555666).unwrap(),
            "Path with spaces/file.blp"
        );
    }

    #[test]
    fn test_listfile_resolver_manual_mappings() {
        let mut resolver = ListfileResolver::new();

        resolver.add_mapping(12345, "test/file.m2");
        resolver.add_mapping(67890, "another/file.blp");

        assert_eq!(resolver.len(), 2);
        assert_eq!(
            resolver.resolve_file_data_id(12345).unwrap(),
            "test/file.m2"
        );

        let removed = resolver.remove_mapping(12345);
        assert_eq!(removed, Some("test/file.m2".to_string()));
        assert_eq!(resolver.len(), 1);
    }

    #[test]
    fn test_path_resolver() {
        let temp_dir = tempfile::tempdir().unwrap();
        let resolver = PathResolver::new(temp_dir.path());

        // PathResolver should not support FileDataID resolution
        assert!(resolver.resolve_file_data_id(123).is_err());
        assert!(resolver.load_file_by_id(123).is_err());

        // Create a test file
        let test_file_path = temp_dir.path().join("test.txt");
        fs::write(&test_file_path, b"test content").unwrap();

        let content = resolver.load_file("test.txt").unwrap();
        assert_eq!(content, b"test content");
    }
}
