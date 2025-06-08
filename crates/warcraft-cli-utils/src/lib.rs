//! Shared utilities for warcraft-rs command-line tools

use chrono::{Local, TimeZone};
use humansize::{DECIMAL, format_size};
use indicatif::{ProgressBar, ProgressStyle};
use prettytable::{Cell, Row, Table};
use std::path::Path;

/// Format file size in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    format_size(bytes, DECIMAL)
}

/// Create a standard progress bar
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Create a spinner for indeterminate progress
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// Format a timestamp
pub fn format_timestamp(timestamp: u64) -> String {
    if timestamp == 0 {
        "N/A".to_string()
    } else {
        // Convert timestamp to DateTime
        match Local.timestamp_opt(timestamp as i64, 0) {
            chrono::LocalResult::Single(datetime) => {
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            }
            _ => "Invalid timestamp".to_string(),
        }
    }
}

/// Truncate a path for display
pub fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        let filename = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if filename.len() >= max_len {
            // Filename alone is too long, truncate it
            format!("...{}", &filename[filename.len() - max_len + 3..])
        } else {
            // Try to include some path with the filename
            let parts: Vec<&str> = path.split('/').collect();
            let mut result = String::new();
            let mut has_ellipsis = false;

            // Calculate how much space we have for the path
            let space_for_path = max_len.saturating_sub(filename.len() + 1); // -1 for final '/'

            // Try to fit path components from the beginning
            for part in parts.iter().take(parts.len() - 1) {
                let needed = if result.is_empty() {
                    part.len()
                } else {
                    part.len() + 1 // +1 for '/'
                };

                if !has_ellipsis && result.len() + needed > space_for_path {
                    // Need to add ellipsis
                    if space_for_path >= 3 {
                        result = "...".to_string();
                        has_ellipsis = true;
                    } else {
                        break;
                    }
                } else if result.len() + needed <= space_for_path {
                    if !result.is_empty() && result != "..." {
                        result.push('/');
                    }
                    result.push_str(part);
                }
            }

            if result.is_empty() || result == "..." {
                format!(".../{}", filename)
            } else {
                format!("{}/{}", result, filename)
            }
        }
    }
}

/// Simple wildcard pattern matching
pub fn matches_pattern(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return true;
    }

    if pattern == "*" {
        return true;
    }

    let pattern_lower = pattern.to_lowercase();
    let text_lower = text.to_lowercase();

    if pattern_lower.contains('*') {
        let parts: Vec<&str> = pattern_lower.split('*').collect();
        if parts.is_empty() {
            return true;
        }

        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }

            if i == 0 && !text_lower.starts_with(part) {
                return false;
            }

            if let Some(found) = text_lower[pos..].find(part) {
                pos += found + part.len();
            } else {
                return false;
            }
        }

        if let Some(last) = parts.last() {
            if !last.is_empty() && !text_lower.ends_with(last) {
                return false;
            }
        }

        true
    } else {
        text_lower.contains(&pattern_lower)
    }
}

/// Create a table with headers
pub fn create_table(headers: Vec<&str>) -> Table {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

    let header_cells: Vec<Cell> = headers
        .into_iter()
        .map(|h| Cell::new(h).style_spec("b"))
        .collect();
    table.set_titles(Row::new(header_cells));

    table
}

/// Add a row to a table
pub fn add_table_row(table: &mut Table, cells: Vec<String>) {
    let row_cells: Vec<Cell> = cells.into_iter().map(|s| Cell::new(&s)).collect();
    table.add_row(Row::new(row_cells));
}

/// Format a percentage
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Format a compression ratio
pub fn format_compression_ratio(original: u64, compressed: u64) -> String {
    if original == 0 {
        "N/A".to_string()
    } else {
        let ratio = 100.0 - (compressed as f64 / original as f64 * 100.0);
        format_percentage(ratio)
    }
}

/// Common error messages
pub mod errors {
    pub const FILE_NOT_FOUND: &str = "File not found";
    pub const INVALID_FORMAT: &str = "Invalid file format";
    pub const PERMISSION_DENIED: &str = "Permission denied";
    pub const OPERATION_FAILED: &str = "Operation failed";
}

/// Common success messages
pub mod messages {
    pub const OPERATION_COMPLETE: &str = "Operation completed successfully";
    pub const FILES_EXTRACTED: &str = "Files extracted successfully";
    pub const ARCHIVE_CREATED: &str = "Archive created successfully";
    pub const VERIFICATION_PASSED: &str = "Verification passed";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.02 kB");
        assert_eq!(format_bytes(1048576), "1.05 MB");
        assert_eq!(format_bytes(1073741824), "1.07 GB");
    }

    #[test]
    fn test_truncate_path() {
        // Short paths should not be truncated
        assert_eq!(truncate_path("short.txt", 20), "short.txt");

        // Long paths should be truncated
        let result = truncate_path("very/long/path/to/file.txt", 15);

        // Result should be at most max_len characters
        assert!(
            result.len() <= 15,
            "Result '{}' is {} chars, expected <= 15",
            result,
            result.len()
        );

        // Result should contain ellipsis to indicate truncation
        assert!(
            result.contains("..."),
            "Result '{}' should contain '...'",
            result
        );

        // Result should contain the filename (unless the filename itself is too long)
        assert!(
            result.contains("file.txt") || result.ends_with("...txt"),
            "Result '{}' should contain filename",
            result
        );

        // Test edge cases
        assert_eq!(truncate_path("a", 5), "a");
        assert_eq!(truncate_path("abcdef", 5), "...ef");
        assert_eq!(truncate_path("dir/file.txt", 50), "dir/file.txt");
    }

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("test.txt", "*"));
        assert!(matches_pattern("test.txt", "*.txt"));
        assert!(matches_pattern("test.txt", "test.*"));
        assert!(matches_pattern("test.txt", "*test*"));
        assert!(!matches_pattern("test.txt", "*.exe"));
        assert!(!matches_pattern("test.txt", "other.*"));
        assert!(matches_pattern("TEST.TXT", "*.txt")); // Case insensitive
    }

    #[test]
    fn test_format_compression_ratio() {
        assert_eq!(format_compression_ratio(1000, 500), "50.0%");
        assert_eq!(format_compression_ratio(1000, 250), "75.0%");
        assert_eq!(format_compression_ratio(1000, 1000), "0.0%");
        assert_eq!(format_compression_ratio(0, 0), "N/A");
    }
}
