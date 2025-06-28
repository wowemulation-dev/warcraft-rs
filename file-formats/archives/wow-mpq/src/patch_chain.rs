//! Patch chain implementation for handling multiple MPQ archives with priority
//!
//! This module provides functionality for managing multiple MPQ archives in a chain,
//! where files in higher-priority archives override those in lower-priority ones.
//! This is essential for World of Warcraft's patching system.

use crate::{Archive, Error, FileEntry, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A chain of MPQ archives with priority ordering
///
/// `PatchChain` manages multiple MPQ archives where files in higher-priority
/// archives override those in lower-priority ones. This mimics how World of Warcraft
/// handles its patch system, where patch-N.MPQ files override files in the base archives.
///
/// # Examples
///
/// ```no_run
/// use wow_mpq::PatchChain;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut chain = PatchChain::new();
///
/// // Add base archive with lowest priority
/// chain.add_archive("Data/common.MPQ", 0)?;
///
/// // Add patches with increasing priority
/// chain.add_archive("Data/patch.MPQ", 100)?;
/// chain.add_archive("Data/patch-2.MPQ", 200)?;
/// chain.add_archive("Data/patch-3.MPQ", 300)?;
///
/// // Extract file - will use the highest priority version available
/// let data = chain.read_file("Interface/Icons/INV_Misc_QuestionMark.blp")?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct PatchChain {
    /// Archives ordered by priority (highest first)
    archives: Vec<ChainEntry>,
    /// Cache of file locations for quick lookup
    file_map: HashMap<String, usize>,
}

#[derive(Debug)]
struct ChainEntry {
    /// The MPQ archive
    archive: Archive,
    /// Priority (higher numbers override lower)
    priority: i32,
    /// Path to the archive file
    path: PathBuf,
}

impl PatchChain {
    /// Create a new empty patch chain
    pub fn new() -> Self {
        Self {
            archives: Vec::new(),
            file_map: HashMap::new(),
        }
    }

    /// Add an archive to the chain with a specific priority
    ///
    /// Archives with higher priority values will override files in archives
    /// with lower priority values.
    ///
    /// # Parameters
    /// - `path`: Path to the MPQ archive
    /// - `priority`: Priority value (higher = overrides lower)
    ///
    /// # Typical priority values
    /// - 0: Base archives (common.MPQ, expansion.MPQ)
    /// - 100-999: Official patches (patch.MPQ, patch-2.MPQ, etc.)
    /// - 1000+: Custom patches or mods
    pub fn add_archive<P: AsRef<Path>>(&mut self, path: P, priority: i32) -> Result<()> {
        let path = path.as_ref();
        let archive = Archive::open(path)?;

        // Insert in sorted order (highest priority first)
        let entry = ChainEntry {
            archive,
            priority,
            path: path.to_path_buf(),
        };

        let insert_pos = self
            .archives
            .iter()
            .position(|e| e.priority < priority)
            .unwrap_or(self.archives.len());

        self.archives.insert(insert_pos, entry);

        // Rebuild file map
        self.rebuild_file_map()?;

        Ok(())
    }

