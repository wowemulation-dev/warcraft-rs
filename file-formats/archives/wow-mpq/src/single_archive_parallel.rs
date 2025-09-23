//! Single archive parallel processing support
//!
//! This module provides utilities for reading multiple files from a single MPQ archive
//! in parallel. This is achieved by cloning file handles for each thread, allowing
//! concurrent reads without seek conflicts.

use crate::{Archive, Error, Result};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A thread-safe wrapper around an MPQ archive for parallel operations
///
/// `ParallelArchive` enables concurrent reads from a single MPQ archive by
/// giving each thread its own file handle. This avoids seek conflicts that
/// would occur with a shared file handle.
///
/// # Examples
///
/// ```no_run
/// use wow_mpq::single_archive_parallel::ParallelArchive;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let archive = ParallelArchive::open("data.mpq")?;
///
/// // Extract multiple files in parallel
/// let files = vec!["file1.txt", "file2.txt", "file3.txt"];
/// let results = archive.extract_files_parallel(&files)?;
///
/// for (filename, data) in results {
///     println!("{}: {} bytes", filename, data.len());
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ParallelArchive {
    /// Path to the archive file
    path: PathBuf,
    /// Cached file list for quick lookups
    file_list: Arc<Vec<String>>,
}

impl ParallelArchive {
    /// Open an MPQ archive for parallel processing
    ///
    /// This caches the file list upfront to enable efficient parallel operations.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Open archive and get file list
        let archive = Archive::open(&path)?;
        let entries = archive.list()?;
        let file_list = Arc::new(entries.into_iter().map(|e| e.name).collect());

        Ok(Self { path, file_list })
    }

    /// Extract multiple files from the archive in parallel
    ///
    /// Each file is extracted in a separate thread with its own file handle.
    /// This allows true parallel I/O without seek conflicts.
    ///
    /// # Returns
    /// A vector of (filename, data) tuples in the same order as the input
    pub fn extract_files_parallel(&self, filenames: &[&str]) -> Result<Vec<(String, Vec<u8>)>> {
        filenames
            .par_iter()
            .map(|&filename| {
                // Each thread opens its own file handle
                let data = self.read_file_with_new_handle(filename)?;
                Ok((filename.to_string(), data))
            })
            .collect()
    }

    /// Extract files matching a predicate in parallel
    ///
    /// This method lists all files in the archive and extracts those that
    /// match the given predicate function.
    pub fn extract_matching_parallel<F>(&self, predicate: F) -> Result<Vec<(String, Vec<u8>)>>
    where
        F: Fn(&str) -> bool + Sync,
    {
        // Get the cached file list
        let files = self.list_files();

        // Filter and extract in parallel
        files
            .par_iter()
            .filter(|name| predicate(name))
            .map(|filename| {
                let data = self.read_file_with_new_handle(filename)?;
                Ok((filename.clone(), data))
            })
            .collect()
    }

    /// Process files in parallel with a custom function
    ///
    /// This is the most flexible method, allowing custom processing of each file.
    pub fn process_files_parallel<F, T>(&self, filenames: &[&str], processor: F) -> Result<Vec<T>>
    where
        F: Fn(&str, Vec<u8>) -> Result<T> + Sync,
        T: Send,
    {
        filenames
            .par_iter()
            .map(|&filename| {
                let data = self.read_file_with_new_handle(filename)?;
                processor(filename, data)
            })
            .collect()
    }

    /// Read a file using a new file handle
    ///
    /// This is the core method that enables parallel reads. Each call opens
    /// a new file handle, avoiding conflicts with other threads.
    pub fn read_file_with_new_handle(&self, filename: &str) -> Result<Vec<u8>> {
        // Open a new file handle for this thread
        let archive = Archive::open(&self.path)?;

        // Read the file
        archive.read_file(filename)
    }

    /// Get the cached file list
    pub fn list_files(&self) -> &[String] {
        &self.file_list
    }

    /// Get the number of worker threads that will be used
    pub fn thread_count(&self) -> usize {
        rayon::current_num_threads()
    }

    /// Extract files in batches for better performance with many small files
    ///
    /// When extracting many small files, the overhead of opening file handles
    /// can dominate. This method processes files in batches per thread.
    pub fn extract_files_batched(
        &self,
        filenames: &[&str],
        batch_size: usize,
    ) -> Result<Vec<(String, Vec<u8>)>> {
        // Divide files into chunks
        let chunks: Vec<_> = filenames.chunks(batch_size).collect();

        // Process each chunk in parallel
        let results: Result<Vec<_>> = chunks
            .par_iter()
            .map(|chunk| {
                // Open one archive handle per batch
                let archive = Archive::open(&self.path)?;

                // Extract all files in this batch
                let mut batch_results = Vec::new();
                for &filename in chunk.iter() {
                    let data = archive.read_file(filename)?;
                    batch_results.push((filename.to_string(), data));
                }
                Ok(batch_results)
            })
            .collect();

        // Flatten the results
        Ok(results?.into_iter().flatten().collect())
    }
}

