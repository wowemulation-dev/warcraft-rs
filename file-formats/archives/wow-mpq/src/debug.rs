//! Debug utilities for inspecting MPQ archive internals.
//!
//! This module provides various debugging tools for analyzing MPQ archives,
//! including hex dumps, table formatters, and structure visualizers.

use crate::tables::{BlockEntry, HashEntry};

/// Hex dump configuration options
#[derive(Debug, Clone)]
pub struct HexDumpConfig {
    /// Number of bytes per line
    pub bytes_per_line: usize,
    /// Show ASCII representation
    pub show_ascii: bool,
    /// Show byte offsets
    pub show_offset: bool,
    /// Maximum bytes to dump (0 = unlimited)
    pub max_bytes: usize,
}

impl Default for HexDumpConfig {
    fn default() -> Self {
        Self {
            bytes_per_line: 16,
            show_ascii: true,
            show_offset: true,
            max_bytes: 512,
        }
    }
}

/// Generate a hex dump of binary data
pub fn hex_dump(data: &[u8], config: &HexDumpConfig) -> String {
    let mut output = String::new();
    let mut offset = 0;

    let max_bytes = if config.max_bytes == 0 {
        data.len()
    } else {
        data.len().min(config.max_bytes)
    };

    while offset < max_bytes {
        let chunk_end = (offset + config.bytes_per_line).min(max_bytes);
        let chunk = &data[offset..chunk_end];

        // Offset
        if config.show_offset {
            output.push_str(&format!("{:08X}  ", offset));
        }

        // Hex bytes
        for (i, byte) in chunk.iter().enumerate() {
            output.push_str(&format!("{:02X} ", byte));
            if i == 7 && config.bytes_per_line > 8 {
                output.push(' '); // Extra space in the middle
            }
        }

        // Padding for incomplete lines
        if chunk.len() < config.bytes_per_line {
            let padding = config.bytes_per_line - chunk.len();
            for _ in 0..padding {
                output.push_str("   ");
            }
            if config.bytes_per_line > 8 && chunk.len() <= 8 {
                output.push(' ');
            }
        }

        // ASCII representation
        if config.show_ascii {
            output.push_str(" |");
            for byte in chunk {
                let ch = if *byte >= 0x20 && *byte < 0x7F {
                    *byte as char
                } else {
                    '.'
                };
                output.push(ch);
            }
            output.push('|');
        }

        output.push('\n');
        offset += config.bytes_per_line;
    }

    if max_bytes < data.len() {
        output.push_str(&format!("... ({} more bytes)\n", data.len() - max_bytes));
    }

    output
}

/// Generate a hex dump with custom configuration
pub fn hex_dump_custom(data: &[u8], bytes_per_line: usize, max_bytes: usize) -> String {
    let config = HexDumpConfig {
        bytes_per_line,
        max_bytes,
        ..Default::default()
    };
    hex_dump(data, &config)
}

/// Format a single hex line for inline display
pub fn hex_string(data: &[u8], max_len: usize) -> String {
    let len = data.len().min(max_len);
    let hex: Vec<String> = data[..len].iter().map(|b| format!("{:02X}", b)).collect();
    if data.len() > max_len {
        format!("{} ... ({} bytes total)", hex.join(" "), data.len())
    } else {
        hex.join(" ")
    }
}

/// Table formatter for displaying structured data
#[derive(Debug)]
pub struct TableFormatter {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
}

impl TableFormatter {
    /// Create a new table formatter with headers
    pub fn new(headers: Vec<&str>) -> Self {
        let headers: Vec<String> = headers.into_iter().map(String::from).collect();
        let column_widths = headers.iter().map(|h| h.len()).collect();

        Self {
            headers,
            rows: Vec::new(),
            column_widths,
        }
    }