    /// Remove an archive from the chain
    pub fn remove_archive<P: AsRef<Path>>(&mut self, path: P) -> Result<bool> {
        let path = path.as_ref();

        if let Some(pos) = self.archives.iter().position(|e| e.path == path) {
            self.archives.remove(pos);
            self.rebuild_file_map()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Clear all archives from the chain
    pub fn clear(&mut self) {
        self.archives.clear();
        self.file_map.clear();
    }

    /// Get the number of archives in the chain
    pub fn archive_count(&self) -> usize {
        self.archives.len()
    }

    /// Read a file from the chain
    ///
    /// Returns the file from the highest-priority archive that contains it.
    pub fn read_file(&mut self, filename: &str) -> Result<Vec<u8>> {
        // Normalize the filename to use backslashes for lookup
        let normalized_filename = crate::path::normalize_mpq_path(filename);

        if let Some(&archive_idx) = self.file_map.get(&normalized_filename) {
            // Use the original filename for read_file (it handles both separators)
            self.archives[archive_idx].archive.read_file(filename)
        } else {
            Err(Error::FileNotFound(filename.to_string()))
        }
    }

    /// Check if a file exists in the chain
    pub fn contains_file(&self, filename: &str) -> bool {
        let normalized_filename = crate::path::normalize_mpq_path(filename);
        self.file_map.contains_key(&normalized_filename)
    }

    /// Find which archive contains a file
    ///
    /// Returns the path to the archive containing the file, or None if not found.
    pub fn find_file_archive(&self, filename: &str) -> Option<&Path> {
        let normalized_filename = crate::path::normalize_mpq_path(filename);
        self.file_map
            .get(&normalized_filename)
            .map(|&idx| self.archives[idx].path.as_path())
    }

    /// List all files in the chain
    ///
    /// Returns a deduplicated list of all files across all archives,
    /// with file information from the highest-priority archive for each file.
    pub fn list(&mut self) -> Result<Vec<FileEntry>> {
        let mut seen = HashMap::new();
        let mut result = Vec::new();

        // Process archives in priority order (highest first)
        for (idx, entry) in self.archives.iter_mut().enumerate() {
            match entry.archive.list() {
                Ok(files) => {
                    for file in files {
                        // Only add if we haven't seen this file yet
                        if !seen.contains_key(&file.name) {
                            seen.insert(file.name.clone(), idx);
                            result.push(file);
                        }
                    }
                }
                Err(_) => {
                    // Try list_all if no listfile
                    if let Ok(files) = entry.archive.list_all() {
                        for file in files {
                            if !seen.contains_key(&file.name) {
                                seen.insert(file.name.clone(), idx);
                                result.push(file);
                            }
                        }
                    }
                }
            }
        }

        // Sort by name for consistent output
        result.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(result)
    }

    /// Get information about all archives in the chain
    pub fn get_chain_info(&mut self) -> Vec<ChainInfo> {
        self.archives
            .iter_mut()
            .filter_map(|entry| {
                entry.archive.get_info().ok().map(|info| ChainInfo {
                    path: entry.path.clone(),
                    priority: entry.priority,
                    file_count: info.file_count,
                    archive_size: info.file_size,
                    format_version: info.format_version,
                })
            })
            .collect()
    }

    /// Rebuild the internal file map
    fn rebuild_file_map(&mut self) -> Result<()> {
        self.file_map.clear();

        // Process archives in priority order (highest first)
        for (idx, entry) in self.archives.iter_mut().enumerate() {
            // Try to get file list
            let files = match entry.archive.list() {
                Ok(files) => files,
                Err(_) => {
                    // Try list_all if no listfile
                    match entry.archive.list_all() {
                        Ok(files) => files,
                        Err(_) => continue, // Skip this archive
                    }
                }
            };

            // Add files to map (only if not already present from higher priority)
            for file in files {
                self.file_map.entry(file.name).or_insert(idx);
            }
        }

        Ok(())
    }

    /// Extract multiple files efficiently
    ///
    /// This method extracts multiple files in a single pass, which can be more
    /// efficient than calling `read_file` multiple times.
    pub fn extract_files(&mut self, filenames: &[&str]) -> Vec<(String, Result<Vec<u8>>)> {
        filenames
            .iter()
            .map(|&filename| {
                let result = self.read_file(filename);
                (filename.to_string(), result)
            })
            .collect()
    }

    /// Get a reference to a specific archive by path
    pub fn get_archive<P: AsRef<Path>>(&self, path: P) -> Option<&Archive> {
        let path = path.as_ref();
        self.archives
            .iter()
            .find(|e| e.path == path)
            .map(|e| &e.archive)
    }

    /// Get archive priority by path
    pub fn get_priority<P: AsRef<Path>>(&self, path: P) -> Option<i32> {
        let path = path.as_ref();
        self.archives
            .iter()
            .find(|e| e.path == path)
            .map(|e| e.priority)
    }

    /// Load multiple archives in parallel and build a patch chain
    ///
    /// This method loads all archives concurrently using rayon, then builds
    /// the patch chain with proper priority ordering. This is significantly
    /// faster than loading archives sequentially.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_mpq::PatchChain;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let archives = vec![
    ///     ("Data/common.MPQ", 0),
    ///     ("Data/patch.MPQ", 100),
    ///     ("Data/patch-2.MPQ", 200),
    ///     ("Data/patch-3.MPQ", 300),
    /// ];
    ///
    /// let chain = PatchChain::from_archives_parallel(archives)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_archives_parallel<P: AsRef<Path> + Sync>(archives: Vec<(P, i32)>) -> Result<Self> {
        use rayon::prelude::*;

        // Load all archives in parallel
        let loaded_archives: Result<Vec<_>> = archives
            .par_iter()
            .map(|(path, priority)| {
                let path_ref = path.as_ref();
                Archive::open(path_ref).map(|archive| ChainEntry {
                    archive,
                    priority: *priority,
                    path: path_ref.to_path_buf(),
                })
            })
            .collect();

        let mut loaded_archives = loaded_archives?;

        // Sort by priority (highest first)
        loaded_archives.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut chain = Self {
            archives: loaded_archives,
            file_map: HashMap::new(),
        };

        // Build the file map
        chain.rebuild_file_map()?;

        Ok(chain)
    }

    /// Add multiple archives to the chain in parallel
    ///
    /// This method loads multiple archives concurrently and adds them to the
    /// existing chain with their specified priorities.
    pub fn add_archives_parallel<P: AsRef<Path> + Sync>(
        &mut self,
        archives: Vec<(P, i32)>,
    ) -> Result<()> {
        use rayon::prelude::*;

        // Load new archives in parallel
        let new_archives: Result<Vec<_>> = archives
            .par_iter()
            .map(|(path, priority)| {
                let path_ref = path.as_ref();
                Archive::open(path_ref).map(|archive| ChainEntry {
                    archive,
                    priority: *priority,
                    path: path_ref.to_path_buf(),
                })
            })
            .collect();

        let new_archives = new_archives?;

        // Add each new archive at the correct position
        for entry in new_archives {
            let insert_pos = self
                .archives
                .iter()
                .position(|e| e.priority < entry.priority)
                .unwrap_or(self.archives.len());

            self.archives.insert(insert_pos, entry);
        }

        // Rebuild file map
        self.rebuild_file_map()?;

        Ok(())
    }

    /// Update the priority of an existing archive
    pub fn set_priority<P: AsRef<Path>>(&mut self, path: P, new_priority: i32) -> Result<()> {
        let path = path.as_ref();

        // Find and remove the archive
        let archive_idx = self
            .archives
            .iter()
            .position(|e| e.path == path)
            .ok_or_else(|| Error::InvalidFormat("Archive not found in chain".to_string()))?;

        let mut entry = self.archives.remove(archive_idx);
        entry.priority = new_priority;

        // Re-insert at correct position
        let insert_pos = self
            .archives
            .iter()
            .position(|e| e.priority < new_priority)
            .unwrap_or(self.archives.len());

        self.archives.insert(insert_pos, entry);

        // Rebuild file map
        self.rebuild_file_map()?;

        Ok(())
    }
}

impl Default for PatchChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about an archive in the chain
#[derive(Debug, Clone)]
pub struct ChainInfo {
    /// Path to the archive
    pub path: PathBuf,
    /// Priority in the chain
    pub priority: i32,
    /// Number of files in the archive
    pub file_count: usize,
    /// Total size of the archive
    pub archive_size: u64,
    /// MPQ format version
    pub format_version: crate::FormatVersion,
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{ArchiveBuilder, ListfileOption};
    use tempfile::TempDir;

