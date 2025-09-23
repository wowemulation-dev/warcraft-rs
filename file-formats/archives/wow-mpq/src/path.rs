//! Path utilities for MPQ archives
//!
//! MPQ archives use backslashes (`\`) as path separators internally, regardless
//! of the host operating system. This module provides utilities to normalize
//! paths for storage and convert them for extraction.
//!
//! # Path Separator Handling
//!
//! When working with MPQ archives:
//! - **Adding files**: Paths are automatically normalized to use backslashes
//! - **Reading files**: Both forward slashes and backslashes are accepted
//! - **Listing files**: Paths are displayed using the system's native separator
//! - **Extracting files**: Output paths use the system's native separator
//!
//! # Examples
//!
//! ```no_run
//! use wow_mpq::ArchiveBuilder;
//!
//! // These all refer to the same file internally
//! let builder = ArchiveBuilder::new()
//!     .add_file("data.txt", "dir/subdir/file.txt")      // Forward slashes
//!     .add_file("data.txt", "dir\\subdir\\file.txt")    // Backslashes
//!     .add_file("data.txt", "dir/subdir\\file.txt");    // Mixed
//!
//! // When reading, both separators work
//! # use wow_mpq::Archive;
//! # let archive = Archive::open("test.mpq").unwrap();
//! let data1 = archive.read_file("dir/subdir/file.txt").unwrap();
//! let data2 = archive.read_file("dir\\subdir\\file.txt").unwrap();
//! assert_eq!(data1, data2);
//! ```

/// Normalize a path for storage in an MPQ archive
///
/// Converts forward slashes to backslashes to match MPQ format requirements.
///
/// # Examples
///
/// ```
/// use wow_mpq::path::normalize_mpq_path;
///
/// assert_eq!(normalize_mpq_path("dir/file.txt"), "dir\\file.txt");
/// assert_eq!(normalize_mpq_path("dir\\file.txt"), "dir\\file.txt");
/// assert_eq!(normalize_mpq_path("a/b/c/d.txt"), "a\\b\\c\\d.txt");
/// ```
pub fn normalize_mpq_path(path: &str) -> String {
    path.replace('/', "\\")
}

/// Convert an MPQ path to a system path
///
/// On Windows, this is a no-op since Windows uses backslashes.
/// On Unix-like systems, this converts backslashes to forward slashes.
///
/// # Examples
///
/// ```
/// use wow_mpq::path::mpq_path_to_system;
///
/// #[cfg(unix)]
/// assert_eq!(mpq_path_to_system("dir\\file.txt"), "dir/file.txt");
///
/// #[cfg(windows)]
/// assert_eq!(mpq_path_to_system("dir\\file.txt"), "dir\\file.txt");
/// ```
pub fn mpq_path_to_system(path: &str) -> String {
    #[cfg(unix)]
    {
        path.replace('\\', "/")
    }

    #[cfg(windows)]
    {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_mpq_path() {
        // Forward slashes should be converted
        assert_eq!(normalize_mpq_path("path/to/file.txt"), "path\\to\\file.txt");

        // Backslashes should remain unchanged
        assert_eq!(
            normalize_mpq_path("path\\to\\file.txt"),
            "path\\to\\file.txt"
        );

        // Mixed separators should be normalized
        assert_eq!(
            normalize_mpq_path("path/to\\file.txt"),
            "path\\to\\file.txt"
        );

        // Empty and simple paths
        assert_eq!(normalize_mpq_path(""), "");
        assert_eq!(normalize_mpq_path("file.txt"), "file.txt");

        // Multiple consecutive slashes
        assert_eq!(
            normalize_mpq_path("path//to///file.txt"),
            "path\\\\to\\\\\\file.txt"
        );
    }

    #[test]
    fn test_mpq_path_to_system() {
        let mpq_path = "dir\\subdir\\file.txt";

        #[cfg(unix)]
        {
            assert_eq!(mpq_path_to_system(mpq_path), "dir/subdir/file.txt");
            assert_eq!(mpq_path_to_system(""), "");
            assert_eq!(mpq_path_to_system("file.txt"), "file.txt");
        }

        #[cfg(windows)]
        {
            assert_eq!(mpq_path_to_system(mpq_path), mpq_path);
            assert_eq!(mpq_path_to_system(""), "");
            assert_eq!(mpq_path_to_system("file.txt"), "file.txt");
        }
    }
}
