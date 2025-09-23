//! MPQ archive command implementations

use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum};
use std::fs;
use std::path::Path;
use wow_mpq::{
    Archive, ArchiveBuilder, FormatVersion, PatchChain, RebuildOptions,
    compare_archives as mpq_compare_archives,
    debug::{
        HexDumpConfig, dump_block_entry, dump_hash_entry, format_bet_table, format_block_table,
        format_hash_table, format_het_table, hex_dump,
    },
    path::mpq_path_to_system,
    rebuild_archive,
    single_archive_parallel::{ParallelArchive, ParallelConfig},
};

use crate::utils::{
    NodeType, TreeNode, TreeOptions, add_table_row, create_progress_bar, create_spinner,
    create_table, detect_ref_type, format_bytes, format_compression_ratio, matches_pattern,
    render_tree, truncate_path,
};

#[derive(ValueEnum, Clone, Debug)]
pub enum VersionArg {
    V1,
    V2,
    V3,
    V4,
}

impl From<VersionArg> for FormatVersion {
    fn from(arg: VersionArg) -> Self {
        match arg {
            VersionArg::V1 => FormatVersion::V1,
            VersionArg::V2 => FormatVersion::V2,
            VersionArg::V3 => FormatVersion::V3,
            VersionArg::V4 => FormatVersion::V4,
        }
    }
}

#[derive(Subcommand)]
pub enum MpqCommands {
    /// Show information about an MPQ archive
    Info {
        /// Path to the MPQ archive
        archive: String,

        /// Show hash table details
        #[arg(long)]
        show_hash_table: bool,

        /// Show block table details
        #[arg(long)]
        show_block_table: bool,
    },

    /// Validate integrity of an MPQ archive
    Validate {
        /// Path to the MPQ archive
        archive: String,

        /// Check CRC/MD5 checksums if available
        #[arg(long)]
        check_checksums: bool,

        /// Number of threads for parallel validation (default: CPU cores)
        #[arg(long)]
        threads: Option<usize>,
    },

    /// List files in an MPQ archive
    List {
        /// Path to the MPQ archive
        archive: String,

        /// Show detailed information (size, compression ratio)
        #[arg(short, long)]
        long: bool,

        /// Filter files by pattern (supports wildcards)
        #[arg(short, long)]
        filter: Option<String>,

        /// Use hash database for name resolution
        #[arg(long)]
        use_db: bool,

        /// Automatically record found filenames to database
        #[arg(long)]
        record_to_db: bool,
    },

    /// Extract files from an MPQ archive
    Extract {
        /// Path to the MPQ archive
        archive: String,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,

        /// Specific files to extract (extracts all if not specified)
        files: Vec<String>,

        /// File types to extract (e.g. ".txt", "jpg"). Case-insensitive. Ignored if \[FILES\] are specified.
        #[arg(short, long)]
        file_type: Option<String>,

        /// Preserve directory structure
        #[arg(short, long)]
        preserve_paths: bool,

        /// Number of threads for parallel extraction (default: CPU cores)
        #[arg(long)]
        threads: Option<usize>,

        /// Continue extraction even if some files fail
        #[arg(long)]
        skip_errors: bool,

        /// Patch archives to apply (in order of priority)
        #[arg(long = "patch", action = clap::ArgAction::Append)]
        patches: Vec<String>,
    },

    /// Create a new MPQ archive
    Create {
        /// Path for the new MPQ archive
        archive: String,

        /// Files to add to the archive
        #[arg(short, long, required = true)]
        add: Vec<String>,

        /// Archive format version (v1, v2, v3, v4)
        #[arg(long, default_value = "v2")]
        version: String,

        /// Compression method (none, zlib, bzip2, lzma)
        #[arg(short, long, default_value = "zlib")]
        compression: String,

        /// Create or update (listfile)
        #[arg(long)]
        with_listfile: bool,
    },

    /// Rebuild an MPQ archive 1:1
    Rebuild {
        /// Source MPQ archive
        source: String,

        /// Target path for rebuilt archive
        target: String,

        /// Preserve original MPQ format version
        #[arg(long, default_value_t = true)]
        preserve_format: bool,

        /// Upgrade to specific format version
        #[arg(long, value_enum)]
        upgrade_to: Option<VersionArg>,

        /// Skip encrypted files
        #[arg(long)]
        skip_encrypted: bool,

        /// Skip digital signatures
        #[arg(long, default_value_t = true)]
        skip_signatures: bool,

        /// Verify rebuilt archive matches original
        #[arg(long)]
        verify: bool,

        /// Override compression method
        #[arg(long)]
        compression: Option<String>,

        /// Override block size
        #[arg(long)]
        block_size: Option<u16>,

        /// List files that would be processed (dry run)
        #[arg(long)]
        list_only: bool,
    },