    /// Add a row to the table
    pub fn add_row(&mut self, row: Vec<String>) {
        // Update column widths
        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                self.column_widths[i] = self.column_widths[i].max(cell.len());
            }
        }
        self.rows.push(row);
    }

    /// Format the table as a string
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Header
        self.write_separator(&mut output);
        self.write_row(&mut output, &self.headers);
        self.write_separator(&mut output);

        // Rows
        for row in &self.rows {
            self.write_row(&mut output, row);
        }

        if !self.rows.is_empty() {
            self.write_separator(&mut output);
        }

        output
    }

    fn write_separator(&self, output: &mut String) {
        output.push('+');
        for width in &self.column_widths {
            output.push('-');
            for _ in 0..*width {
                output.push('-');
            }
            output.push('-');
            output.push('+');
        }
        output.push('\n');
    }

    fn write_row(&self, output: &mut String, row: &[String]) {
        output.push('|');
        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                output.push(' ');
                output.push_str(cell);
                let padding = self.column_widths[i] - cell.len();
                for _ in 0..padding {
                    output.push(' ');
                }
                output.push(' ');
            }
            output.push('|');
        }
        output.push('\n');
    }
}

/// Progress indicator for long operations
#[derive(Debug)]
pub struct ProgressTracker {
    name: String,
    total: usize,
    current: usize,
    start_time: std::time::Instant,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(name: &str, total: usize) -> Self {
        Self {
            name: name.to_string(),
            total,
            current: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Update the current progress
    pub fn update(&mut self, current: usize) {
        self.current = current;
    }

    /// Increment the current progress by 1
    pub fn increment(&mut self) {
        self.current += 1;
    }

    /// Mark the operation as finished and log the total time
    pub fn finish(&self) {
        let elapsed = self.start_time.elapsed();
        log::debug!(
            "{} completed: {} items in {:.2}s",
            self.name,
            self.total,
            elapsed.as_secs_f64()
        );
    }

    /// Log the current progress percentage
    pub fn log_progress(&self) {
        if self.total > 0 {
            let percent = (self.current as f64 / self.total as f64) * 100.0;
            log::trace!(
                "{}: {}/{} ({:.1}%)",
                self.name,
                self.current,
                self.total,
                percent
            );
        }
    }
}

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format a bitflag value showing individual flags
pub fn format_flags(value: u32, flag_names: &[(u32, &str)]) -> String {
    let mut flags = Vec::new();

    for (flag, name) in flag_names {
        if value & flag != 0 {
            flags.push(*name);
        }
    }

    if flags.is_empty() {
        format!("0x{:08X} (none)", value)
    } else {
        format!("0x{:08X} ({})", value, flags.join(" | "))
    }
}

/// Debug context for tracking operation flow
#[derive(Debug)]
pub struct DebugContext {
    indent: usize,
    start_time: std::time::Instant,
}

impl Default for DebugContext {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugContext {
    /// Create a new debug context
    pub fn new() -> Self {
        Self {
            indent: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Enter a new scope, increasing indentation
    pub fn enter_scope(&mut self, name: &str) {
        let indent = "  ".repeat(self.indent);
        log::trace!("{}→ {}", indent, name);
        self.indent += 1;
    }

    /// Exit the current scope, decreasing indentation
    pub fn exit_scope(&mut self, name: &str) {
        self.indent = self.indent.saturating_sub(1);
        let indent = "  ".repeat(self.indent);
        log::trace!("{}← {} ({}ms)", indent, name, self.elapsed_ms());
    }

    /// Log a message at the current indentation level
    pub fn log(&self, message: &str) {
        let indent = "  ".repeat(self.indent);
        log::trace!("{}  {}", indent, message);
    }

    fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_dump() {
        let data = b"Hello, World!\x00\x01\x02\x03";
        let dump = hex_dump(data, &HexDumpConfig::default());
        assert!(dump.contains("48 65 6C 6C 6F 2C 20 57"));
        assert!(dump.contains("|Hello, World!"));
    }

    #[test]
    fn test_table_formatter() {
        let mut table = TableFormatter::new(vec!["ID", "Name", "Size"]);
        table.add_row(vec![
            "1".to_string(),
            "test.txt".to_string(),
            "1024".to_string(),
        ]);
        table.add_row(vec![
            "2".to_string(),
            "data.bin".to_string(),
            "2048".to_string(),
        ]);

        let output = table.format();
        assert!(output.contains("test.txt"));
        assert!(output.contains("1024"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
    }
}

/// Format a hash table for display
pub fn format_hash_table(entries: &[HashEntry]) -> String {
    let mut table = TableFormatter::new(vec![
        "Index",
        "Name1",
        "Name2",
        "Locale",
        "Platform",
        "Block Index",
        "Status",
    ]);

    for (index, entry) in entries.iter().enumerate() {
        let status = if entry.is_empty() {
            "Empty"
        } else if entry.is_deleted() {
            "Deleted"
        } else {
            "Active"
        };

        table.add_row(vec![
            format!("{}", index),
            format!("0x{:08X}", entry.name_1),
            format!("0x{:08X}", entry.name_2),
            format!("0x{:04X}", entry.locale),
            format!("{}", entry.platform),
            if entry.block_index == HashEntry::EMPTY_NEVER_USED {
                "FFFFFFFF".to_string()
            } else if entry.block_index == HashEntry::EMPTY_DELETED {
                "FFFFFFFE".to_string()
            } else {
                format!("{}", entry.block_index)
            },
            status.to_string(),
        ]);
    }

    table.format()
}

/// Format a block table for display
pub fn format_block_table(entries: &[BlockEntry]) -> String {
    let mut table = TableFormatter::new(vec![
        "Index",
        "File Pos",
        "Comp Size",
        "File Size",
        "Flags",
        "Compression",
    ]);

    for (index, entry) in entries.iter().enumerate() {
        let compression = if entry.is_compressed() {
            if entry.is_imploded() {
                "PKWARE"
            } else {
                "Multi"
            }
        } else {
            "None"
        };

        table.add_row(vec![
            format!("{}", index),
            format!("0x{:08X}", entry.file_pos),
            format_size(entry.compressed_size as u64),
            format_size(entry.file_size as u64),
            format_flags(
                entry.flags,
                &[
                    (BlockEntry::FLAG_IMPLODE, "IMPLODE"),
                    (BlockEntry::FLAG_COMPRESS, "COMPRESS"),
                    (BlockEntry::FLAG_ENCRYPTED, "ENCRYPTED"),
                    (BlockEntry::FLAG_FIX_KEY, "FIX_KEY"),
                    (BlockEntry::FLAG_PATCH_FILE, "PATCH"),
                    (BlockEntry::FLAG_SINGLE_UNIT, "SINGLE"),
                    (BlockEntry::FLAG_DELETE_MARKER, "DELETE"),
                    (BlockEntry::FLAG_SECTOR_CRC, "CRC"),
                    (BlockEntry::FLAG_EXISTS, "EXISTS"),
                ],
            ),
            compression.to_string(),
        ]);
    }

    table.format()
}

/// Debug dump for a single hash entry
pub fn dump_hash_entry(entry: &HashEntry, index: usize) -> String {
    format!(
        "HashEntry[{}]:\n  Name1: 0x{:08X}\n  Name2: 0x{:08X}\n  Locale: 0x{:04X}\n  Platform: {}\n  Block Index: {}\n  Status: {}",
        index,
        entry.name_1,
        entry.name_2,
        entry.locale,
        entry.platform,
        if entry.block_index == HashEntry::EMPTY_NEVER_USED {
            "FFFFFFFF (Never Used)".to_string()
        } else if entry.block_index == HashEntry::EMPTY_DELETED {
            "FFFFFFFE (Deleted)".to_string()
        } else {
            format!("{}", entry.block_index)
        },
        if entry.is_empty() {
            "Empty"
        } else if entry.is_deleted() {
            "Deleted"
        } else {
            "Active"
        }
    )
}

/// Debug dump for a single block entry
pub fn dump_block_entry(entry: &BlockEntry, index: usize) -> String {
    let mut flags_str = Vec::new();

    if entry.flags & BlockEntry::FLAG_IMPLODE != 0 {
        flags_str.push("IMPLODE");
    }
    if entry.flags & BlockEntry::FLAG_COMPRESS != 0 {
        flags_str.push("COMPRESS");
    }
    if entry.flags & BlockEntry::FLAG_ENCRYPTED != 0 {
        flags_str.push("ENCRYPTED");
    }
    if entry.flags & BlockEntry::FLAG_FIX_KEY != 0 {
        flags_str.push("FIX_KEY");
    }
    if entry.flags & BlockEntry::FLAG_PATCH_FILE != 0 {
        flags_str.push("PATCH");
    }
    if entry.flags & BlockEntry::FLAG_SINGLE_UNIT != 0 {
        flags_str.push("SINGLE_UNIT");
    }
    if entry.flags & BlockEntry::FLAG_DELETE_MARKER != 0 {
        flags_str.push("DELETE_MARKER");
    }
    if entry.flags & BlockEntry::FLAG_SECTOR_CRC != 0 {
        flags_str.push("SECTOR_CRC");
    }
    if entry.flags & BlockEntry::FLAG_EXISTS != 0 {
        flags_str.push("EXISTS");
    }

    format!(
        "BlockEntry[{}]:\n  File Position: 0x{:08X}\n  Compressed Size: {} ({})\n  File Size: {} ({})\n  Flags: 0x{:08X} [{}]\n  Compression: {}",
        index,
        entry.file_pos,
        entry.compressed_size,
        format_size(entry.compressed_size as u64),
        entry.file_size,
        format_size(entry.file_size as u64),
        entry.flags,
        flags_str.join(", "),
        if entry.is_compressed() {
            if entry.is_imploded() {
                "PKWARE Implode"
            } else {
                "Multiple Methods"
            }
        } else {
            "None"
        }
    )
}

impl BlockEntry {
    /// Check if file uses PKWARE implode compression
    pub fn is_imploded(&self) -> bool {
        (self.flags & Self::FLAG_IMPLODE) != 0
    }
}

/// Archive structure visualization
#[derive(Debug)]
pub struct ArchiveStructureVisualizer {
    sections: Vec<(u64, u64, String, String)>, // (offset, size, name, description)
}

impl Default for ArchiveStructureVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveStructureVisualizer {
    /// Create a new archive structure visualizer
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a section to the visualization
    pub fn add_section(&mut self, offset: u64, size: u64, name: &str, description: &str) {
        self.sections
            .push((offset, size, name.to_string(), description.to_string()));
    }

    /// Generate the visual representation
    pub fn visualize(&mut self) -> String {
        // Sort sections by offset
        self.sections.sort_by_key(|s| s.0);

        let mut output = String::new();
        output.push_str("MPQ Archive Structure:\n");
        output.push_str("=====================\n\n");

        let max_name_len = self.sections.iter().map(|s| s.2.len()).max().unwrap_or(10);

        // Header
        output.push_str(&format!(
            "{:>10} | {:>10} | {:<width$} | Description\n",
            "Offset",
            "Size",
            "Section",
            width = max_name_len
        ));
        output.push_str(&format!(
            "{:-<10}-+-{:-<10}-+-{:-<width$}-+-{:-<40}\n",
            "",
            "",
            "",
            "",
            width = max_name_len
        ));

        // Sections
        for (offset, size, name, desc) in &self.sections {
            output.push_str(&format!(
                "0x{:08X} | {:>10} | {:<width$} | {}\n",
                offset,
                format_size(*size),
                name,
                desc,
                width = max_name_len
            ));
        }

        // Visual block diagram
        output.push_str("\nVisual Layout:\n");
        output.push_str("-------------\n");

        let mut current_offset = 0u64;
        for (offset, size, name, _) in &self.sections {
            // Add gap if there is one
            if *offset > current_offset {
                let gap = *offset - current_offset;
                output.push_str(&format!("│ {:^20} │ {} gap\n", "...", format_size(gap)));
            }

            output.push_str("├──────────────────────┤\n");
            output.push_str(&format!("│ {:^20} │ @ 0x{:08X}\n", name, offset));
            output.push_str(&format!("│ {:^20} │ {}\n", format_size(*size), ""));

            current_offset = offset + size;
        }
        output.push_str("└──────────────────────┘\n");

        output
    }
}

/// Create an archive structure visualization from archive info
pub fn visualize_archive_structure(info: &crate::ArchiveInfo) -> String {
    let mut viz = ArchiveStructureVisualizer::new();

    // User data (if present)
    if let Some(user_data) = &info.user_data_info {
        viz.add_section(
            0,
            user_data.header_size as u64 + user_data.data_size as u64,
            "User Data",
            "Custom user data section",
        );
    }

    // MPQ Header
    viz.add_section(info.archive_offset, 32, "MPQ Header", "Main archive header");

    // Hash Table
    if let Some(size) = info.hash_table_info.size {
        viz.add_section(
            info.hash_table_info.offset,
            info.hash_table_info
                .compressed_size
                .unwrap_or(size as u64 * 16),
            "Hash Table",
            &format!("{} entries", size),
        );
    }

    // Block Table
    if let Some(size) = info.block_table_info.size {
        viz.add_section(
            info.block_table_info.offset,
            info.block_table_info
                .compressed_size
                .unwrap_or(size as u64 * 16),
            "Block Table",
            &format!("{} entries", size),
        );
    }

    // HET Table (v3+)
    if let Some(het_info) = &info.het_table_info {
        if let Some(size) = het_info.size {
            viz.add_section(
                het_info.offset,
                het_info.compressed_size.unwrap_or(size as u64),
                "HET Table",
                "Extended hash table (v3+)",
            );
        }
    }

    // BET Table (v3+)
    if let Some(bet_info) = &info.bet_table_info {
        if let Some(size) = bet_info.size {
            viz.add_section(
                bet_info.offset,
                bet_info.compressed_size.unwrap_or(size as u64),
                "BET Table",
                "Extended block table (v3+)",
            );
        }
    }

    // Hi-block table (v2+)
    if let Some(hi_info) = &info.hi_block_table_info {
        if let Some(size) = hi_info.size {
            viz.add_section(
                hi_info.offset,
                hi_info.compressed_size.unwrap_or(size as u64 * 8),
                "Hi-Block Table",
                "High 32-bits of block offsets (v2+)",
            );
        }
    }

    viz.visualize()
}

/// File extraction tracer for detailed debugging
#[derive(Debug)]
pub struct FileExtractionTracer {
    file_name: String,
    steps: Vec<(String, Option<String>)>, // (step description, optional details)
    start_time: std::time::Instant,
}

impl FileExtractionTracer {
    /// Create a new file extraction tracer
    pub fn new(file_name: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
            steps: Vec::new(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a step in the extraction process
    pub fn record_step(&mut self, step: &str, details: Option<String>) {
        self.steps.push((step.to_string(), details));

        // Log immediately if tracing is enabled
        if log::log_enabled!(log::Level::Trace) {
            let elapsed = self.start_time.elapsed().as_millis();
            if let Some(ref details) = self.steps.last().unwrap().1 {
                log::trace!("[{}ms] {} - {}: {}", elapsed, self.file_name, step, details);
            } else {
                log::trace!("[{}ms] {} - {}", elapsed, self.file_name, step);
            }
        }
    }

    /// Generate a detailed report of the extraction process
    pub fn generate_report(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("File Extraction Trace: {}\n", self.file_name));
        output.push_str(&format!("{:=<50}\n", ""));

        let total_time = self.start_time.elapsed();

        for (i, (step, details)) in self.steps.iter().enumerate() {
            output.push_str(&format!("{:2}. {}\n", i + 1, step));
            if let Some(details) = details {
                output.push_str(&format!("    └─ {}\n", details));
            }
        }

        output.push_str(&format!(
            "\nTotal extraction time: {:.2}ms\n",
            total_time.as_secs_f64() * 1000.0
        ));
        output
    }
}

/// Compression method analyzer
#[derive(Debug)]
pub struct CompressionAnalyzer {
    results: Vec<CompressionAnalysisResult>,
}

/// Result of compression analysis for a single file
#[derive(Debug, Clone)]
pub struct CompressionAnalysisResult {
    /// Name of the file analyzed
    pub file_name: String,
    /// Block index in the archive
    pub block_index: usize,
    /// Compression mask byte
    pub compression_mask: u8,
    /// List of compression methods used
    pub methods: Vec<&'static str>,
    /// Original uncompressed size
    pub original_size: u64,
    /// Compressed size in archive
    pub compressed_size: u64,
    /// Compression ratio (compressed/original)
    pub ratio: f64,
}

impl Default for CompressionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressionAnalyzer {
    /// Create a new compression analyzer
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Analyze compression methods from a mask
    pub fn analyze_compression_mask(mask: u8) -> Vec<&'static str> {
        let mut methods = Vec::new();

        if mask & 0x02 != 0 {
            methods.push("ZLIB");
        }
        if mask & 0x08 != 0 {
            methods.push("PKWARE");
        }
        if mask & 0x10 != 0 {
            methods.push("BZIP2");
        }
        if mask & 0x20 != 0 {
            methods.push("SPARSE");
        }
        if mask & 0x40 != 0 {
            methods.push("ADPCM_MONO");
        }
        if mask & 0x80 != 0 {
            methods.push("ADPCM_STEREO");
        }
        if mask & 0x12 != 0 {
            methods.push("LZMA");
        }

        if methods.is_empty() {
            methods.push("NONE");
        }

        methods
    }

    /// Add analysis result
    pub fn add_result(
        &mut self,
        file_name: &str,
        block_index: usize,
        compression_mask: u8,
        original_size: u64,
        compressed_size: u64,
    ) {
        let methods = Self::analyze_compression_mask(compression_mask);
        let ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            1.0
        };

        self.results.push(CompressionAnalysisResult {
            file_name: file_name.to_string(),
            block_index,
            compression_mask,
            methods,
            original_size,
            compressed_size,
            ratio,
        });
    }

    /// Generate compression statistics report
    pub fn generate_report(&self) -> String {
        let mut output = String::new();
        output.push_str("Compression Analysis Report\n");
        output.push_str("==========================\n\n");

        // Summary statistics
        let total_original: u64 = self.results.iter().map(|r| r.original_size).sum();
        let total_compressed: u64 = self.results.iter().map(|r| r.compressed_size).sum();
        let overall_ratio = if total_original > 0 {
            total_compressed as f64 / total_original as f64
        } else {
            1.0
        };

        output.push_str(&format!("Total files analyzed: {}\n", self.results.len()));
        output.push_str(&format!(
            "Total original size: {}\n",
            format_size(total_original)
        ));
        output.push_str(&format!(
            "Total compressed size: {}\n",
            format_size(total_compressed)
        ));
        output.push_str(&format!(
            "Overall compression ratio: {:.1}%\n\n",
            overall_ratio * 100.0
        ));

        // Method usage statistics
        let mut method_counts = std::collections::HashMap::new();
        for result in &self.results {
            for method in &result.methods {
                *method_counts.entry(*method).or_insert(0) += 1;
            }
        }

        output.push_str("Compression methods used:\n");
        for (method, count) in method_counts.iter() {
            output.push_str(&format!("  {}: {} files\n", method, count));
        }

        // Detailed results table
        output.push_str("\nDetailed Results:\n");
        output.push_str("-----------------\n");

        let mut table = TableFormatter::new(vec![
            "File",
            "Block",
            "Methods",
            "Original",
            "Compressed",
            "Ratio",
        ]);

        for result in &self.results {
            table.add_row(vec![
                result.file_name.clone(),
                format!("{}", result.block_index),
                result.methods.join(", "),
                format_size(result.original_size),
                format_size(result.compressed_size),
                format!("{:.1}%", result.ratio * 100.0),
            ]);
        }

        output.push_str(&table.format());
        output
    }
}

/// Format HET table for display
pub fn format_het_table(het: &crate::tables::HetTable) -> String {
    let mut output = String::new();

    // Copy packed struct fields to local variables to avoid alignment issues
    let table_size = het.header.table_size;
    let max_file_count = het.header.max_file_count;
    let hash_table_size = het.header.hash_table_size;
    let hash_entry_size = het.header.hash_entry_size;
    let total_index_size = het.header.total_index_size;
    let index_size_extra = het.header.index_size_extra;
    let index_size = het.header.index_size;
    let block_table_size = het.header.block_table_size;

    // Header information
    output.push_str("HET Table Header:\n");
    output.push_str(&format!("  Table Size: {} bytes\n", table_size));
    output.push_str(&format!("  Max File Count: {}\n", max_file_count));
    output.push_str(&format!("  Hash Table Size: {} bytes\n", hash_table_size));
    output.push_str(&format!("  Hash Entry Size: {} bits\n", hash_entry_size));
    output.push_str(&format!("  Total Index Size: {} bits\n", total_index_size));
    output.push_str(&format!("  Index Size Extra: {} bits\n", index_size_extra));
    output.push_str(&format!("  Index Size: {} bits\n", index_size));
    output.push_str(&format!("  Block Table Size: {} bytes\n", block_table_size));

    output
}

/// Format BET table for display
pub fn format_bet_table(bet: &crate::tables::BetTable) -> String {
    let mut output = String::new();

    // Copy packed struct fields to local variables to avoid alignment issues
    let table_size = bet.header.table_size;
    let file_count = bet.header.file_count;
    let unknown_08 = bet.header.unknown_08;
    let table_entry_size = bet.header.table_entry_size;
    let bit_index_file_pos = bet.header.bit_index_file_pos;
    let bit_count_file_pos = bet.header.bit_count_file_pos;
    let bit_index_file_size = bet.header.bit_index_file_size;
    let bit_count_file_size = bet.header.bit_count_file_size;
    let bit_index_cmp_size = bet.header.bit_index_cmp_size;
    let bit_count_cmp_size = bet.header.bit_count_cmp_size;
    let bit_index_flag_index = bet.header.bit_index_flag_index;
    let bit_count_flag_index = bet.header.bit_count_flag_index;
    let bit_index_unknown = bet.header.bit_index_unknown;
    let bit_count_unknown = bet.header.bit_count_unknown;
    let total_bet_hash_size = bet.header.total_bet_hash_size;
    let bet_hash_size_extra = bet.header.bet_hash_size_extra;
    let bet_hash_size = bet.header.bet_hash_size;
    let bet_hash_array_size = bet.header.bet_hash_array_size;
    let flag_count = bet.header.flag_count;

    // Header information
    output.push_str("BET Table Header:\n");
    output.push_str(&format!("  Table Size: {} bytes\n", table_size));
    output.push_str(&format!("  File Count: {}\n", file_count));
    output.push_str(&format!("  Unknown: 0x{:08X}\n", unknown_08));
    output.push_str(&format!("  Table Entry Size: {} bits\n", table_entry_size));

    output.push_str("\nBit Field Positions:\n");
    output.push_str(&format!(
        "  File Position: bit {} (width: {})\n",
        bit_index_file_pos, bit_count_file_pos
    ));
    output.push_str(&format!(
        "  File Size: bit {} (width: {})\n",
        bit_index_file_size, bit_count_file_size
    ));
    output.push_str(&format!(
        "  Compressed Size: bit {} (width: {})\n",
        bit_index_cmp_size, bit_count_cmp_size
    ));
    output.push_str(&format!(
        "  Flag Index: bit {} (width: {})\n",
        bit_index_flag_index, bit_count_flag_index
    ));
    output.push_str(&format!(
        "  Unknown: bit {} (width: {})\n",
        bit_index_unknown, bit_count_unknown
    ));

    output.push_str("\nHash Information:\n");
    output.push_str(&format!(
        "  Total Hash Size: {} bytes\n",
        total_bet_hash_size
    ));
    output.push_str(&format!(
        "  BET Hash Size Extra: {} bits\n",
        bet_hash_size_extra
    ));
    output.push_str(&format!("  BET Hash Size: {} bits\n", bet_hash_size));
    output.push_str(&format!(
        "  BET Hash Array Size: {} bytes\n",
        bet_hash_array_size
    ));
    output.push_str(&format!("  Flag Count: {}\n", flag_count));

    output
}
