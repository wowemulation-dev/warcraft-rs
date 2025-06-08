//! MPQ Archive Rebuild Functionality
//!
//! This module provides functionality to rebuild MPQ archives 1:1, preserving
//! original structure, metadata, and settings while optionally allowing format
//! upgrades and optimizations.

use crate::{
    Archive, ArchiveBuilder, Error, FormatVersion, ListfileOption, Result,
    compression::flags as compression_flags,
};
use std::path::Path;

/// Options for rebuilding an MPQ archive
#[derive(Debug, Clone)]
pub struct RebuildOptions {
    /// Whether to preserve the original MPQ format version
    pub preserve_format: bool,

    /// Target format version (overrides preserve_format if set)
    pub target_format: Option<FormatVersion>,

    /// Whether to preserve original file order
    pub preserve_order: bool,

    /// Whether to skip encrypted files during rebuild
    pub skip_encrypted: bool,

    /// Whether to skip digital signatures
    pub skip_signatures: bool,

    /// Whether to verify the rebuilt archive matches the original
    pub verify: bool,

    /// Override compression method for all files
    pub override_compression: Option<u8>,

    /// Override block size
    pub override_block_size: Option<u16>,

    /// Whether to perform a dry run (list only)
    pub list_only: bool,
}

impl Default for RebuildOptions {
    fn default() -> Self {
        Self {
            preserve_format: true,
            target_format: None,
            preserve_order: true,
            skip_encrypted: false,
            skip_signatures: true,
            verify: false,
            override_compression: None,
            override_block_size: None,
            list_only: false,
        }
    }
}

/// Metadata extracted from the source archive
#[derive(Debug)]
struct ArchiveMetadata {
    format_version: FormatVersion,
    block_size: u16,
    #[allow(dead_code)] // Kept for future validation features
    sector_size: usize,
    has_het_bet: bool,
    #[allow(dead_code)] // Kept for future rebuild strategy features
    has_classic_tables: bool,
    table_compression_enabled: bool,
    table_compression_method: u8,
    file_count: usize,
}

/// Metadata for an individual file in the archive
#[derive(Debug, Clone)]
struct FileMetadata {
    name: String,
    original_index: usize,
    compression: u8,
    encrypted: bool,
    use_fix_key: bool,
    locale: u16,
    #[allow(dead_code)] // Kept for future validation features
    file_size: u64,
    #[allow(dead_code)] // Kept for future validation features
    compressed_size: u64,
    #[allow(dead_code)] // Kept for future validation features
    flags: u32,
}

/// Progress callback for rebuild operations
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send + Sync>;

/// Rebuild an MPQ archive with the specified options
pub fn rebuild_archive<P: AsRef<Path>>(
    source_path: P,
    target_path: P,
    options: RebuildOptions,
    progress_callback: Option<ProgressCallback>,
) -> Result<RebuildSummary> {
    let source_path = source_path.as_ref();
    let target_path = target_path.as_ref();

    log::info!(
        "Starting rebuild: {} -> {}",
        source_path.display(),
        target_path.display()
    );

    // Phase 1: Analyze source archive
    log::debug!("Phase 1: Analyzing source archive");
    let mut source = Archive::open(source_path)?;
    let metadata = analyze_archive(&mut source)?;

    log::info!(
        "Source archive: {:?}, {} files",
        metadata.format_version,
        metadata.file_count
    );

    // Phase 2: Extract files and metadata
    log::debug!("Phase 2: Extracting files and metadata");
    let extracted_files =
        extract_files_with_metadata(&mut source, &metadata, &options, &progress_callback)?;

    let extracted_count = extracted_files.len();
    log::info!("Extracted {} files from source archive", extracted_count);

    if options.list_only {
        return Ok(RebuildSummary {
            source_files: metadata.file_count,
            extracted_files: extracted_count,
            skipped_files: metadata.file_count - extracted_count,
            target_format: determine_target_format(&metadata, &options),
            verified: false,
        });
    }

    // Phase 3: Rebuild archive
    log::debug!("Phase 3: Rebuilding archive");
    let target_format = determine_target_format(&metadata, &options);
    rebuild_with_files(
        target_path,
        &metadata,
        extracted_files,
        target_format,
        &options,
    )?;

    log::info!("Successfully rebuilt archive: {}", target_path.display());

    // Phase 4: Verification (optional)
    let verified = if options.verify {
        log::debug!("Phase 4: Verifying rebuilt archive");
        verify_rebuild(source_path, target_path, &options)?;
        true
    } else {
        false
    };

    Ok(RebuildSummary {
        source_files: metadata.file_count,
        extracted_files: extracted_count,
        skipped_files: metadata.file_count - extracted_count,
        target_format,
        verified,
    })
}