    /// Compare two MPQ archives
    Compare {
        /// Source MPQ archive
        source: String,

        /// Target MPQ archive to compare against
        target: String,

        /// Show detailed file-by-file comparison
        #[arg(short, long)]
        detailed: bool,

        /// Compare actual file contents (slower but thorough)
        #[arg(long)]
        content_check: bool,

        /// Only compare archive metadata
        #[arg(long)]
        metadata_only: bool,

        /// Ignore file order differences
        #[arg(long)]
        ignore_order: bool,

        /// Output format: table, json, summary
        #[arg(long, default_value = "table")]
        output: String,

        /// Filter files by pattern (supports wildcards)
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Show tree structure of an MPQ archive
    Tree {
        /// Path to the MPQ archive
        archive: String,

        /// Maximum depth to display
        #[arg(long)]
        depth: Option<usize>,

        /// Hide external file references
        #[arg(long)]
        no_external_refs: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,

        /// Show compact metadata inline
        #[arg(long)]
        compact: bool,

        /// Filter files by pattern (supports wildcards)
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Debug MPQ archive internals
    Debug {
        /// Path to the MPQ archive
        archive: String,

        /// Show hash table entries
        #[arg(long)]
        hash_table: bool,

        /// Show block table entries
        #[arg(long)]
        block_table: bool,

        /// Show HET table (if present)
        #[arg(long)]
        het_table: bool,

        /// Show BET table (if present)
        #[arg(long)]
        bet_table: bool,

        /// Show specific entry by index
        #[arg(long)]
        entry: Option<usize>,

        /// Find and show entries for a specific file
        #[arg(long)]
        find: Option<String>,

        /// Show raw hex dump of table data
        #[arg(long)]
        raw: bool,

        /// Show all tables
        #[arg(long)]
        all: bool,
    },

    /// Database operations for MPQ hash resolution
    #[command(subcommand)]
    Db(DbCommands),
}

#[derive(Subcommand)]
pub enum DbCommands {
    /// Initialize or show status of the hash database
    Status {
        /// Show detailed statistics
        #[arg(long)]
        detailed: bool,
    },

    /// Import filenames from various sources
    Import {
        /// Path to import from (listfile, MPQ archive, or directory)
        path: String,

        /// Source type
        #[arg(value_enum)]
        source_type: ImportSourceArg,

        /// Show progress
        #[arg(long)]
        show_progress: bool,
    },

    /// Analyze an MPQ archive and record its filenames
    Analyze {
        /// Path to the MPQ archive
        archive: String,

        /// Also record anonymous files with generated names
        #[arg(long)]
        include_anonymous: bool,
    },

    /// Look up a filename's hash values
    Lookup {
        /// Filename to look up
        filename: String,
    },

    /// Export database to listfile format
    Export {
        /// Output file path
        output: String,

        /// Filter by source
        #[arg(long)]
        source: Option<String>,
    },

    /// List entries in the database
    List {
        /// Filter entries by pattern (supports wildcards)
        #[arg(short, long)]
        filter: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        long: bool,

        /// Limit number of results
        #[arg(short = 'n', long, default_value = "100")]
        limit: usize,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ImportSourceArg {
    /// Import from a listfile
    Listfile,
    /// Import from an MPQ archive's internal listfile
    Archive,
    /// Scan a directory for WoW file patterns
    Directory,
}

pub fn execute(command: MpqCommands) -> Result<()> {
    match command {
        MpqCommands::List {
            archive,
            long,
            filter,
            use_db,
            record_to_db,
        } => list_archive(&archive, long, filter, use_db, record_to_db),
        MpqCommands::Extract {
            archive,
            output,
            files,
            file_type,
            preserve_paths,
            threads,
            skip_errors,
            patches,
        } => extract_files(
            &archive,
            &output,
            files,
            file_type,
            preserve_paths,
            threads,
            skip_errors,
            patches,
        ),
        MpqCommands::Create {
            archive,
            add,
            version,
            compression,
            with_listfile,
        } => create_archive(&archive, add, &version, &compression, with_listfile),
        MpqCommands::Info {
            archive,
            show_hash_table,
            show_block_table,
        } => show_info(&archive, show_hash_table, show_block_table),
        MpqCommands::Validate {
            archive,
            check_checksums,
            threads,
        } => validate_archive(&archive, check_checksums, threads),
        MpqCommands::Rebuild {
            source,
            target,
            preserve_format,
            upgrade_to,
            skip_encrypted,
            skip_signatures,
            verify,
            compression,
            block_size,
            list_only,
        } => rebuild_mpq_archive(RebuildParams {
            source_path: &source,
            target_path: &target,
            preserve_format,
            upgrade_to,
            skip_encrypted,
            skip_signatures,
            verify,
            compression,
            block_size,
            list_only,
        }),
        MpqCommands::Compare {
            source,
            target,
            detailed,
            content_check,
            metadata_only,
            ignore_order,
            output,
            filter,
        } => compare_archives(CompareParams {
            source_path: &source,
            target_path: &target,
            detailed,
            content_check,
            metadata_only,
            ignore_order,
            output_format: &output,
            filter,
        }),
        MpqCommands::Tree {
            archive,
            depth,
            no_external_refs,
            no_color,
            compact,
            filter,
        } => show_tree(
            &archive,
            depth,
            !no_external_refs,
            no_color,
            compact,
            filter,
        ),
        MpqCommands::Debug {
            archive,
            hash_table,
            block_table,
            het_table,
            bet_table,
            entry,
            find,
            raw,
            all,
        } => debug_archive(DebugParams {
            archive_path: &archive,
            show_hash_table: hash_table || all,
            show_block_table: block_table || all,
            show_het_table: het_table || all,
            show_bet_table: bet_table || all,
            entry_index: entry,
            find_file: find,
            raw_dump: raw,
        }),
        MpqCommands::Db(db_command) => execute_db_command(db_command),
    }
}

fn list_archive(
    path: &str,
    long: bool,
    filter: Option<String>,
    use_db: bool,
    record_to_db: bool,
) -> Result<()> {
    use wow_mpq::database::Database;

    let spinner = create_spinner("Opening archive...");
    let mut archive = Archive::open(path).context("Failed to open archive")?;
    spinner.finish_and_clear();

    // Open database if needed
    let db = if use_db || record_to_db {
        Some(Database::open_default().context("Failed to open database")?)
    } else {
        None
    };

    // Record filenames to database if requested
    if record_to_db {
        if let Some(ref db) = db {
            let count = archive.record_listfile_to_db(db)?;
            if count > 0 {
                println!("Recorded {count} filenames to database");
            }
        }
    }

    // Get file list
    let files: Vec<String> = if use_db {
        if let Some(ref db) = db {
            archive
                .list_with_db(db)?
                .into_iter()
                .map(|e| e.name)
                .collect()
        } else {
            archive.list()?.into_iter().map(|e| e.name).collect()
        }
    } else {
        archive.list()?.into_iter().map(|e| e.name).collect()
    };

    let pattern = filter.as_deref().unwrap_or("*");

    let mut filtered_files: Vec<_> = files
        .iter()
        .filter(|f| matches_pattern(f, pattern))
        .collect();
    filtered_files.sort();

    if filtered_files.is_empty() {
        println!("No files found matching pattern: {pattern}");
        return Ok(());
    }

    if long {
        let mut table = create_table(vec!["File", "Size", "Compressed", "Ratio"]);

        for file in filtered_files {
            // Try to get file info from the entries
            if let Ok(entries) = archive.list() {
                if let Some(entry) = entries.iter().find(|e| &e.name == file) {
                    add_table_row(
                        &mut table,
                        vec![
                            truncate_path(file, 50),
                            format_bytes(entry.size),
                            format_bytes(entry.compressed_size),
                            format_compression_ratio(entry.size, entry.compressed_size),
                        ],
                    );
                }
            }
        }

        table.printstd();
    } else {
        for file in filtered_files {
            println!("{file}");
        }
    }

    Ok(())
}

struct ExtractOptions {
    archive_path: String,
    output_dir: String,
    files: Vec<String>,
    file_type: Option<String>,
    preserve_paths: bool,
    threads: Option<usize>,
    skip_errors: bool,
    patches: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
fn extract_files(
    archive_path: &str,
    output_dir: &str,
    files: Vec<String>,
    file_type: Option<String>,
    preserve_paths: bool,
    threads: Option<usize>,
    skip_errors: bool,
    patches: Vec<String>,
) -> Result<()> {
    let options = ExtractOptions {
        archive_path: archive_path.to_string(),
        output_dir: output_dir.to_string(),
        files,
        file_type,
        preserve_paths,
        threads,
        skip_errors,
        patches,
    };

    extract_files_with_options(options)
}

fn extract_files_with_options(options: ExtractOptions) -> Result<()> {
    let ExtractOptions {
        archive_path,
        output_dir,
        files,
        file_type,
        preserve_paths,
        threads,
        skip_errors,
        patches,
    } = options;
    if patches.is_empty() {
        // Use parallel extraction by default
        let files_to_extract: Vec<String> = if files.is_empty() {
            // For bulk extraction, read listfile directly to avoid slow database lookups
            println!("Reading file list from archive...");
            let mut archive = Archive::open(&archive_path).context("Failed to open archive")?;

            // Try to read (listfile) directly for faster bulk operations
            let mut file_list = match archive.read_file("(listfile)") {
                Ok(listfile_data) => {
                    println!("Parsing listfile...");
                    match wow_mpq::special_files::parse_listfile(&listfile_data) {
                        Ok(filenames) => {
                            println!("Found {} files in listfile", filenames.len());
                            filenames
                        }
                        Err(_) => {
                            println!(
                                "Failed to parse listfile, falling back to slow enumeration..."
                            );
                            let file_list = archive
                                .list()?
                                .into_iter()
                                .map(|e| e.name)
                                .collect::<Vec<String>>();
                            println!("Found {} files", file_list.len());
                            file_list
                        }
                    }
                }
                Err(_) => {
                    println!("No listfile found, using slow enumeration...");
                    let file_list = archive
                        .list()?
                        .into_iter()
                        .map(|e| e.name)
                        .collect::<Vec<String>>();
                    println!("Found {} files", file_list.len());
                    file_list
                }
            };

            // Filter by file type if specified
            if let Some(file_type) = file_type {
                println!("Filtering by file type...");
                file_list.retain(|f| {
                    // Ignore case
                    let lowercase_filename = f.to_lowercase();
                    let lowercase_file_type = file_type.to_lowercase();
                    lowercase_filename.ends_with(&lowercase_file_type)
                });
                println!(
                    "Found {} files matching type {}",
                    file_list.len(),
                    file_type
                );
            }

            file_list
        } else {
            files
        };

        let pb = create_progress_bar(files_to_extract.len() as u64, "Extracting files");

        // Configure parallel extraction with sensible defaults for large extractions
        let default_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let batch_size = if files_to_extract.len() > 5000 {
            // For very large extractions, use bigger batches to reduce overhead
            std::cmp::max(
                50,
                files_to_extract.len() / (threads.unwrap_or(default_threads) * 4),
            )
        } else if files_to_extract.len() > 1000 {
            // For moderately large extractions, use medium batches
            25
        } else {
            // For small extractions, use small batches
            10
        };

        let mut config = ParallelConfig::new()
            .batch_size(batch_size)
            .skip_errors(skip_errors);

        if let Some(num_threads) = threads {
            config = config.threads(num_threads);
        }

        // Extract files in parallel using the direct API (avoids slow ParallelArchive::open)
        let file_refs: Vec<&str> = files_to_extract.iter().map(|s| s.as_str()).collect();
        use wow_mpq::single_archive_parallel::extract_with_config;
        let results = extract_with_config(archive_path, &file_refs, config)?;

        // Write extracted files to disk
        let mut success_count = 0;
        let mut error_count = 0;

        for (file, data_result) in results {
            pb.set_message(format!("Writing: {file}"));

            match data_result {
                Ok(data) => {
                    let output_path = if preserve_paths {
                        let system_path = mpq_path_to_system(&file);
                        Path::new(&output_dir).join(system_path)
                    } else {
                        let system_path = mpq_path_to_system(&file);
                        let filename = Path::new(&system_path).file_name().unwrap_or_default();
                        Path::new(&output_dir).join(filename)
                    };

                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::write(&output_path, data)?;
                    success_count += 1;
                }
                Err(e) => {
                    log::warn!("Failed to extract {file}: {e}");
                    error_count += 1;
                }
            }

            pb.inc(1);
        }

        let msg = if error_count > 0 {
            format!("Extraction complete: {success_count} succeeded, {error_count} failed")
        } else {
            format!("Extraction complete: {success_count} files")
        };
        pb.finish_with_message(msg);
    } else {
        // Use patch chain logic
        let spinner = create_spinner("Building patch chain...");
        let mut chain = PatchChain::new();

        // Add base archive with priority 0
        chain
            .add_archive(archive_path, 0)
            .context("Failed to add base archive to patch chain")?;

        // Add patch archives with increasing priority
        for (index, patch_path) in patches.iter().enumerate() {
            let priority = (index + 1) * 100;
            chain
                .add_archive(patch_path, priority as i32)
                .with_context(|| format!("Failed to add patch archive: {patch_path}"))?;
        }

        spinner.finish_and_clear();

        println!("Patch chain built with {} archives", chain.archive_count());

        let files_to_extract = if files.is_empty() {
            // Get all files from the chain
            let entries = chain.list()?;
            entries.into_iter().map(|e| e.name).collect()
        } else {
            files
        };

        let pb = create_progress_bar(files_to_extract.len() as u64, "Extracting files");

        for file in files_to_extract.iter() {
            pb.set_message(format!("Extracting: {file}"));

            match chain.read_file(file) {
                Ok(data) => {
                    let output_path = if preserve_paths {
                        // Convert MPQ path separators to system path separators
                        let system_path = mpq_path_to_system(file);
                        Path::new(&output_dir).join(system_path)
                    } else {
                        // Convert MPQ path to system path, then extract just the filename
                        let system_path = mpq_path_to_system(file);
                        let filename = Path::new(&system_path).file_name().unwrap_or_default();
                        Path::new(&output_dir).join(filename)
                    };

                    if let Some(parent) = output_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::write(&output_path, data)?;

                    // Show which archive the file came from
                    if let Some(source) = chain.find_file_archive(file) {
                        log::debug!("Extracted {} from {}", file, source.display());
                    }
                }
                Err(e) => {
                    log::warn!("Failed to extract {file}: {e}");
                }
            }

            pb.inc(1);
        }

        pb.finish_with_message("Extraction complete");

        // Show patch chain info
        println!("\nPatch chain info:");
        for info in chain.get_chain_info() {
            println!(
                "  {} (priority {}, {} files)",
                info.path.display(),
                info.priority,
                info.file_count
            );
        }
    }

    Ok(())
}

fn create_archive(
    path: &str,
    files: Vec<String>,
    version: &str,
    compression: &str,
    with_listfile: bool,
) -> Result<()> {
    let mut builder = ArchiveBuilder::new();

    // Parse version
    let format_version = match version {
        "v1" => FormatVersion::V1,
        "v2" => FormatVersion::V2,
        "v3" => FormatVersion::V3,
        "v4" => FormatVersion::V4,
        _ => anyhow::bail!("Invalid version: {}", version),
    };
    builder = builder.version(format_version);

    // Parse compression
    let compression_flags = match compression {
        "none" => 0,
        "zlib" => wow_mpq::compression::flags::ZLIB,
        "bzip2" => wow_mpq::compression::flags::BZIP2,
        "lzma" => wow_mpq::compression::flags::LZMA,
        _ => anyhow::bail!("Invalid compression: {}", compression),
    };
    builder = builder.default_compression(compression_flags);

    if with_listfile {
        builder = builder.listfile_option(wow_mpq::ListfileOption::Generate);
    }

    let pb = create_progress_bar(files.len() as u64, "Adding files");

    for file_path in files {
        pb.set_message(format!("Adding: {file_path}"));
        let data = fs::read(&file_path)?;
        let archive_path = Path::new(&file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&file_path);
        builder = builder.add_file_data(data, archive_path);
        pb.inc(1);
    }

    pb.finish_and_clear();

    let spinner = create_spinner("Building archive...");
    builder.build(path)?;
    spinner.finish_with_message("Archive created successfully");

    Ok(())
}

fn show_info(path: &str, include_hash_table: bool, include_block_table: bool) -> Result<()> {
    let spinner = create_spinner("Opening archive...");
    let mut archive = Archive::open(path).context("Failed to open archive")?;
    spinner.finish_and_clear();

    let info = archive.get_info()?;

    println!("MPQ Archive Information");
    println!("======================");
    println!("Path: {path}");
    println!("Format version: {:?}", info.format_version);
    println!("Archive size: {}", format_bytes(info.file_size));
    println!("Number of files: {}", info.file_count);

    if include_hash_table {
        println!();
        show_hash_table(&mut archive, false)?;
    }

    if include_block_table {
        println!();
        show_block_table(&mut archive, false)?;
    }

    Ok(())
}

fn validate_archive(path: &str, check_checksums: bool, threads: Option<usize>) -> Result<()> {
    // Use parallel validation by default
    let spinner = create_spinner("Opening archive...");
    let parallel_archive = ParallelArchive::open(path).context("Failed to open archive")?;
    spinner.finish_and_clear();

    let files: Vec<&str> = parallel_archive
        .list_files()
        .iter()
        .map(|s| s.as_str())
        .collect();
    let pb = create_progress_bar(files.len() as u64, "Validating files");

    // Configure parallel processing
    let mut config = ParallelConfig::new().skip_errors(true);
    if let Some(num_threads) = threads {
        config = config.threads(num_threads);
    }

    // Validate files by trying to read them
    use wow_mpq::single_archive_parallel::extract_with_config;
    let validation_results = extract_with_config(path, &files, config)?;

    let mut errors = 0;
    let mut total_size = 0u64;

    for (filename, result) in validation_results {
        pb.set_message(format!("Validating: {filename}"));
        match result {
            Ok(data) => {
                total_size += data.len() as u64;
                if check_checksums {
                    // TODO: Implement checksum validation
                }
            }
            Err(e) => {
                log::error!("Failed to read {filename}: {e}");
                errors += 1;
            }
        }
        pb.inc(1);
    }

    pb.finish_and_clear();

    if errors == 0 {
        println!(
            "✓ Archive validation passed - {} files ({} total)",
            files.len(),
            format_bytes(total_size)
        );
    } else {
        println!("✗ Archive validation failed with {errors} errors");
        println!(
            "  Successfully validated: {} files ({})",
            files.len() - errors,
            format_bytes(total_size)
        );
    }

    Ok(())
}

/// Parameters for MPQ archive rebuild operation
struct RebuildParams<'a> {
    source_path: &'a str,
    target_path: &'a str,
    preserve_format: bool,
    upgrade_to: Option<VersionArg>,
    skip_encrypted: bool,
    skip_signatures: bool,
    verify: bool,
    compression: Option<String>,
    block_size: Option<u16>,
    list_only: bool,
}

fn rebuild_mpq_archive(params: RebuildParams<'_>) -> Result<()> {
    // Parse compression override if provided
    let override_compression = if let Some(comp) = params.compression {
        let compression_flags = match comp.as_str() {
            "none" => 0,
            "zlib" => wow_mpq::compression::flags::ZLIB,
            "bzip2" => wow_mpq::compression::flags::BZIP2,
            "lzma" => wow_mpq::compression::flags::LZMA,
            _ => anyhow::bail!("Invalid compression: {}", comp),
        };
        Some(compression_flags)
    } else {
        None
    };

    // Set up rebuild options
    let options = RebuildOptions {
        preserve_format: params.preserve_format,
        target_format: params.upgrade_to.map(|v| v.into()),
        preserve_order: true,
        skip_encrypted: params.skip_encrypted,
        skip_signatures: params.skip_signatures,
        verify: params.verify,
        override_compression,
        override_block_size: params.block_size,
        list_only: params.list_only,
    };

    if params.list_only {
        println!("Analyzing source archive: {}", params.source_path);
    } else {
        println!(
            "Rebuilding archive: {} -> {}",
            params.source_path, params.target_path
        );
    }

    // Set up progress callback
    let progress_callback = Some(Box::new(|current: usize, total: usize, file: &str| {
        if current % 100 == 0 || current == total {
            println!("  [{current}/{total}] Processing: {file}");
        }
    }) as Box<dyn Fn(usize, usize, &str) + Send + Sync>);

    // Perform the rebuild
    let spinner = if params.list_only {
        create_spinner("Analyzing archive...")
    } else {
        create_spinner("Rebuilding archive...")
    };

    let summary = rebuild_archive(
        params.source_path,
        params.target_path,
        options,
        progress_callback,
    )
    .context("Failed to rebuild archive")?;

    spinner.finish_and_clear();

    // Display results
    println!("\nRebuild Summary:");
    println!("================");
    println!("Source files: {}", summary.source_files);
    println!("Extracted files: {}", summary.extracted_files);
    if summary.skipped_files > 0 {
        println!("Skipped files: {}", summary.skipped_files);
    }
    println!("Target format: {:?}", summary.target_format);

    if params.list_only {
        println!("\nDry run completed. Use without --list-only to perform actual rebuild.");
    } else {
        if summary.verified {
            println!("✓ Verification: PASSED");
        } else if params.verify {
            println!("⚠ Verification: SKIPPED");
        }
        println!("✓ Archive rebuilt successfully: {}", params.target_path);
    }

    Ok(())
}

/// Parameters for MPQ archive comparison operation
struct CompareParams<'a> {
    source_path: &'a str,
    target_path: &'a str,
    detailed: bool,
    content_check: bool,
    metadata_only: bool,
    ignore_order: bool,
    output_format: &'a str,
    filter: Option<String>,
}

/// Parameters for MPQ archive debug operation
struct DebugParams<'a> {
    archive_path: &'a str,
    show_hash_table: bool,
    show_block_table: bool,
    show_het_table: bool,
    show_bet_table: bool,
    entry_index: Option<usize>,
    find_file: Option<String>,
    raw_dump: bool,
}

fn compare_archives(params: CompareParams<'_>) -> Result<()> {
    let spinner = create_spinner("Comparing archives...");

    let comparison_result = mpq_compare_archives(
        params.source_path,
        params.target_path,
        params.detailed,
        params.content_check,
        params.metadata_only,
        params.ignore_order,
        params.filter,
    )?;

    spinner.finish_and_clear();

    // Display results based on output format
    match params.output_format {
        "json" => {
            display_json_output(&comparison_result)?;
        }
        "summary" => {
            display_summary_output(params.source_path, params.target_path, &comparison_result)?;
        }
        "table" => {
            display_table_output(params.source_path, params.target_path, &comparison_result)?;
        }
        _ => {
            display_table_output(params.source_path, params.target_path, &comparison_result)?;
        }
    }

    Ok(())
}

fn display_summary_output(
    source_path: &str,
    target_path: &str,
    result: &wow_mpq::ComparisonResult,
) -> Result<()> {
    println!("Archive Comparison Summary");
    println!("=========================");
    println!("Source: {source_path}");
    println!("Target: {target_path}");
    println!();

    if result.identical {
        println!("✓ Archives are identical");
        return Ok(());
    }

    println!("✗ Archives differ");
    println!();

    // Metadata differences
    if !result.metadata.matches {
        println!("Metadata Differences:");
        if result.metadata.format_version.0 != result.metadata.format_version.1 {
            println!(
                "  Format Version: {:?} → {:?}",
                result.metadata.format_version.0, result.metadata.format_version.1
            );
        }
        if result.metadata.block_size.0 != result.metadata.block_size.1 {
            println!(
                "  Block Size: {} → {}",
                result.metadata.block_size.0, result.metadata.block_size.1
            );
        }
        if result.metadata.file_count.0 != result.metadata.file_count.1 {
            println!(
                "  File Count: {} → {}",
                result.metadata.file_count.0, result.metadata.file_count.1
            );
        }
        if result.metadata.archive_size.0 != result.metadata.archive_size.1 {
            println!(
                "  Archive Size: {} → {}",
                format_bytes(result.metadata.archive_size.0),
                format_bytes(result.metadata.archive_size.1)
            );
        }
        println!();
    }

    // File differences
    if let Some(files) = &result.files {
        if !files.source_only.is_empty() {
            println!(
                "Files only in source ({}): {}",
                files.source_only.len(),
                files
                    .source_only
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if files.source_only.len() > 5 {
                println!("  ... and {} more", files.source_only.len() - 5);
            }
            println!();
        }

        if !files.target_only.is_empty() {
            println!(
                "Files only in target ({}): {}",
                files.target_only.len(),
                files
                    .target_only
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if files.target_only.len() > 5 {
                println!("  ... and {} more", files.target_only.len() - 5);
            }
            println!();
        }

        if !files.size_differences.is_empty() {
            println!(
                "Files with size differences: {}",
                files.size_differences.len()
            );
        }

        if !files.content_differences.is_empty() {
            println!(
                "Files with content differences: {}",
                files.content_differences.len()
            );
        }

        if !files.metadata_differences.is_empty() {
            println!(
                "Files with metadata differences: {}",
                files.metadata_differences.len()
            );
        }
    }

    println!();
    println!(
        "Summary: {} total differences",
        result.summary.different_files
            + result.summary.source_only_count
            + result.summary.target_only_count
    );

    Ok(())
}

fn display_table_output(
    source_path: &str,
    target_path: &str,
    result: &wow_mpq::ComparisonResult,
) -> Result<()> {
    println!(
        "Archive Comparison: {} vs {}",
        truncate_path(source_path, 30),
        truncate_path(target_path, 30)
    );
    println!("{}", "=".repeat(80));

    if result.identical {
        println!("✓ Archives are identical");
        return Ok(());
    }

    // Metadata table
    let mut metadata_table = create_table(vec!["Property", "Source", "Target", "Match"]);
    add_table_row(
        &mut metadata_table,
        vec![
            "Format Version".to_string(),
            format!("{:?}", result.metadata.format_version.0),
            format!("{:?}", result.metadata.format_version.1),
            if result.metadata.format_version.0 == result.metadata.format_version.1 {
                "✓"
            } else {
                "✗"
            }
            .to_string(),
        ],
    );
    add_table_row(
        &mut metadata_table,
        vec![
            "Block Size".to_string(),
            result.metadata.block_size.0.to_string(),
            result.metadata.block_size.1.to_string(),
            if result.metadata.block_size.0 == result.metadata.block_size.1 {
                "✓"
            } else {
                "✗"
            }
            .to_string(),
        ],
    );
    add_table_row(
        &mut metadata_table,
        vec![
            "File Count".to_string(),
            result.metadata.file_count.0.to_string(),
            result.metadata.file_count.1.to_string(),
            if result.metadata.file_count.0 == result.metadata.file_count.1 {
                "✓"
            } else {
                "✗"
            }
            .to_string(),
        ],
    );
    add_table_row(
        &mut metadata_table,
        vec![
            "Archive Size".to_string(),
            format_bytes(result.metadata.archive_size.0),
            format_bytes(result.metadata.archive_size.1),
            if result.metadata.archive_size.0 == result.metadata.archive_size.1 {
                "✓"
            } else {
                "✗"
            }
            .to_string(),
        ],
    );

    println!("\nMetadata Comparison:");
    metadata_table.printstd();

    // File differences
    if let Some(files) = &result.files {
        if !files.source_only.is_empty()
            || !files.target_only.is_empty()
            || !files.size_differences.is_empty()
        {
            println!("\nFile Differences:");

            if !files.source_only.is_empty() {
                println!("\nFiles only in source ({}):", files.source_only.len());
                for file in files.source_only.iter().take(10) {
                    println!("  - {file}");
                }
                if files.source_only.len() > 10 {
                    println!("  ... and {} more", files.source_only.len() - 10);
                }
            }

            if !files.target_only.is_empty() {
                println!("\nFiles only in target ({}):", files.target_only.len());
                for file in files.target_only.iter().take(10) {
                    println!("  + {file}");
                }
                if files.target_only.len() > 10 {
                    println!("  ... and {} more", files.target_only.len() - 10);
                }
            }

            if !files.size_differences.is_empty() {
                println!("\nFiles with size differences:");
                let mut size_table =
                    create_table(vec!["File", "Source Size", "Target Size", "Compression"]);

                for diff in files.size_differences.iter().take(10) {
                    add_table_row(
                        &mut size_table,
                        vec![
                            truncate_path(&diff.name, 40),
                            format_bytes(diff.source_size),
                            format_bytes(diff.target_size),
                            format!(
                                "{} → {}",
                                format_compression_ratio(diff.source_size, diff.source_compressed),
                                format_compression_ratio(diff.target_size, diff.target_compressed)
                            ),
                        ],
                    );
                }

                size_table.printstd();

                if files.size_differences.len() > 10 {
                    println!(
                        "... and {} more files with size differences",
                        files.size_differences.len() - 10
                    );
                }
            }

            if !files.content_differences.is_empty() {
                println!(
                    "\nFiles with content differences ({}):",
                    files.content_differences.len()
                );
                for file in files.content_differences.iter().take(10) {
                    println!("  ≠ {file}");
                }
                if files.content_differences.len() > 10 {
                    println!("  ... and {} more", files.content_differences.len() - 10);
                }
            }
        }
    }

    // Summary
    println!("\nSummary:");
    println!("  {} files identical", result.summary.identical_files);
    println!("  {} files different", result.summary.different_files);
    println!(
        "  {} files only in source",
        result.summary.source_only_count
    );
    println!(
        "  {} files only in target",
        result.summary.target_only_count
    );

    Ok(())
}

fn display_json_output(result: &wow_mpq::ComparisonResult) -> Result<()> {
    // For now, just pretty-print the debug representation
    // In a real implementation, you'd use serde_json
    println!("{result:#?}");
    Ok(())
}

fn show_tree(
    path: &str,
    max_depth: Option<usize>,
    show_external_refs: bool,
    no_color: bool,
    compact: bool,
    filter: Option<String>,
) -> Result<()> {
    let spinner = create_spinner("Analyzing archive structure...");
    let mut archive = Archive::open(path).context("Failed to open archive")?;
    let info = archive.get_info()?;
    spinner.finish_and_clear();

    // Create root node with archive information
    let mut root = TreeNode::new(
        format!(
            "{}",
            std::path::Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
        ),
        NodeType::Root,
    )
    .with_size(info.file_size)
    .with_metadata("format", &format!("{:?}", info.format_version))
    .with_metadata("files", &info.file_count.to_string());

    // Add header information
    let header = TreeNode::new("Header".to_string(), NodeType::Header)
        .with_size(32) // Typical MPQ header size
        .with_metadata("version", &format!("{:?}", info.format_version))
        .with_metadata("sector_size", &format!("{}", info.sector_size));

    root = root.add_child(header);

    // Add hash table information
    if let Some(hash_table) = archive.hash_table() {
        let hash_node = TreeNode::new("Hash Table".to_string(), NodeType::Table)
            .with_size((hash_table.size() * 16) as u64) // Each hash entry is 16 bytes
            .with_metadata("entries", &hash_table.size().to_string())
            .with_metadata("encrypted", "true");
        root = root.add_child(hash_node);
    }

    // Add block table information
    if let Some(block_table) = archive.block_table() {
        let block_node = TreeNode::new("Block Table".to_string(), NodeType::Table)
            .with_size((block_table.size() * 16) as u64) // Each block entry is 16 bytes
            .with_metadata("entries", &block_table.size().to_string())
            .with_metadata("encrypted", "true");
        root = root.add_child(block_node);
    }

    // Add HET/BET tables if present
    if archive.het_table().is_some() {
        let het_node = TreeNode::new("HET Table".to_string(), NodeType::Table)
            .with_metadata("type", "Extended Hash Table")
            .with_metadata("version", "v4");
        root = root.add_child(het_node);
    }

    if archive.bet_table().is_some() {
        let bet_node = TreeNode::new("BET Table".to_string(), NodeType::Table)
            .with_metadata("type", "Extended Block Table")
            .with_metadata("version", "v4");
        root = root.add_child(bet_node);
    }

    // Build file tree
    let files: Vec<String> = archive.list()?.into_iter().map(|e| e.name).collect();
    let pattern = filter.as_deref().unwrap_or("*");
    let filtered_files: Vec<_> = files
        .iter()
        .filter(|f| matches_pattern(f, pattern))
        .collect();

    if !filtered_files.is_empty() {
        let mut files_node = TreeNode::new("Files".to_string(), NodeType::Directory)
            .with_metadata("count", &filtered_files.len().to_string());

        // Build directory structure
        let mut dir_structure = std::collections::BTreeMap::<String, Vec<&String>>::new();

        for file in &filtered_files {
            let path_parts: Vec<&str> = file.split('\\').collect();
            if path_parts.len() > 1 {
                let dir = path_parts[..path_parts.len() - 1].join("\\");
                dir_structure.entry(dir).or_default().push(file);
            } else {
                dir_structure.entry("/".to_string()).or_default().push(file);
            }
        }

        // Add directories and files to tree
        for (dir_path, dir_files) in dir_structure {
            if dir_path == "/" {
                // Root level files
                for file in dir_files {
                    let file_node = create_file_node(file, &mut archive, show_external_refs)?;
                    files_node = files_node.add_child(file_node);
                }
            } else {
                // Directory with files
                let mut dir_node = TreeNode::new(
                    format!("{}/", dir_path.split('\\').next_back().unwrap_or(&dir_path)),
                    NodeType::Directory,
                )
                .with_metadata("files", &dir_files.len().to_string());

                for file in dir_files {
                    let file_node = create_file_node(file, &mut archive, show_external_refs)?;
                    dir_node = dir_node.add_child(file_node);
                }

                files_node = files_node.add_child(dir_node);
            }
        }

        root = root.add_child(files_node);
    }

    // Add special files
    let special_files = vec!["(listfile)", "(attributes)", "(signature)"];
    for special_file in special_files {
        if archive.read_file(special_file).is_ok() {
            let special_node = match special_file {
                "(listfile)" => TreeNode::new("(listfile)".to_string(), NodeType::File)
                    .with_metadata("type", "Auto-generated file list")
                    .with_metadata("purpose", "File enumeration"),
                "(attributes)" => TreeNode::new("(attributes)".to_string(), NodeType::File)
                    .with_metadata("type", "File attributes")
                    .with_metadata("purpose", "CRC checksums and timestamps"),
                "(signature)" => TreeNode::new("(signature)".to_string(), NodeType::File)
                    .with_metadata("type", "Digital signature")
                    .with_metadata("purpose", "Archive integrity verification"),
                _ => continue,
            };
            root = root.add_child(special_node);
        }
    }

    // Render the tree
    let options = TreeOptions {
        max_depth,
        show_external_refs,
        no_color,
        show_metadata: true,
        compact,
        verbose: false,
    };

    println!("{}", render_tree(&root, &options));
    Ok(())
}

fn create_file_node(
    file_path: &str,
    archive: &mut Archive,
    show_external_refs: bool,
) -> Result<TreeNode> {
    let file_name = file_path.split('\\').next_back().unwrap_or(file_path);
    let extension = std::path::Path::new(file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let mut node = TreeNode::new(file_name.to_string(), NodeType::File);

    // Add file size if available
    if let Ok(entries) = archive.list() {
        if let Some(entry) = entries.iter().find(|e| e.name == file_path) {
            node = node
                .with_size(entry.size)
                .with_metadata("compressed_size", &format_bytes(entry.compressed_size))
                .with_metadata(
                    "compression_ratio",
                    &format_compression_ratio(entry.size, entry.compressed_size),
                );
        }
    }

    // Add file type metadata
    let file_type = match extension.to_lowercase().as_str() {
        "blp" => "Texture",
        "m2" | "mdx" => "Model",
        "wdt" => "World Map Definition",
        "adt" => "Terrain Data",
        "wdl" => "Low-res Terrain",
        "dbc" => "Database",
        "lua" => "Script",
        "xml" => "Interface Definition",
        "toc" => "AddOn Manifest",
        "wav" | "mp3" => "Audio",
        _ => "Data",
    };
    node = node.with_metadata("type", file_type);

    // Add external references for certain file types
    if show_external_refs {
        match extension.to_lowercase().as_str() {
            "wdt" => {
                // WDT files reference ADT files
                let base_name = file_name.trim_end_matches(".wdt");
                node = node
                    .with_external_ref(&format!("{base_name}/*.adt"), detect_ref_type("file.adt"));
            }
            "adt" => {
                // ADT files might reference textures and models
                node = node.with_external_ref("*.blp", detect_ref_type("file.blp"));
                node = node.with_external_ref("*.m2", detect_ref_type("file.m2"));
            }
            "m2" => {
                // M2 files reference textures and animations
                let base_name = file_name.trim_end_matches(".m2");
                node = node
                    .with_external_ref(&format!("{base_name}.skin"), detect_ref_type("file.skin"));
                node = node.with_external_ref("*.blp", detect_ref_type("file.blp"));
            }
            "dbc" => {
                // Some DBC files reference other files
                if file_name.to_lowercase().contains("item") {
                    node = node
                        .with_external_ref("Interface/Icons/*.blp", detect_ref_type("file.blp"));
                }
            }
            _ => {}
        }
    }

    Ok(node)
}

fn debug_archive(params: DebugParams<'_>) -> Result<()> {
    let spinner = create_spinner("Opening archive...");
    let mut archive = Archive::open(params.archive_path).context("Failed to open archive")?;
    spinner.finish_and_clear();

    println!("🔍 MPQ Debug Information");
    println!("========================");
    println!("Archive: {}", params.archive_path);

    let info = archive.get_info()?;
    println!("Format: {:?}", info.format_version);
    println!("Files: {}/{}", info.file_count, info.max_file_count);
    println!();

    // Handle finding a specific file first
    if let Some(filename) = &params.find_file {
        return find_file_entries(&mut archive, filename, params.raw_dump);
    }

    // Handle specific entry index
    if let Some(index) = params.entry_index {
        return show_entry_at_index(&mut archive, index, params.raw_dump);
    }

    // Show requested tables
    if params.show_hash_table {
        show_hash_table(&mut archive, params.raw_dump)?;
    }

    if params.show_block_table {
        show_block_table(&mut archive, params.raw_dump)?;
    }

    if params.show_het_table {
        show_het_table(&mut archive, params.raw_dump)?;
    }

    if params.show_bet_table {
        show_bet_table(&mut archive, params.raw_dump)?;
    }

    Ok(())
}

fn show_hash_table(archive: &mut Archive, raw_dump: bool) -> Result<()> {
    println!("🔑 Hash Table");
    println!("-------------");

    if let Some(hash_table) = archive.hash_table() {
        if raw_dump {
            // Show raw hex dump of the hash table
            let entries = hash_table.entries();
            let data_size = std::mem::size_of_val(entries);
            println!("Raw data ({data_size} bytes):");

            // Convert entries to bytes for hex dump
            let bytes =
                unsafe { std::slice::from_raw_parts(entries.as_ptr() as *const u8, data_size) };

            let config = HexDumpConfig {
                bytes_per_line: 16,
                show_ascii: false,
                show_offset: true,
                max_bytes: 512,
            };
            println!("{}", hex_dump(bytes, &config));
        } else {
            // Use the formatted table display
            println!("{}", format_hash_table(hash_table.entries()));
        }
    } else {
        println!("No hash table found (archive may use HET/BET tables)");
    }
    println!();

    Ok(())
}

fn show_block_table(archive: &mut Archive, raw_dump: bool) -> Result<()> {
    println!("📦 Block Table");
    println!("--------------");

    if let Some(block_table) = archive.block_table() {
        if raw_dump {
            // Show raw hex dump of the block table
            let entries = block_table.entries();
            let data_size = std::mem::size_of_val(entries);
            println!("Raw data ({data_size} bytes):");

            // Convert entries to bytes for hex dump
            let bytes =
                unsafe { std::slice::from_raw_parts(entries.as_ptr() as *const u8, data_size) };

            let config = HexDumpConfig {
                bytes_per_line: 16,
                show_ascii: false,
                show_offset: true,
                max_bytes: 512,
            };
            println!("{}", hex_dump(bytes, &config));
        } else {
            // Use the formatted table display
            println!("{}", format_block_table(block_table.entries()));
        }
    } else {
        println!("No block table found (archive may use HET/BET tables)");
    }
    println!();

    Ok(())
}

fn show_het_table(archive: &mut Archive, raw_dump: bool) -> Result<()> {
    println!("🔍 HET Table (Extended Hash Table)");
    println!("----------------------------------");

    if let Some(het_table) = archive.het_table() {
        if raw_dump {
            println!("Raw HET data not yet implemented");
        } else {
            // Use the formatted display
            println!("{}", format_het_table(het_table));
        }
    } else {
        println!("No HET table found");
    }
    println!();

    Ok(())
}

fn show_bet_table(archive: &mut Archive, raw_dump: bool) -> Result<()> {
    println!("📋 BET Table (Extended Block Table)");
    println!("-----------------------------------");

    if let Some(bet_table) = archive.bet_table() {
        if raw_dump {
            println!("Raw BET data not yet implemented");
        } else {
            // Use the formatted display
            println!("{}", format_bet_table(bet_table));
        }
    } else {
        println!("No BET table found");
    }
    println!();

    Ok(())
}

fn find_file_entries(archive: &mut Archive, filename: &str, _raw_dump: bool) -> Result<()> {
    println!("🔎 Finding entries for: {filename}");
    println!("========================");

    // Try to find the file using the archive's find_file method
    match archive.find_file(filename)? {
        Some(file_info) => {
            println!("✓ File found!");
            println!("  Hash table index: {}", file_info.hash_index);
            println!("  Block table index: {}", file_info.block_index);
            println!("  File position: 0x{:08X}", file_info.file_pos);
            println!("  File size: {}", format_bytes(file_info.file_size));
            println!(
                "  Compressed size: {}",
                format_bytes(file_info.compressed_size)
            );
            println!("  Flags: 0x{:08X}", file_info.flags);
            println!("  Locale: 0x{:04X}", file_info.locale);
            println!();

            // Show hash entry
            if let Some(hash_table) = archive.hash_table() {
                if let Some(hash_entry) = hash_table.entries().get(file_info.hash_index) {
                    println!("Hash Entry:");
                    println!("{}", dump_hash_entry(hash_entry, file_info.hash_index));
                }
            }

            // Show block entry
            if let Some(block_table) = archive.block_table() {
                if let Some(block_entry) = block_table.entries().get(file_info.block_index) {
                    println!("\nBlock Entry:");
                    println!("{}", dump_block_entry(block_entry, file_info.block_index));
                }
            }
        }
        None => {
            println!("✗ File not found in archive");
        }
    }

    Ok(())
}

fn show_entry_at_index(archive: &mut Archive, index: usize, _raw_dump: bool) -> Result<()> {
    println!("📍 Entry at index: {index}");
    println!("===================");

    let mut found = false;

    // Check hash table
    if let Some(hash_table) = archive.hash_table() {
        if let Some(hash_entry) = hash_table.entries().get(index) {
            println!("Hash Entry:");
            println!("{}", dump_hash_entry(hash_entry, index));
            found = true;
        }
    }

    // Check block table
    if let Some(block_table) = archive.block_table() {
        if let Some(block_entry) = block_table.entries().get(index) {
            if found {
                println!();
            }
            println!("Block Entry:");
            println!("{}", dump_block_entry(block_entry, index));
            found = true;
        }
    }

    if !found {
        println!("No entry found at index {index}");
    }

    Ok(())
}

// Database command implementation
fn execute_db_command(command: DbCommands) -> Result<()> {
    use std::io::Write;
    use wow_mpq::database::{Database, HashLookup, ImportSource, Importer};
    use wow_mpq::database::{calculate_het_hashes, calculate_mpq_hashes};

    match command {
        DbCommands::Status { detailed } => {
            let db = Database::open_default().context("Failed to open database")?;
            let conn = db.connection();

            // Get basic statistics
            let filename_count: i64 =
                conn.query_row("SELECT COUNT(*) FROM filenames", [], |row| row.get(0))?;

            println!("MPQ Hash Database Status");
            println!("========================");
            println!("Database location: {}", db.path().display());
            println!("Total filenames: {filename_count}");

            if detailed {
                println!("\nDetailed Statistics:");

                // Count by source
                let mut stmt = conn.prepare(
                    "SELECT source, COUNT(*) FROM filenames GROUP BY source ORDER BY COUNT(*) DESC",
                )?;
                let sources = stmt.query_map([], |row| {
                    Ok((row.get::<_, Option<String>>(0)?, row.get::<_, i64>(1)?))
                })?;

                println!("\nFilenames by source:");
                for source in sources {
                    let (src, count) = source?;
                    println!(
                        "  {}: {}",
                        src.unwrap_or_else(|| "unknown".to_string()),
                        count
                    );
                }

                // Recent additions
                let mut stmt = conn.prepare(
                    "SELECT filename, created_at FROM filenames ORDER BY created_at DESC LIMIT 10",
                )?;
                let recent = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;

                println!("\nMost recent additions:");
                for entry in recent {
                    let (filename, created_at) = entry?;
                    println!("  {filename} - {created_at}");
                }
            }

            Ok(())
        }

        DbCommands::Import {
            path,
            source_type,
            show_progress,
        } => {
            let db = Database::open_default().context("Failed to open database")?;
            let importer = Importer::new(&db);

            let import_source = match source_type {
                ImportSourceArg::Listfile => ImportSource::Listfile,
                ImportSourceArg::Archive => ImportSource::Archive,
                ImportSourceArg::Directory => ImportSource::Directory,
            };

            let spinner = if show_progress {
                Some(create_spinner(&format!("Importing from {path}...")))
            } else {
                None
            };

            let stats = importer
                .import(Path::new(&path), import_source)
                .context("Import failed")?;

            if let Some(s) = spinner {
                s.finish_and_clear();
            }

            println!("Import completed:");
            println!("  Files processed: {}", stats.files_processed);
            println!("  New entries added: {}", stats.new_entries);
            println!("  Existing entries updated: {}", stats.updated_entries);
            if stats.errors > 0 {
                println!("  Errors: {}", stats.errors);
            }

            Ok(())
        }

        DbCommands::Analyze {
            archive,
            include_anonymous,
        } => {
            let db = Database::open_default().context("Failed to open database")?;
            let mut mpq = Archive::open(&archive).context("Failed to open archive")?;

            let spinner = create_spinner("Analyzing archive...");

            // Record listfile entries
            let count = mpq.record_listfile_to_db(&db)?;

            spinner.finish_with_message(format!("Recorded {count} filenames from listfile"));

            if include_anonymous {
                // TODO: Could also record anonymous entries with generated names
                println!("Note: Recording anonymous entries not yet implemented");
            }

            Ok(())
        }

        DbCommands::Lookup { filename } => {
            let db = Database::open_default().context("Failed to open database")?;

            // Calculate and display hashes
            let (hash_a, hash_b, hash_offset) = calculate_mpq_hashes(&filename);
            let het_40 = calculate_het_hashes(&filename, 40);
            let het_48 = calculate_het_hashes(&filename, 48);
            let het_56 = calculate_het_hashes(&filename, 56);
            let het_64 = calculate_het_hashes(&filename, 64);

            println!("Filename: {filename}");
            println!("\nTraditional MPQ hashes:");
            println!("  Hash A (Name1):  0x{hash_a:08X}");
            println!("  Hash B (Name2):  0x{hash_b:08X}");
            println!("  Table Offset:    0x{hash_offset:08X}");

            println!("\nHET hashes:");
            println!(
                "  40-bit: file=0x{:010X}, name=0x{:010X}",
                het_40.0, het_40.1
            );
            println!(
                "  48-bit: file=0x{:012X}, name=0x{:012X}",
                het_48.0, het_48.1
            );
            println!(
                "  56-bit: file=0x{:014X}, name=0x{:014X}",
                het_56.0, het_56.1
            );
            println!(
                "  64-bit: file=0x{:016X}, name=0x{:016X}",
                het_64.0, het_64.1
            );

            // Check if it exists in database
            if db.filename_exists(&filename)? {
                println!("\n✓ Filename exists in database");
            } else {
                println!("\n✗ Filename not found in database");
            }

            Ok(())
        }

        DbCommands::Export { output, source } => {
            let db = Database::open_default().context("Failed to open database")?;
            let conn = db.connection();

            let mut query = "SELECT DISTINCT filename FROM filenames".to_string();
            let mut params: Vec<String> = Vec::new();

            if let Some(src) = source {
                query.push_str(" WHERE source = ?1");
                params.push(src);
            }

            query.push_str(" ORDER BY filename");

            let mut stmt = conn.prepare(&query)?;
            let filenames: Vec<String> = if params.is_empty() {
                stmt.query_map([], |row| row.get::<_, String>(0))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            } else {
                stmt.query_map([&params[0]], |row| row.get::<_, String>(0))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            };

            let mut file = fs::File::create(&output).context("Failed to create output file")?;
            let mut count = 0;

            for filename in filenames {
                writeln!(file, "{filename}")?;
                count += 1;
            }

            println!("Exported {count} filenames to {output}");

            Ok(())
        }

        DbCommands::List {
            filter,
            long,
            limit,
        } => {
            let db = Database::open_default().context("Failed to open database")?;
            let conn = db.connection();

            let mut query = "SELECT filename, hash_a, hash_b, source FROM filenames".to_string();
            let mut params: Vec<String> = Vec::new();

            if let Some(pattern) = filter {
                query.push_str(" WHERE filename LIKE ?1");
                params.push(pattern.replace('*', "%"));
            }

            query.push_str(&format!(" ORDER BY filename LIMIT {limit}"));

            let mut stmt = conn.prepare(&query)?;
            let rows: Vec<(String, u32, u32, Option<String>)> = if params.is_empty() {
                stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, u32>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?
            } else {
                stmt.query_map([&params[0]], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, u32>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?
            };

            if long {
                let mut table = create_table(vec!["Filename", "Hash A", "Hash B", "Source"]);
                for (filename, hash_a, hash_b, source) in rows {
                    add_table_row(
                        &mut table,
                        vec![
                            filename,
                            format!("0x{:08X}", hash_a),
                            format!("0x{:08X}", hash_b),
                            source.unwrap_or_else(|| "unknown".to_string()),
                        ],
                    );
                }
                println!("{table}");
            } else {
                for (filename, _, _, _) in rows {
                    println!("{filename}");
                }
            }

            Ok(())
        }
    }
}
