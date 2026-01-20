//! I/O and path utilities

use std::path::Path;

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
                format!(".../{filename}")
            } else {
                format!("{result}/{filename}")
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

        if let Some(last) = parts.last()
            && !last.is_empty()
            && !text_lower.ends_with(last)
        {
            return false;
        }

        true
    } else {
        text_lower.contains(&pattern_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "Result '{result}' should contain '...'"
        );

        // Result should contain the filename (unless the filename itself is too long)
        assert!(
            result.contains("file.txt") || result.ends_with("...txt"),
            "Result '{result}' should contain filename"
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
}