/// Summary of rebuild operation
#[derive(Debug)]
pub struct RebuildSummary {
    /// Number of files in the source archive
    pub source_files: usize,
    /// Number of files successfully extracted and added to target
    pub extracted_files: usize,
    /// Number of files skipped during rebuild
    pub skipped_files: usize,
    /// Format version of the target archive
    pub target_format: FormatVersion,
    /// Whether the rebuilt archive was verified against original
    pub verified: bool,
}

/// Analyze the source archive to extract metadata
fn analyze_archive(archive: &mut Archive) -> Result<ArchiveMetadata> {
    // Get archive info first (requires mutable borrow)
    let info = archive.get_info()?;

    // Then get header (immutable borrow after mutable borrow is dropped)
    let header = archive.header();

    Ok(ArchiveMetadata {
        format_version: header.format_version,
        block_size: header.block_size,
        sector_size: crate::calculate_sector_size(header.block_size),
        has_het_bet: info.het_table_info.is_some() && info.bet_table_info.is_some(),
        has_classic_tables: info.hash_table_info.size.is_some()
            && info.block_table_info.size.is_some(),
        table_compression_enabled: false, // TODO: Detect from archive
        table_compression_method: compression_flags::ZLIB,
        file_count: info.file_count,
    })
}

/// Extract files with their metadata from the source archive
fn extract_files_with_metadata(
    archive: &mut Archive,
    metadata: &ArchiveMetadata,
    options: &RebuildOptions,
    progress_callback: &Option<ProgressCallback>,
) -> Result<Vec<(Vec<u8>, FileMetadata)>> {
    // Get file list, preferring the most complete method
    let files = if metadata.has_het_bet {
        archive
            .list_all_with_hashes()
            .unwrap_or_else(|_| archive.list().unwrap_or_default())
    } else {
        archive
            .list()
            .unwrap_or_else(|_| archive.list_all().unwrap_or_default())
    };

    let mut extracted_files = Vec::new();
    let total_files = files.len();

    for (i, file) in files.iter().enumerate() {
        // Report progress
        if let Some(callback) = progress_callback {
            callback(i + 1, total_files, &file.name);
        }

        // Apply filters
        if options.skip_signatures && is_signature_file(&file.name) {
            log::debug!("Skipping signature file: {}", file.name);
            continue;
        }

        if options.skip_encrypted && file.flags & crate::tables::BlockEntry::FLAG_ENCRYPTED != 0 {
            log::debug!("Skipping encrypted file: {}", file.name);
            continue;
        }

        // Extract file data
        let data = match archive.read_file(&file.name) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Failed to read file {}: {}", file.name, e);
                continue;
            }
        };

        // Extract metadata
        let file_meta = FileMetadata {
            name: file.name.clone(),
            original_index: i, // Use list order as proxy for original index
            compression: extract_compression_method(file.flags),
            encrypted: file.flags & crate::tables::BlockEntry::FLAG_ENCRYPTED != 0,
            use_fix_key: file.flags & crate::tables::BlockEntry::FLAG_FIX_KEY != 0,
            locale: 0, // TODO: Extract actual locale from hash table
            file_size: file.size,
            compressed_size: file.compressed_size,
            flags: file.flags,
        };

        extracted_files.push((data, file_meta));
    }

    // Sort by original index if preserving order
    if options.preserve_order {
        extracted_files.sort_by_key(|(_, meta)| meta.original_index);
    }

    Ok(extracted_files)
}

