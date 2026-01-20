//! MPQ Archive Comparison Functionality
//!
//! This module provides functionality to compare two MPQ archives, highlighting
//! differences in metadata, file lists, and file contents.

use crate::{Archive, FormatVersion, Result};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Options for archive comparison
#[derive(Debug, Clone)]
pub struct CompareOptions {
    /// Show detailed file-by-file comparison
    pub detailed: bool,
    /// Compare actual file contents (slower but thorough)
    pub content_check: bool,
    /// Only compare archive metadata
    pub metadata_only: bool,
    /// Ignore file order differences
    pub ignore_order: bool,
    /// Filter files by pattern
    pub filter: Option<String>,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            detailed: false,
            content_check: false,
            metadata_only: false,
            ignore_order: true,
            filter: None,
        }
    }
}

/// Result of comparing two archives
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    /// Whether the archives are identical
    pub identical: bool,
    /// Archive metadata comparison
    pub metadata: MetadataComparison,
    /// File list comparison (None if metadata_only)
    pub files: Option<FileComparison>,
    /// Summary of differences
    pub summary: ComparisonSummary,
}

/// Archive metadata comparison
#[derive(Debug, Clone)]
pub struct MetadataComparison {
    /// Format version comparison
    pub format_version: (FormatVersion, FormatVersion),
    /// Block size comparison
    pub block_size: (u16, u16),
    /// File count comparison
    pub file_count: (usize, usize),
    /// Archive size comparison
    pub archive_size: (u64, u64),
    /// Whether metadata matches
    pub matches: bool,
}

/// File list comparison
#[derive(Debug, Clone)]
pub struct FileComparison {
    /// Files only in source archive
    pub source_only: Vec<String>,
    /// Files only in target archive
    pub target_only: Vec<String>,
    /// Files present in both archives
    pub common_files: Vec<String>,
    /// Files with different sizes
    pub size_differences: Vec<FileSizeDiff>,
    /// Files with different content (if content_check enabled)
    pub content_differences: Vec<String>,
    /// Files with different metadata
    pub metadata_differences: Vec<FileMetadataDiff>,
}

/// File size difference
#[derive(Debug, Clone)]
pub struct FileSizeDiff {
    /// File name
    pub name: String,
    /// Size in source archive
    pub source_size: u64,
    /// Size in target archive
    pub target_size: u64,
    /// Compressed size in source archive
    pub source_compressed: u64,
    /// Compressed size in target archive
    pub target_compressed: u64,
}

/// File metadata difference
#[derive(Debug, Clone)]
pub struct FileMetadataDiff {
    /// File name
    pub name: String,
    /// Difference description
    pub difference: String,
    /// Source value
    pub source_value: String,
    /// Target value
    pub target_value: String,
}

/// Summary of comparison results
#[derive(Debug, Clone)]
pub struct ComparisonSummary {
    /// Total files in source
    pub source_files: usize,
    /// Total files in target
    pub target_files: usize,
    /// Files only in source
    pub source_only_count: usize,
    /// Files only in target
    pub target_only_count: usize,
    /// Files with differences
    pub different_files: usize,
    /// Files that are identical
    pub identical_files: usize,
}

/// Compare two MPQ archives
pub fn compare_archives<P: AsRef<Path>>(
    source_path: P,
    target_path: P,
    detailed: bool,
    content_check: bool,
    metadata_only: bool,
    _ignore_order: bool,
    filter: Option<String>,
) -> Result<ComparisonResult> {
    let source_path = source_path.as_ref();
    let target_path = target_path.as_ref();

    log::info!(
        "Comparing archives: {} vs {}",
        source_path.display(),
        target_path.display()
    );

    // Open both archives
    let mut source_archive = Archive::open(source_path)?;
    let mut target_archive = Archive::open(target_path)?;

    // Compare metadata
    let metadata = compare_metadata(&mut source_archive, &mut target_archive)?;

    // Early return if only comparing metadata
    if metadata_only {
        return Ok(ComparisonResult {
            identical: metadata.matches,
            metadata: metadata.clone(),
            files: None,
            summary: ComparisonSummary {
                source_files: metadata.file_count.0,
                target_files: metadata.file_count.1,
                source_only_count: 0,
                target_only_count: 0,
                different_files: 0,
                identical_files: 0,
            },
        });
    }

    // Compare files
    let files = compare_files(
        &mut source_archive,
        &mut target_archive,
        detailed,
        content_check,
        filter,
    )?;

    // Generate summary
    let summary = ComparisonSummary {
        source_files: metadata.file_count.0,
        target_files: metadata.file_count.1,
        source_only_count: files.source_only.len(),
        target_only_count: files.target_only.len(),
        different_files: files.size_differences.len()
            + files.content_differences.len()
            + files.metadata_differences.len(),
        identical_files: files.common_files.len()
            - files.size_differences.len()
            - files.content_differences.len()
            - files.metadata_differences.len(),
    };

    // Determine if archives are identical
    let files_identical = files.source_only.is_empty()
        && files.target_only.is_empty()
        && files.size_differences.is_empty()
        && files.content_differences.is_empty()
        && files.metadata_differences.is_empty();

    let identical = metadata.matches && files_identical;

    Ok(ComparisonResult {
        identical,
        metadata,
        files: Some(files),
        summary,
    })
}

/// Compare archive metadata
fn compare_metadata(source: &mut Archive, target: &mut Archive) -> Result<MetadataComparison> {
    // Get info first (requires mutable borrow)
    let source_info = source.get_info()?;
    let target_info = target.get_info()?;

    // Then get headers (immutable borrow after mutable borrow is dropped)
    let source_header = source.header();
    let target_header = target.header();

    let format_version = (source_header.format_version, target_header.format_version);
    let block_size = (source_header.block_size, target_header.block_size);
    let file_count = (source_info.file_count, target_info.file_count);
    let archive_size = (source_info.file_size, target_info.file_size);

    let matches = format_version.0 == format_version.1
        && block_size.0 == block_size.1
        && file_count.0 == file_count.1;

    Ok(MetadataComparison {
        format_version,
        block_size,
        file_count,
        archive_size,
        matches,
    })
}