/// Configuration for parallel extraction operations
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Number of worker threads (None = use rayon default)
    pub num_threads: Option<usize>,
    /// Batch size for small file extraction
    pub batch_size: usize,
    /// Whether to skip files that fail to extract
    pub skip_errors: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: None,
            batch_size: 10,
            skip_errors: false,
        }
    }
}

impl ParallelConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of worker threads
    pub fn threads(mut self, num: usize) -> Self {
        self.num_threads = Some(num);
        self
    }

    /// Set the batch size for small files
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set whether to skip extraction errors
    pub fn skip_errors(mut self, skip: bool) -> Self {
        self.skip_errors = skip;
        self
    }
}

/// Extract files with custom configuration
///
/// This function efficiently handles large numbers of files by using batched extraction
/// to reduce resource pressure and prevent system overload.
pub fn extract_with_config<P: AsRef<Path>>(
    archive_path: P,
    filenames: &[&str],
    config: ParallelConfig,
) -> Result<Vec<(String, Result<Vec<u8>>)>> {
    // For large file counts, force batched extraction to prevent resource exhaustion
    let use_batched = filenames.len() > 1000;

    if use_batched {
        extract_with_config_batched(archive_path, filenames, config)
    } else {
        extract_with_config_unbatched(archive_path, filenames, config)
    }
}

/// Extract files using batched approach for better resource management
fn extract_with_config_batched<P: AsRef<Path>>(
    archive_path: P,
    filenames: &[&str],
    config: ParallelConfig,
) -> Result<Vec<(String, Result<Vec<u8>>)>> {
    let archive = ParallelArchive::open(archive_path)?;

    // Calculate appropriate batch size based on file count and available threads
    let num_threads = config
        .num_threads
        .unwrap_or_else(rayon::current_num_threads);
    let effective_batch_size = if filenames.len() > 5000 {
        // For very large extractions, use larger batches to reduce overhead
        std::cmp::max(config.batch_size, filenames.len() / (num_threads * 2))
    } else {
        config.batch_size
    };

    // Configure thread pool if specified
    let pool = if let Some(threads) = config.num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|e| {
                Error::Io(std::io::Error::other(format!(
                    "Failed to create thread pool: {e}"
                )))
            })?
    } else {
        rayon::ThreadPoolBuilder::new().build().unwrap()
    };

    // Execute batched extraction in the configured thread pool
    pool.install(|| {
        // Split files into chunks for batched processing
        let chunks: Vec<_> = filenames.chunks(effective_batch_size).collect();

        // Process chunks in parallel, each chunk using one Archive handle
        let batch_results: Result<Vec<_>> = chunks
            .par_iter()
            .map(|chunk| {
                // Open one archive handle per batch to limit resource usage
                let archive_handle = Archive::open(archive.path.as_path())?;

                // Process all files in this batch with the same handle
                let mut batch_results = Vec::with_capacity(chunk.len());
                for &filename in chunk.iter() {
                    let result = if config.skip_errors {
                        archive_handle.read_file(filename)
                    } else {
                        let data = archive_handle.read_file(filename)?;
                        Ok(data)
                    };
                    batch_results.push((filename.to_string(), result));
                }
                Ok(batch_results)
            })
            .collect();

        // Flatten results while preserving order
        match batch_results {
            Ok(batches) => Ok(batches.into_iter().flatten().collect()),
            Err(e) => Err(e),
        }
    })
}