    fn create_test_archive(dir: &Path, name: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let path = dir.join(name);
        let mut builder = ArchiveBuilder::new().listfile_option(ListfileOption::Generate);

        for (filename, data) in files {
            builder = builder.add_file_data(data.to_vec(), filename);
        }

        builder.build(&path).unwrap();
        path
    }

    #[test]
    fn test_patch_chain_priority() {
        let temp = TempDir::new().unwrap();

        // Create base archive
        let base_files: Vec<(&str, &[u8])> = vec![
            ("file1.txt", b"base file1"),
            ("file2.txt", b"base file2"),
            ("file3.txt", b"base file3"),
        ];
        let base_path = create_test_archive(temp.path(), "base.mpq", &base_files);

        // Create patch archive that overrides file2
        let patch_files: Vec<(&str, &[u8])> =
            vec![("file2.txt", b"patched file2"), ("file4.txt", b"new file4")];
        let patch_path = create_test_archive(temp.path(), "patch.mpq", &patch_files);

        // Build chain
        let mut chain = PatchChain::new();
        chain.add_archive(&base_path, 0).unwrap();
        chain.add_archive(&patch_path, 100).unwrap();

        // Test file priority
        assert_eq!(chain.read_file("file1.txt").unwrap(), b"base file1");
        assert_eq!(chain.read_file("file2.txt").unwrap(), b"patched file2"); // Override
        assert_eq!(chain.read_file("file3.txt").unwrap(), b"base file3");
        assert_eq!(chain.read_file("file4.txt").unwrap(), b"new file4");
    }