/// Rebuild the archive with extracted files
fn rebuild_with_files(
    target_path: &Path,
    metadata: &ArchiveMetadata,
    files: Vec<(Vec<u8>, FileMetadata)>,
    target_format: FormatVersion,
    options: &RebuildOptions,
) -> Result<()> {
    let mut builder = ArchiveBuilder::new()
        .version(target_format)
        .block_size(options.override_block_size.unwrap_or(metadata.block_size));

    // Configure based on target format
    if target_format >= FormatVersion::V3 {
        builder = builder
            .compress_tables(metadata.table_compression_enabled)
            .table_compression(metadata.table_compression_method);
    }

    // Determine listfile strategy
    let has_listfile = files.iter().any(|(_, meta)| meta.name == "(listfile)");
    builder = builder.listfile_option(if has_listfile {
        ListfileOption::None // We'll add it manually to preserve content
    } else {
        ListfileOption::Generate
    });

    // Add files in order
    for (data, meta) in files {
        let compression = options.override_compression.unwrap_or(meta.compression);

        if meta.encrypted && meta.use_fix_key {
            builder = builder.add_file_data_with_encryption(
                data,
                &meta.name,
                compression,
                meta.use_fix_key,
                meta.locale,
            );
        } else if meta.encrypted {
            builder = builder.add_file_data_with_options(
                data,
                &meta.name,
                compression,
                true, // encrypted
                meta.locale,
            );
        } else {
            builder = builder.add_file_data_with_options(
                data,
                &meta.name,
                compression,
                false, // not encrypted
                meta.locale,
            );
        }
    }

    builder.build(target_path)?;
    Ok(())
}

/// Determine the target format based on options and source metadata
fn determine_target_format(metadata: &ArchiveMetadata, options: &RebuildOptions) -> FormatVersion {
    if let Some(target_format) = options.target_format {
        target_format
    } else if options.preserve_format {
        metadata.format_version
    } else {
        // Default to modernizing to V4 if not preserving format
        FormatVersion::V4
    }
}

/// Extract compression method from block entry flags
fn extract_compression_method(flags: u32) -> u8 {
    if flags & crate::tables::BlockEntry::FLAG_COMPRESS != 0 {
        // Default to ZLIB if compressed (we can't determine exact method from flags alone)
        compression_flags::ZLIB
    } else {
        0 // No compression
    }
}

/// Check if a file is a digital signature file
fn is_signature_file(filename: &str) -> bool {
    matches!(filename, "(signature)" | "(strong signature)")
}

/// Verify that the rebuilt archive matches the original
fn verify_rebuild(source_path: &Path, target_path: &Path, options: &RebuildOptions) -> Result<()> {
    let mut source_archive = Archive::open(source_path)?;
    let mut target_archive = Archive::open(target_path)?;

    let source_files = source_archive
        .list()
        .unwrap_or_else(|_| source_archive.list_all().unwrap_or_default());
    let target_files = target_archive
        .list()
        .unwrap_or_else(|_| target_archive.list_all().unwrap_or_default());

    // Calculate expected file count after filtering
    let mut expected_files = Vec::new();
    for file in &source_files {
        if options.skip_signatures && is_signature_file(&file.name) {
            continue;
        }
        if options.skip_encrypted && file.flags & crate::tables::BlockEntry::FLAG_ENCRYPTED != 0 {
            continue;
        }
        expected_files.push(&file.name);
    }

    if target_files.len() != expected_files.len() {
        return Err(Error::invalid_format(format!(
            "File count mismatch: expected {}, got {}",
            expected_files.len(),
            target_files.len()
        )));
    }

    // Verify each file's content
    for expected_file in &expected_files {
        let source_data = source_archive.read_file(expected_file)?;
        let target_data = target_archive.read_file(expected_file).map_err(|e| {
            Error::invalid_format(format!("File {} missing in target: {}", expected_file, e))
        })?;

        if source_data != target_data {
            return Err(Error::invalid_format(format!(
                "Content mismatch for file: {}",
                expected_file
            )));
        }
    }

    log::info!("✅ Verification successful - rebuilt archive matches original");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rebuild_options_default() {
        let options = RebuildOptions::default();
        assert!(options.preserve_format);
        assert!(options.preserve_order);
        assert!(!options.skip_encrypted);
        assert!(options.skip_signatures);
        assert!(!options.verify);
    }

    #[test]
    fn test_is_signature_file() {
        assert!(is_signature_file("(signature)"));
        assert!(is_signature_file("(strong signature)"));
        assert!(!is_signature_file("(listfile)"));
        assert!(!is_signature_file("normal_file.txt"));
    }

    #[test]
    fn test_extract_compression_method() {
        assert_eq!(extract_compression_method(0), 0);
        assert_eq!(
            extract_compression_method(crate::tables::BlockEntry::FLAG_COMPRESS),
            compression_flags::ZLIB
        );
    }
}