/// Compare files between archives
fn compare_files(
    source: &mut Archive,
    target: &mut Archive,
    detailed: bool,
    content_check: bool,
    filter: Option<String>,
) -> Result<FileComparison> {
    // Get file lists
    let source_files = get_file_list(source, &filter)?;
    let target_files = get_file_list(target, &filter)?;

    // Convert to HashSets for set operations
    let source_set: HashSet<_> = source_files.keys().collect();
    let target_set: HashSet<_> = target_files.keys().collect();

    // Find differences
    let source_only: Vec<String> = source_set
        .difference(&target_set)
        .map(|s| s.to_string())
        .collect();

    let target_only: Vec<String> = target_set
        .difference(&source_set)
        .map(|s| s.to_string())
        .collect();

    let common_files: Vec<String> = source_set
        .intersection(&target_set)
        .map(|s| s.to_string())
        .collect();

    // Compare common files
    let mut size_differences = Vec::new();
    let mut content_differences = Vec::new();
    let mut metadata_differences = Vec::new();

    for filename in &common_files {
        let source_entry = &source_files[filename];
        let target_entry = &target_files[filename];

        // Check size differences
        if source_entry.size != target_entry.size
            || source_entry.compressed_size != target_entry.compressed_size
        {
            size_differences.push(FileSizeDiff {
                name: filename.clone(),
                source_size: source_entry.size,
                target_size: target_entry.size,
                source_compressed: source_entry.compressed_size,
                target_compressed: target_entry.compressed_size,
            });
        }

        // Check metadata differences (if detailed)
        if detailed && source_entry.flags != target_entry.flags {
            metadata_differences.push(FileMetadataDiff {
                name: filename.clone(),
                difference: "Flags".to_string(),
                source_value: format!("0x{:08x}", source_entry.flags),
                target_value: format!("0x{:08x}", target_entry.flags),
            });
        }

        // Check content differences (if content_check enabled)
        if content_check {
            match (source.read_file(filename), target.read_file(filename)) {
                (Ok(source_data), Ok(target_data)) => {
                    if source_data != target_data {
                        content_differences.push(filename.clone());
                    }
                }
                _ => {
                    // If we can't read either file, consider it a content difference
                    content_differences.push(filename.clone());
                }
            }
        }
    }

    Ok(FileComparison {
        source_only,
        target_only,
        common_files,
        size_differences,
        content_differences,
        metadata_differences,
    })
}

/// Get file list from archive with optional filtering
fn get_file_list(
    archive: &mut Archive,
    filter: &Option<String>,
) -> Result<HashMap<String, crate::FileEntry>> {
    let files = archive
        .list()
        .unwrap_or_else(|_| archive.list_all().unwrap_or_default());

    let mut file_map = HashMap::new();

    for file in files {
        // Apply filter if provided
        if let Some(pattern) = filter
            && !simple_pattern_match(&file.name, pattern)
        {
            continue;
        }

        file_map.insert(file.name.clone(), file);
    }

    Ok(file_map)
}

/// Simple pattern matching function (supports * wildcard)
fn simple_pattern_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') {
        // Convert glob pattern to regex-like matching
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.is_empty() {
            return true;
        }

        let mut text_pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            if i == 0 {
                // First part must match from the beginning
                if !text[text_pos..].starts_with(part) {
                    return false;
                }
                text_pos += part.len();
            } else if i == parts.len() - 1 {
                // Last part must match at the end
                return text[text_pos..].ends_with(part);
            } else {
                // Middle parts must be found somewhere
                if let Some(pos) = text[text_pos..].find(part) {
                    text_pos += pos + part.len();
                } else {
                    return false;
                }
            }
        }
        true
    } else {
        // Exact match if no wildcards
        text == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_options_default() {
        let options = CompareOptions::default();
        assert!(!options.detailed);
        assert!(!options.content_check);
        assert!(!options.metadata_only);
        assert!(options.ignore_order);
        assert!(options.filter.is_none());
    }

    #[test]
    fn test_comparison_summary_new() {
        let summary = ComparisonSummary {
            source_files: 100,
            target_files: 95,
            source_only_count: 5,
            target_only_count: 2,
            different_files: 3,
            identical_files: 90,
        };

        assert_eq!(summary.source_files, 100);
        assert_eq!(summary.target_files, 95);
        assert_eq!(summary.source_only_count, 5);
        assert_eq!(summary.target_only_count, 2);
        assert_eq!(summary.different_files, 3);
        assert_eq!(summary.identical_files, 90);
    }

    #[test]
    fn test_simple_pattern_match() {
        // Exact matches
        assert!(simple_pattern_match("test.txt", "test.txt"));
        assert!(!simple_pattern_match("test.txt", "other.txt"));

        // Wildcard matches
        assert!(simple_pattern_match("test.txt", "*"));
        assert!(simple_pattern_match("test.txt", "*.txt"));
        assert!(simple_pattern_match("test.txt", "test.*"));
        assert!(simple_pattern_match("test.txt", "*test*"));

        // Complex patterns
        assert!(simple_pattern_match("folder/test.txt", "*/test.txt"));
        assert!(simple_pattern_match("folder/test.txt", "folder/*.txt"));
        assert!(!simple_pattern_match("folder/test.txt", "*.dbc"));
        assert!(!simple_pattern_match("folder/test.txt", "other/*"));
    }
}