/// Extract files using individual file approach for smaller file sets
fn extract_with_config_unbatched<P: AsRef<Path>>(
    archive_path: P,
    filenames: &[&str],
    config: ParallelConfig,
) -> Result<Vec<(String, Result<Vec<u8>>)>> {
    let archive = ParallelArchive::open(archive_path)?;

    // Configure thread pool if specified
    let pool = if let Some(threads) = config.num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|e| {
                Error::Io(std::io::Error::other(format!(
                    "Failed to create thread pool: {e}"
                )))
            })?
    } else {
        rayon::ThreadPoolBuilder::new().build().unwrap()
    };

    // Execute in the configured thread pool
    pool.install(|| {
        if config.skip_errors {
            // Return results with individual errors
            Ok(filenames
                .par_iter()
                .map(|&filename| {
                    let result = archive.read_file_with_new_handle(filename);
                    (filename.to_string(), result)
                })
                .collect())
        } else {
            // Fail on first error
            let results: Result<Vec<_>> = filenames
                .par_iter()
                .map(|&filename| {
                    let data = archive.read_file_with_new_handle(filename)?;
                    Ok((filename.to_string(), Ok(data)))
                })
                .collect();
            results
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArchiveBuilder;
    use tempfile::TempDir;

    fn create_test_archive() -> (TempDir, PathBuf) {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.mpq");

        let mut builder = ArchiveBuilder::new();

        // Add multiple files for parallel testing
        for i in 0..20 {
            let content = format!("File {i} content with some data to make it larger").repeat(100);
            builder = builder.add_file_data(content.into_bytes(), &format!("file_{i:02}.txt"));
        }

        builder.build(&path).unwrap();
        (temp, path)
    }

    #[test]
    fn test_parallel_extraction() {
        let (_temp, archive_path) = create_test_archive();
        let archive = ParallelArchive::open(&archive_path).unwrap();

        let files = vec!["file_00.txt", "file_05.txt", "file_10.txt", "file_15.txt"];
        let results = archive.extract_files_parallel(&files).unwrap();

        assert_eq!(results.len(), 4);
        for (filename, data) in results {
            assert!(!data.is_empty());
            assert!(files.contains(&filename.as_str()));
        }
    }

    #[test]
    fn test_extract_matching() {
        let (_temp, archive_path) = create_test_archive();
        let archive = ParallelArchive::open(&archive_path).unwrap();

        // Extract files ending with 5
        let results = archive
            .extract_matching_parallel(|name| name.ends_with("5.txt"))
            .unwrap();

        assert_eq!(results.len(), 2); // file_05.txt and file_15.txt
    }

    #[test]
    fn test_batched_extraction() {
        let (_temp, archive_path) = create_test_archive();
        let archive = ParallelArchive::open(&archive_path).unwrap();

        let files: Vec<&str> = (0..10)
            .map(|i| Box::leak(format!("file_{i:02}.txt").into_boxed_str()) as &str)
            .collect();

        let results = archive.extract_files_batched(&files, 3).unwrap();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_custom_processing() {
        let (_temp, archive_path) = create_test_archive();
        let archive = ParallelArchive::open(&archive_path).unwrap();

        let files = vec!["file_00.txt", "file_01.txt"];

        // Custom processor that returns file size
        let sizes = archive
            .process_files_parallel(&files, |_name, data| Ok(data.len()))
            .unwrap();

        assert_eq!(sizes.len(), 2);
        for size in sizes {
            assert!(size > 0);
        }
    }

    #[test]
    fn test_with_config() {
        let (_temp, archive_path) = create_test_archive();

        let config = ParallelConfig::new()
            .threads(2)
            .batch_size(5)
            .skip_errors(true);

        let files = vec!["file_00.txt", "nonexistent.txt", "file_01.txt"];
        let results = extract_with_config(&archive_path, &files, config).unwrap();

        assert_eq!(results.len(), 3);
        assert!(results[0].1.is_ok());
        assert!(results[1].1.is_err());
        assert!(results[2].1.is_ok());
    }
}