    #[test]
    fn test_patch_chain_listing() {
        let temp = TempDir::new().unwrap();

        // Create archives
        let base_files: Vec<(&str, &[u8])> = vec![("file1.txt", b"data1"), ("file2.txt", b"data2")];
        let patch_files: Vec<(&str, &[u8])> =
            vec![("file2.txt", b"patch2"), ("file3.txt", b"data3")];

        let base_path = create_test_archive(temp.path(), "base.mpq", &base_files);
        let patch_path = create_test_archive(temp.path(), "patch.mpq", &patch_files);

        // Build chain
        let mut chain = PatchChain::new();
        chain.add_archive(&base_path, 0).unwrap();
        chain.add_archive(&patch_path, 100).unwrap();

        // List should show all unique files
        let files = chain.list().unwrap();

        // Filter out the listfile
        let files: Vec<_> = files
            .into_iter()
            .filter(|f| f.name != "(listfile)")
            .collect();

        assert_eq!(files.len(), 3);

        let names: Vec<_> = files.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"file1.txt"));
        assert!(names.contains(&"file2.txt"));
        assert!(names.contains(&"file3.txt"));
    }

    #[test]
    fn test_find_file_archive() {
        let temp = TempDir::new().unwrap();

        let base_files: Vec<(&str, &[u8])> = vec![("file1.txt", b"data")];
        let patch_files: Vec<(&str, &[u8])> = vec![("file2.txt", b"data")];

        let base_path = create_test_archive(temp.path(), "base.mpq", &base_files);
        let patch_path = create_test_archive(temp.path(), "patch.mpq", &patch_files);

        let mut chain = PatchChain::new();
        chain.add_archive(&base_path, 0).unwrap();
        chain.add_archive(&patch_path, 100).unwrap();

        assert_eq!(
            chain.find_file_archive("file1.txt"),
            Some(base_path.as_path())
        );
        assert_eq!(
            chain.find_file_archive("file2.txt"),
            Some(patch_path.as_path())
        );
        assert_eq!(chain.find_file_archive("nonexistent.txt"), None);
    }

    #[test]
    fn test_remove_archive() {
        let temp = TempDir::new().unwrap();

        let files: Vec<(&str, &[u8])> = vec![("file.txt", b"data")];
        let path = create_test_archive(temp.path(), "test.mpq", &files);

        let mut chain = PatchChain::new();
        chain.add_archive(&path, 0).unwrap();

        assert!(chain.contains_file("file.txt"));
        assert!(chain.remove_archive(&path).unwrap());
        assert!(!chain.contains_file("file.txt"));
        assert!(!chain.remove_archive(&path).unwrap()); // Already removed
    }

    #[test]
    fn test_priority_reordering() {
        let temp = TempDir::new().unwrap();

        let files: Vec<(&str, &[u8])> = vec![("file.txt", b"data")];
        let path1 = create_test_archive(temp.path(), "test1.mpq", &files);
        let path2 = create_test_archive(temp.path(), "test2.mpq", &files);

        let mut chain = PatchChain::new();
        chain.add_archive(&path1, 100).unwrap();
        chain.add_archive(&path2, 50).unwrap();

        // path1 should be first (higher priority)
        assert_eq!(chain.archives[0].priority, 100);

        // Update priority
        chain.set_priority(&path2, 150).unwrap();

        // Now path2 should be first
        assert_eq!(chain.archives[0].priority, 150);
    }

    #[test]
    fn test_parallel_patch_chain_loading() {
        let temp = TempDir::new().unwrap();

        // Create multiple test archives
        let mut archive_paths = Vec::new();
        for i in 0..5 {
            let common_content = format!("Common content v{}", i);
            let unique_name = format!("unique_{}.txt", i);
            let unique_content = format!("Unique to archive {}", i);
            let files: Vec<(&str, &[u8])> = vec![
                ("common.txt", common_content.as_bytes()),
                (&unique_name, unique_content.as_bytes()),
            ];
            let path = create_test_archive(temp.path(), &format!("archive_{}.mpq", i), &files);
            archive_paths.push((path, i * 100)); // Increasing priorities
        }

        // Load sequentially and measure time
        let start = std::time::Instant::now();
        let mut chain_seq = PatchChain::new();
        for (path, priority) in &archive_paths {
            chain_seq.add_archive(path, *priority).unwrap();
        }
        let seq_duration = start.elapsed();

        // Load in parallel
        let start = std::time::Instant::now();
        let mut chain_par = PatchChain::from_archives_parallel(archive_paths.clone()).unwrap();
        let par_duration = start.elapsed();

        // Verify both chains have the same content
        assert_eq!(
            chain_seq.list().unwrap().len(),
            chain_par.list().unwrap().len()
        );

        // Verify priority ordering (highest priority archive should win)
        let common_content = chain_par.read_file("common.txt").unwrap();
        assert_eq!(common_content, b"Common content v4"); // Archive 4 has highest priority (400)

        // Verify all unique files are accessible
        for i in 0..5 {
            let unique_file = format!("unique_{}.txt", i);
            let content = chain_par.read_file(&unique_file).unwrap();
            assert_eq!(content, format!("Unique to archive {}", i).as_bytes());
        }

        // Parallel should be faster (or at least not significantly slower)
        println!("Sequential loading: {:?}", seq_duration);
        println!("Parallel loading: {:?}", par_duration);
    }

    #[test]
    fn test_add_archives_parallel() {
        let temp = TempDir::new().unwrap();

        // Create initial archive
        let base_files: Vec<(&str, &[u8])> = vec![("base.txt", b"base content")];
        let base_path = create_test_archive(temp.path(), "base.mpq", &base_files);

        // Create chain with base archive
        let mut chain = PatchChain::new();
        chain.add_archive(&base_path, 0).unwrap();

        // Create multiple patch archives
        let mut patch_archives = Vec::new();
        for i in 1..=3 {
            let patch_name = format!("patch_{}.txt", i);
            let patch_content = format!("Patch {} content", i);
            let common_content = format!("Common from patch {}", i);
            let files: Vec<(&str, &[u8])> = vec![
                (&patch_name, patch_content.as_bytes()),
                ("common.txt", common_content.as_bytes()),
            ];
            let path = create_test_archive(temp.path(), &format!("patch_{}.mpq", i), &files);
            patch_archives.push((path, i * 100));
        }

        // Add patches in parallel
        chain.add_archives_parallel(patch_archives).unwrap();

        // Verify all files are accessible
        assert_eq!(chain.read_file("base.txt").unwrap(), b"base content");
        assert_eq!(chain.read_file("patch_1.txt").unwrap(), b"Patch 1 content");
        assert_eq!(chain.read_file("patch_2.txt").unwrap(), b"Patch 2 content");
        assert_eq!(chain.read_file("patch_3.txt").unwrap(), b"Patch 3 content");

        // Verify priority (patch 3 has highest priority)
        assert_eq!(
            chain.read_file("common.txt").unwrap(),
            b"Common from patch 3"
        );

        // Verify chain info
        let info = chain.get_chain_info();
        assert_eq!(info.len(), 4); // base + 3 patches
    }

    #[test]
    fn test_parallel_loading_with_invalid_archive() {
        let temp = TempDir::new().unwrap();

        // Create some valid archives
        let mut archives = Vec::new();
        for i in 0..2 {
            let file_name = format!("file_{}.txt", i);
            let files: Vec<(&str, &[u8])> = vec![(&file_name, b"content")];
            let path = create_test_archive(temp.path(), &format!("valid_{}.mpq", i), &files);
            archives.push((path, i * 100));
        }

        // Add a non-existent archive
        archives.push((temp.path().join("nonexistent.mpq"), 200));

        // Try to load in parallel - should fail
        let result = PatchChain::from_archives_parallel(archives);
        assert!(result.is_err());
    }
}
