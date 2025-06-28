//! Parallel processing utilities for MPQ archives
//!
//! This module provides utilities for processing multiple MPQ archives
//! or multiple files within archives in parallel using rayon.

use crate::{Archive, Result};
use std::path::{Path, PathBuf};

/// Extract a file from multiple archives in parallel
///
/// This function opens multiple MPQ archives and attempts to extract the same
/// file from each archive in parallel. This is useful for comparing versions
/// of files across different patches or archives.
///
/// # Examples
///
/// ```no_run
/// use wow_mpq::parallel::extract_from_multiple_archives;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let archives = vec!["patch-1.mpq", "patch-2.mpq", "patch-3.mpq"];
/// let results = extract_from_multiple_archives(&archives, "Interface/Icons/icon.blp")?;
///
/// for (path, data) in results {
///     println!("File from {:?}: {} bytes", path, data.len());
/// }
/// # Ok(())
/// # }
/// ```
pub fn extract_from_multiple_archives<P: AsRef<Path> + Sync>(
    archives: &[P],
    file_name: &str,
) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    use rayon::prelude::*;

    archives
        .par_iter()
        .map(|path| {
            let path_ref = path.as_ref();
            match Archive::open(path_ref) {
                Ok(mut archive) => match archive.read_file(file_name) {
                    Ok(data) => Ok((path_ref.to_path_buf(), data)),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        })
        .collect()
}

/// Extract multiple files from multiple archives in parallel
///
/// This function processes multiple archives and extracts multiple files from
/// each archive, all in parallel. Results are returned as a nested structure.
///
/// # Examples
///
/// ```no_run
/// use wow_mpq::parallel::extract_multiple_from_multiple_archives;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let archives = vec!["data.mpq", "patch.mpq"];
/// let files = vec!["file1.txt", "file2.txt", "file3.txt"];
///
/// let results = extract_multiple_from_multiple_archives(&archives, &files)?;
///
/// for (archive_path, file_results) in results {
///     println!("Archive: {:?}", archive_path);
///     for (file_name, data) in file_results {
///         println!("  {}: {} bytes", file_name, data.len());
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub fn extract_multiple_from_multiple_archives<P: AsRef<Path> + Sync>(
    archives: &[P],
    file_names: &[&str],
) -> Result<Vec<(PathBuf, Vec<(String, Vec<u8>)>)>> {
    use rayon::prelude::*;

    archives
        .par_iter()
        .map(|path| {
            let path_ref = path.as_ref();
            let mut archive = Archive::open(path_ref)?;

            let files: Result<Vec<_>> = file_names
                .iter()
                .map(|&name| archive.read_file(name).map(|data| (name.to_string(), data)))
                .collect();

            Ok((path_ref.to_path_buf(), files?))
        })
        .collect()
}

/// Search for files matching a pattern across multiple archives in parallel
///
/// This function searches for files matching a pattern across multiple archives
/// and returns the archives that contain matching files.
///
/// # Examples
///
/// ```no_run
/// use wow_mpq::parallel::search_in_multiple_archives;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let archives = vec!["data.mpq", "patch-1.mpq", "patch-2.mpq"];
/// let results = search_in_multiple_archives(&archives, "Interface/Icons")?;
///
/// for (archive_path, matching_files) in results {
///     println!("Archive {:?} contains {} matching files",
///              archive_path, matching_files.len());
/// }
/// # Ok(())
/// # }
/// ```
pub fn search_in_multiple_archives<P: AsRef<Path> + Sync>(
    archives: &[P],
    pattern: &str,
) -> Result<Vec<(PathBuf, Vec<String>)>> {
    use rayon::prelude::*;

    archives
        .par_iter()
        .map(|path| {
            let path_ref = path.as_ref();
            let mut archive = Archive::open(path_ref)?;
            let files = archive.list()?;

            let matching: Vec<String> = files
                .into_iter()
                .filter(|entry| entry.name.contains(pattern))
                .map(|entry| entry.name)
                .collect();

            Ok((path_ref.to_path_buf(), matching))
        })
        .collect()
}

/// Process archives in parallel with a custom function
///
/// This is a generic function that allows processing multiple archives
/// in parallel with a custom closure.
///
/// # Examples
///
/// ```no_run
/// use wow_mpq::parallel::process_archives_parallel;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let archives = vec!["data.mpq", "patch.mpq"];
///
/// // Count files in each archive
/// let counts = process_archives_parallel(&archives, |mut archive| {
///     Ok(archive.list()?.len())
/// })?;
///
/// for (path, count) in archives.iter().zip(counts.iter()) {
///     println!("{:?}: {} files", path, count);
/// }
/// # Ok(())
/// # }
/// ```
pub fn process_archives_parallel<P, F, T>(archives: &[P], processor: F) -> Result<Vec<T>>
where
    P: AsRef<Path> + Sync,
    F: Fn(Archive) -> Result<T> + Sync,
    T: Send,
{
    use rayon::prelude::*;

    archives
        .par_iter()
        .map(|path| {
            let archive = Archive::open(path)?;
            processor(archive)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArchiveBuilder;
    use tempfile::TempDir;

    fn create_test_archives(count: usize) -> Result<(TempDir, Vec<PathBuf>)> {
        let temp_dir = TempDir::new()?;
        let mut paths = Vec::new();

        for i in 0..count {
            let path = temp_dir.path().join(format!("test_{}.mpq", i));
            let mut builder = ArchiveBuilder::new();

            // Add some test files
            builder = builder
                .add_file_data(
                    format!("Content from archive {}", i).into_bytes(),
                    "common.txt",
                )
                .add_file_data(
                    format!("Unique to archive {}", i).into_bytes(),
                    &format!("unique_{}.txt", i),
                );

            builder.build(&path)?;
            paths.push(path);
        }

        Ok((temp_dir, paths))
    }

    #[test]
    fn test_extract_from_multiple_archives() -> Result<()> {
        let (_temp_dir, archives) = create_test_archives(3)?;

        let results = extract_from_multiple_archives(&archives, "common.txt")?;

        assert_eq!(results.len(), 3);
        for (i, (path, data)) in results.iter().enumerate() {
            assert_eq!(path, &archives[i]);
            let content = String::from_utf8_lossy(data);
            assert!(content.contains(&format!("Content from archive {}", i)));
        }

        Ok(())
    }

    #[test]
    fn test_search_in_multiple_archives() -> Result<()> {
        let (_temp_dir, archives) = create_test_archives(3)?;

        let results = search_in_multiple_archives(&archives, "unique")?;

        assert_eq!(results.len(), 3);
        for (i, (_path, matches)) in results.iter().enumerate() {
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0], format!("unique_{}.txt", i));
        }

        Ok(())
    }

    #[test]
    fn test_process_archives_parallel() -> Result<()> {
        let (_temp_dir, archives) = create_test_archives(3)?;

        // Count files in each archive
        let counts = process_archives_parallel(&archives, |mut archive| Ok(archive.list()?.len()))?;

        assert_eq!(counts.len(), 3);
        for count in counts {
            // Archives have at least 2 files (the ones we added)
            // May have more due to listfile or attributes
            assert!(count >= 2);
        }

        Ok(())
    }
}
