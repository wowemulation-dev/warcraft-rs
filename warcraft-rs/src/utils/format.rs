//! Formatting utilities

use chrono::{Local, TimeZone};
use humansize::{DECIMAL, format_size};

/// Format file size in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    format_size(bytes, DECIMAL)
}

/// Format a timestamp
#[allow(dead_code)] // Will be used when other file formats are implemented
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

/// Format a percentage
pub fn format_percentage(value: f64) -> String {
    format!("{value:.1}%")
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
    fn test_format_compression_ratio() {
        assert_eq!(format_compression_ratio(1000, 500), "50.0%");
        assert_eq!(format_compression_ratio(1000, 250), "75.0%");
        assert_eq!(format_compression_ratio(1000, 1000), "0.0%");
        assert_eq!(format_compression_ratio(0, 0), "N/A");
    }
}
