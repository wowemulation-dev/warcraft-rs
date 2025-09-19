//! Archive builder for creating MPQ archives

use crate::{
    Error, Result,
    compression::{compress, flags as compression_flags},
    crypto::{encrypt_block, hash_string, hash_type, het_hash, jenkins_hash},
    header::{FormatVersion, MpqHeaderV4Data},
    special_files::{AttributeFlags, Attributes, FileAttributes},
    tables::{BetHeader, BlockEntry, BlockTable, HashEntry, HashTable, HetHeader, HiBlockTable},
};
use md5::{Digest, Md5};
use std::fs::{self};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// Helper trait for writing little-endian integers
trait WriteLittleEndian: Write {
    fn write_u16_le(&mut self, value: u16) -> Result<()> {
        self.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_u32_le(&mut self, value: u32) -> Result<()> {
        self.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    fn write_u64_le(&mut self, value: u64) -> Result<()> {
        self.write_all(&value.to_le_bytes())?;
        Ok(())
    }
}

impl<W: Write> WriteLittleEndian for W {}

/// File to be added to the archive
#[derive(Debug)]
struct PendingFile {
    /// Source path or data
    source: FileSource,
    /// Target filename in archive
    archive_name: String,
    /// Compression method to use
    compression: u8,
    /// Whether to encrypt the file
    encrypt: bool,
    /// Whether to use FIX_KEY encryption (adjusts key by block position)
    use_fix_key: bool,
    /// Locale code
    locale: u16,
}

#[derive(Debug)]
enum FileSource {
    Path(PathBuf),
    Data(Vec<u8>),
}

/// Parameters for writing a file to the archive
struct FileWriteParams<'a> {
    /// File data to write
    file_data: &'a [u8],
    /// Archive name for the file
    archive_name: &'a str,
    /// Compression method
    compression: u8,
    /// Whether to encrypt
    encrypt: bool,
    /// Whether to use FIX_KEY encryption
    use_fix_key: bool,
    /// Sector size
    sector_size: usize,
    /// File position in archive (64-bit for large archives)
    file_pos: u64,
}

/// Parameters for writing the MPQ header
struct HeaderWriteParams {
    archive_size: u64,
    hash_table_pos: u64,
    block_table_pos: u64,
    hash_table_size: u32,
    block_table_size: u32,
    hi_block_table_pos: Option<u64>,
    het_table_pos: Option<u64>,
    bet_table_pos: Option<u64>,
    _het_table_size: Option<u64>,
    _bet_table_size: Option<u64>,
    // V4 specific fields
    v4_data: Option<MpqHeaderV4Data>,
}

/// Options for listfile generation
#[derive(Debug, Clone)]
pub enum ListfileOption {
    /// Automatically generate listfile from added files
    Generate,
    /// Use external listfile
    External(PathBuf),
    /// Don't include a listfile
    None,
}

/// Options for attributes file generation
#[derive(Debug, Clone)]
pub enum AttributesOption {
    /// Generate attributes with CRC32 checksums
    GenerateCrc32,
    /// Generate attributes with CRC32 and MD5
    GenerateFull,
    /// Use external attributes file
    External(PathBuf),
    /// Don't include attributes file
    None,
}

/// Builder for creating new MPQ archives
///
/// `ArchiveBuilder` provides a fluent interface for creating MPQ archives with
/// complete control over format version, compression, encryption, and file organization.
///
/// # Examples
///
/// ## Basic archive creation
///
/// ```no_run
/// use wow_mpq::{ArchiveBuilder, FormatVersion};
///
/// // Create a simple archive with default settings
/// ArchiveBuilder::new()
///     .add_file("readme.txt", "README.txt")
///     .add_file_data(b"Hello world".to_vec(), "hello.txt")
///     .build("my_archive.mpq")?;
/// # Ok::<(), wow_mpq::Error>(())
/// ```
///
/// ## Advanced archive creation
///
/// ```no_run
/// use wow_mpq::{ArchiveBuilder, FormatVersion, compression, ListfileOption};
///
/// ArchiveBuilder::new()
///     .version(FormatVersion::V2)
///     .block_size(7)  // 64KB sectors for better performance
///     .default_compression(compression::flags::BZIP2)
///     .listfile_option(ListfileOption::Generate)
///     .generate_crcs(true)
///     .add_file_data_with_options(
///         b"secret data".to_vec(),
///         "encrypted.dat",
///         compression::flags::ZLIB,
///         true,  // encrypt
///         0,     // locale
///     )
///     .build("advanced.mpq")?;
/// # Ok::<(), wow_mpq::Error>(())
/// ```
#[derive(Debug)]
pub struct ArchiveBuilder {
    /// Target MPQ version
    version: FormatVersion,
    /// Block size (sector size = 512 * 2^block_size)
    block_size: u16,
    /// Files to be added
    pending_files: Vec<PendingFile>,
    /// Listfile option
    listfile_option: ListfileOption,
    /// Attributes option
    attributes_option: AttributesOption,
    /// Default compression method
    default_compression: u8,
    /// Whether to generate sector CRCs for files
    generate_crcs: bool,
    /// Whether to compress HET/BET tables (v3+ only)
    compress_tables: bool,
    /// Compression method for tables
    table_compression: u8,
}

impl ArchiveBuilder {
    /// Create a new archive builder
    pub fn new() -> Self {
        Self {
            version: FormatVersion::V1,
            block_size: 5, // Default 16KB sectors for StormLib compatibility
            pending_files: Vec::new(),
            listfile_option: ListfileOption::Generate,
            attributes_option: AttributesOption::None,
            default_compression: compression_flags::ZLIB,
            generate_crcs: false,
            compress_tables: false, // Default to uncompressed for compatibility
            table_compression: compression_flags::ZLIB,
        }
    }

    /// Set the MPQ format version
    pub fn version(mut self, version: FormatVersion) -> Self {
        self.version = version;
        self
    }

    /// Set the block size (sector size = 512 * 2^block_size)
    ///
    /// The block size determines the sector size used for file storage.
    /// Larger block sizes can improve compression efficiency for large files
    /// but increase overhead for small files.
    ///
    /// # Parameters
    /// - `block_size`: Power of 2 exponent (0-31). Final sector size = 512 * 2^block_size
    ///   - Common values: 3 (4KB sectors), 4 (8KB), 5 (16KB), 6 (32KB), 7 (64KB)
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::ArchiveBuilder;
    ///
    /// // Create archive with 64KB sectors (good for large files)
    /// let builder = ArchiveBuilder::new().block_size(7);
    ///
    /// // Create archive with 4KB sectors (good for small files)
    /// let builder = ArchiveBuilder::new().block_size(3);
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    pub fn block_size(mut self, block_size: u16) -> Self {
        self.block_size = block_size;
        self
    }

    /// Set the default compression method
    pub fn default_compression(mut self, compression: u8) -> Self {
        self.default_compression = compression;
        self
    }

    /// Set the listfile option
    pub fn listfile_option(mut self, option: ListfileOption) -> Self {
        self.listfile_option = option;
        self
    }

    /// Enable or disable sector CRC generation
    ///
    /// When enabled, CRC32 checksums are generated for each sector of each file,
    /// providing integrity verification during file extraction. This adds security
    /// but increases archive size and creation time.
    ///
    /// # Parameters
    /// - `generate`: If `true`, sector CRCs are generated. If `false`, no CRCs.
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::ArchiveBuilder;
    ///
    /// // Enable CRC generation for data integrity
    /// let builder = ArchiveBuilder::new().generate_crcs(true);
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    ///
    /// # Notes
    /// CRC generation is recommended for archives containing critical data
    /// where integrity verification is important.
    pub fn generate_crcs(mut self, generate: bool) -> Self {
        self.generate_crcs = generate;
        // Also enable attributes if CRCs are requested
        if generate && matches!(self.attributes_option, AttributesOption::None) {
            self.attributes_option = AttributesOption::GenerateCrc32;
        }
        self
    }

    /// Set the attributes file option
    ///
    /// Controls how the (attributes) special file is generated, which stores
    /// metadata like CRC32 checksums, MD5 hashes, and file timestamps.
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::{ArchiveBuilder, AttributesOption};
    ///
    /// // Generate attributes with CRC32 checksums
    /// let builder = ArchiveBuilder::new()
    ///     .attributes_option(AttributesOption::GenerateCrc32);
    ///
    /// // Generate full attributes with CRC32 and MD5
    /// let builder = ArchiveBuilder::new()
    ///     .attributes_option(AttributesOption::GenerateFull);
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    pub fn attributes_option(mut self, option: AttributesOption) -> Self {
        // Enable CRC generation if attributes are requested
        if matches!(
            option,
            AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull
        ) {
            self.generate_crcs = true;
        }
        self.attributes_option = option;
        self
    }

    /// Enable or disable HET/BET table compression (v3+ only)
    ///
    /// For MPQ format version 3 and 4, the HET (Hash Extended Table) and BET
    /// (Block Extended Table) can be compressed to reduce archive size. This
    /// only applies to v3+ archives; v1/v2 archives ignore this setting.
    ///
    /// # Parameters
    /// - `compress`: If `true`, HET/BET tables are compressed. If `false`, stored uncompressed.
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::{ArchiveBuilder, FormatVersion};
    ///
    /// // Enable table compression for v3 archive
    /// let builder = ArchiveBuilder::new()
    ///     .version(FormatVersion::V3)
    ///     .compress_tables(true);
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    ///
    /// # Notes
    /// Table compression can significantly reduce archive size for large archives
    /// with many files, but may slightly increase archive opening time.
    pub fn compress_tables(mut self, compress: bool) -> Self {
        self.compress_tables = compress;
        self
    }

    /// Set compression method for tables (default: zlib)
    ///
    /// Specifies which compression algorithm to use when compressing HET/BET tables
    /// in v3+ archives. Only used when `compress_tables` is enabled.
    ///
    /// # Parameters
    /// - `compression`: Compression method flag from `compression::flags`
    ///   - `compression::flags::ZLIB` (default): Fast and widely compatible
    ///   - `compression::flags::BZIP2`: Better compression ratio but slower
    ///   - `compression::flags::LZMA`: Best compression but slowest
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::{ArchiveBuilder, FormatVersion, compression};
    ///
    /// // Use BZIP2 for table compression
    /// let builder = ArchiveBuilder::new()
    ///     .version(FormatVersion::V3)
    ///     .compress_tables(true)
    ///     .table_compression(compression::flags::BZIP2);
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    pub fn table_compression(mut self, compression: u8) -> Self {
        self.table_compression = compression;
        self
    }

    /// Add a file from disk to the archive
    ///
    /// Reads a file from the filesystem and adds it to the archive with default
    /// compression and no encryption. The file will use the builder's default
    /// compression method and neutral locale.
    ///
    /// # Parameters
    /// - `path`: Path to the source file on disk
    /// - `archive_name`: Name the file will have inside the archive
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::ArchiveBuilder;
    ///
    /// let builder = ArchiveBuilder::new()
    ///     .add_file("data/config.txt", "config.txt")
    ///     .add_file("assets/image.jpg", "images/image.jpg");
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    ///
    /// # Notes
    /// - The source file is read when `build()` is called, not when `add_file()` is called
    /// - Archive names are automatically normalized to use backslashes as path separators
    /// - Use `add_file_with_options()` for custom compression or encryption settings
    pub fn add_file<P: AsRef<Path>>(mut self, path: P, archive_name: &str) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Path(path.as_ref().to_path_buf()),
            archive_name: crate::path::normalize_mpq_path(archive_name),
            compression: self.default_compression,
            encrypt: false,
            use_fix_key: false,
            locale: 0, // Neutral locale
        });
        self
    }

    /// Add a file from disk with custom compression and encryption options
    ///
    /// Provides full control over how the file is stored in the archive,
    /// including compression method, encryption, and locale settings.
    ///
    /// # Parameters
    /// - `path`: Path to the source file on disk
    /// - `archive_name`: Name the file will have inside the archive
    /// - `compression`: Compression method from `compression::flags` (0 = no compression)
    /// - `encrypt`: Whether to encrypt the file
    /// - `locale`: Locale code for the file (0 = neutral locale)
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::{ArchiveBuilder, compression};
    ///
    /// let builder = ArchiveBuilder::new()
    ///     .add_file_with_options(
    ///         "secret.txt",
    ///         "hidden/secret.txt",
    ///         compression::flags::BZIP2,
    ///         true,  // encrypt
    ///         0      // neutral locale
    ///     );
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    pub fn add_file_with_options<P: AsRef<Path>>(
        mut self,
        path: P,
        archive_name: &str,
        compression: u8,
        encrypt: bool,
        locale: u16,
    ) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Path(path.as_ref().to_path_buf()),
            archive_name: crate::path::normalize_mpq_path(archive_name),
            compression,
            encrypt,
            use_fix_key: false,
            locale,
        });
        self
    }

    /// Add a file from in-memory data
    ///
    /// Creates a file in the archive from data already loaded in memory.
    /// Useful for dynamically generated content or when you already have
    /// the file data loaded.
    ///
    /// # Parameters
    /// - `data`: Raw file data to store in the archive
    /// - `archive_name`: Name the file will have inside the archive
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::ArchiveBuilder;
    ///
    /// let config_data = b"version=1.0\ndebug=false".to_vec();
    /// let builder = ArchiveBuilder::new()
    ///     .add_file_data(config_data, "config.ini")
    ///     .add_file_data(b"Hello, World!".to_vec(), "readme.txt");
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    ///
    /// # Notes
    /// - Uses the builder's default compression method and neutral locale
    /// - More memory efficient than `add_file()` when data is already in memory
    /// - Use `add_file_data_with_options()` for custom compression or encryption
    pub fn add_file_data(mut self, data: Vec<u8>, archive_name: &str) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Data(data),
            archive_name: crate::path::normalize_mpq_path(archive_name),
            compression: self.default_compression,
            encrypt: false,
            use_fix_key: false,
            locale: 0,
        });
        self
    }

    /// Add a file from memory with custom compression and encryption options
    ///
    /// Creates a file in the archive from in-memory data with full control
    /// over compression, encryption, and locale settings.
    ///
    /// # Parameters
    /// - `data`: Raw file data to store in the archive
    /// - `archive_name`: Name the file will have inside the archive
    /// - `compression`: Compression method from `compression::flags` (0 = no compression)
    /// - `encrypt`: Whether to encrypt the file
    /// - `locale`: Locale code for the file (0 = neutral locale)
    ///
    /// # Examples
    /// ```no_run
    /// use wow_mpq::{ArchiveBuilder, compression};
    ///
    /// let secret_data = b"TOP SECRET INFORMATION".to_vec();
    /// let builder = ArchiveBuilder::new()
    ///     .add_file_data_with_options(
    ///         secret_data,
    ///         "classified.txt",
    ///         compression::flags::LZMA,
    ///         true,  // encrypt
    ///         0      // neutral locale
    ///     );
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    pub fn add_file_data_with_options(
        mut self,
        data: Vec<u8>,
        archive_name: &str,
        compression: u8,
        encrypt: bool,
        locale: u16,
    ) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Data(data),
            archive_name: crate::path::normalize_mpq_path(archive_name),
            compression,
            encrypt,
            use_fix_key: false,
            locale,
        });
        self
    }

    /// Add a file with full encryption options including FIX_KEY support
    pub fn add_file_with_encryption<P: AsRef<Path>>(
        mut self,
        path: P,
        archive_name: &str,
        compression: u8,
        use_fix_key: bool,
        locale: u16,
    ) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Path(path.as_ref().to_path_buf()),
            archive_name: crate::path::normalize_mpq_path(archive_name),
            compression,
            encrypt: true,
            use_fix_key,
            locale,
        });
        self
    }

    /// Add file data with full encryption options including FIX_KEY support
    pub fn add_file_data_with_encryption(
        mut self,
        data: Vec<u8>,
        archive_name: &str,
        compression: u8,
        use_fix_key: bool,
        locale: u16,
    ) -> Self {
        self.pending_files.push(PendingFile {
            source: FileSource::Data(data),
            archive_name: crate::path::normalize_mpq_path(archive_name),
            compression,
            encrypt: true,
            use_fix_key,
            locale,
        });
        self
    }

    /// Calculate optimal hash table size based on file count
    fn calculate_hash_table_size(&self) -> u32 {
        let file_count = self.pending_files.len()
            + match &self.listfile_option {
                ListfileOption::Generate | ListfileOption::External(_) => 1,
                ListfileOption::None => 0,
            }
            + match &self.attributes_option {
                AttributesOption::GenerateCrc32
                | AttributesOption::GenerateFull
                | AttributesOption::External(_) => 1,
                AttributesOption::None => 0,
            };

        // Use 2x the file count for good performance, minimum 16
        let optimal_size = (file_count * 2).max(16) as u32;

        // Round up to next power of 2
        optimal_size.next_power_of_two()
    }

    /// Build the archive and write to the specified path
    pub fn build<P: AsRef<Path>>(mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Create a temporary file in the same directory
        let mut temp_file = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;

        // Add listfile if needed
        self.prepare_listfile()?;

        // Add attributes file if needed
        self.prepare_attributes()?;

        // Write the archive directly to the temp file
        {
            let file = temp_file.as_file_mut();
            use std::io::{Seek as _, Write as _};

            // For v3+ archives that need read-back support, we need to write everything
            // to a buffer first, then copy to file
            if self.version >= FormatVersion::V3 {
                // For v3+, we need to write everything to a buffer first
                // Estimate total size needed to avoid buffer reallocation issues
                let header_size = self.version.header_size() as usize;

                // Estimate file data size (assuming 10:1 compression ratio average)
                let estimated_file_data_size: usize = self
                    .pending_files
                    .iter()
                    .map(|f| match &f.source {
                        FileSource::Data(data) => data.len() / 10 + 1000, // Assume 10:1 compression + overhead
                        FileSource::Path(_) => 100_000,                   // Conservative estimate
                    })
                    .sum();

                // Add table sizes (conservative estimates)
                let estimated_table_size = self.pending_files.len() * 1000; // Conservative per-file overhead

                let total_estimated_size =
                    header_size + estimated_file_data_size + estimated_table_size;

                log::debug!(
                    "Pre-allocating buffer of {total_estimated_size} bytes for v3+ archive (header: {header_size}, estimated data: {estimated_file_data_size}, tables: {estimated_table_size})"
                );

                let mut vec = Vec::with_capacity(total_estimated_size);
                vec.resize(header_size, 0u8);
                let mut buffer = std::io::Cursor::new(vec);
                buffer.seek(SeekFrom::Start(header_size as u64))?;

                self.write_archive(&mut buffer)?;

                // Write the buffer to file
                file.write_all(buffer.get_ref())?;
                file.flush()?;
            } else {
                // For v1/v2, we can write directly
                self.write_archive(file)?;
                file.flush()?;
            }
        }

        // Atomically rename temp file to final destination
        temp_file.persist(path).map_err(|e| Error::Io(e.error))?;

        Ok(())
    }

    /// Prepare the listfile based on the option
    fn prepare_listfile(&mut self) -> Result<()> {
        match &self.listfile_option {
            ListfileOption::Generate => {
                // Generate listfile content from pending files
                let mut content = String::new();
                for file in &self.pending_files {
                    content.push_str(&file.archive_name);
                    content.push('\r');
                    content.push('\n');
                }

                // Add special files
                content.push_str("(listfile)\r\n");

                // Add attributes file if it will be generated
                if matches!(
                    self.attributes_option,
                    AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull
                ) {
                    content.push_str("(attributes)\r\n");
                }

                self.pending_files.push(PendingFile {
                    source: FileSource::Data(content.into_bytes()),
                    archive_name: "(listfile)".to_string(),
                    compression: self.default_compression,
                    encrypt: false,
                    use_fix_key: false,
                    locale: 0,
                });
            }
            ListfileOption::External(path) => {
                // Read external listfile
                let data = fs::read(path)?;

                self.pending_files.push(PendingFile {
                    source: FileSource::Data(data),
                    archive_name: "(listfile)".to_string(),
                    compression: self.default_compression,
                    encrypt: false,
                    use_fix_key: false,
                    locale: 0,
                });
            }
            ListfileOption::None => {}
        }

        Ok(())
    }

    /// Prepare the attributes file based on the option
    fn prepare_attributes(&mut self) -> Result<()> {
        // Import required types (will be used when we implement generation)
        // use crate::special_files::{Attributes, FileAttributes, AttributeFlags};

        match &self.attributes_option {
            AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull => {
                // We'll generate attributes after all files are written
                // For now, just mark that we need to generate them
                // The actual generation happens in write_archive methods
            }
            AttributesOption::External(path) => {
                // Read external attributes file
                let content = fs::read(path)?;
                self.pending_files.push(PendingFile {
                    source: FileSource::Data(content),
                    archive_name: "(attributes)".to_string(),
                    compression: 0, // Attributes are not compressed
                    encrypt: false,
                    use_fix_key: false,
                    locale: 0,
                });
            }
            AttributesOption::None => {}
        }

        Ok(())
    }

    /// Write the complete archive
    fn write_archive<W: Write + Seek + Read>(&self, writer: &mut W) -> Result<()> {
        // For v3+, we should create HET/BET tables instead of/in addition to hash/block
        let use_het_bet = self.version >= FormatVersion::V3;

        if use_het_bet {
            return self.write_archive_with_het_bet(writer);
        }

        let hash_table_size = self.calculate_hash_table_size();
        let mut block_table_size = self.pending_files.len() as u32;

        // Account for attributes file if it will be generated
        if matches!(
            self.attributes_option,
            AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull
        ) {
            block_table_size += 1;
        }

        // Calculate sector size
        let sector_size = crate::calculate_sector_size(self.block_size);

        // Reserve space for header (we'll write it at the end)
        let header_size = self.version.header_size();
        writer.seek(SeekFrom::Start(header_size as u64))?;

        // Build tables and write files
        let mut hash_table = HashTable::new(hash_table_size as usize)?;
        let mut block_table = BlockTable::new(block_table_size as usize)?;
        let mut hi_block_table = if self.version >= FormatVersion::V2 {
            Some(HiBlockTable::new(block_table_size as usize))
        } else {
            None
        };

        // Prepare to collect attributes if needed
        let collect_attributes = matches!(
            self.attributes_option,
            AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull
        );
        let mut collected_attributes = if collect_attributes {
            Some(Vec::new())
        } else {
            None
        };

        // Write all files and populate tables
        let mut actual_block_index = 0;
        for pending_file in self.pending_files.iter() {
            // Skip (attributes) file if it's being generated - we'll write it later
            if pending_file.archive_name == "(attributes)" && collect_attributes {
                continue;
            }

            let file_pos = writer.stream_position()?;

            // Read file data
            let file_data = match &pending_file.source {
                FileSource::Path(path) => fs::read(path)?,
                FileSource::Data(data) => data.clone(),
            };

            // Write file and get sizes
            let params = FileWriteParams {
                file_data: &file_data,
                archive_name: &pending_file.archive_name,
                compression: pending_file.compression,
                encrypt: pending_file.encrypt,
                use_fix_key: pending_file.use_fix_key,
                sector_size,
                file_pos,
            };

            let (compressed_size, flags, file_attr) = if collect_attributes {
                self.write_file_with_attributes(writer, &params)?
            } else {
                let (size, flags) = self.write_file(writer, &params)?;
                (size, flags, FileAttributes::new())
            };

            // Collect attributes if needed
            if let Some(ref mut attrs) = collected_attributes {
                attrs.push(file_attr);
            }

            // Add to hash table
            self.add_to_hash_table(
                &mut hash_table,
                &pending_file.archive_name,
                actual_block_index as u32,
                pending_file.locale,
            )?;

            // Add to block table and hi-block table if needed
            let block_entry = BlockEntry {
                file_pos: file_pos as u32, // Low 32 bits
                compressed_size: compressed_size as u32,
                file_size: file_data.len() as u32,
                flags: flags | BlockEntry::FLAG_EXISTS,
            };

            // Store high 16 bits in hi-block table if needed
            if let Some(ref mut hi_table) = hi_block_table {
                let high_bits = (file_pos >> 32) as u16;
                hi_table.set(actual_block_index, high_bits);
            }

            // Get mutable reference and update
            if let Some(entry) = block_table.get_mut(actual_block_index) {
                *entry = block_entry;
            } else {
                return Err(Error::invalid_format("Block index out of bounds"));
            }

            actual_block_index += 1;
        }

        // Generate and write attributes file if needed
        if let Some(attrs) = collected_attributes {
            log::debug!("Writing attributes for {} files", attrs.len());
            self.write_attributes_file(
                writer,
                &mut hash_table,
                &mut block_table,
                attrs,
                actual_block_index,
            )?;
        }

        // Write hash table
        let hash_table_pos = writer.stream_position()?;
        self.write_hash_table(writer, &hash_table)?;

        // Write block table
        let block_table_pos = writer.stream_position()?;
        self.write_block_table(writer, &block_table)?;

        // Write hi-block table if needed
        let hi_block_table_pos = if let Some(ref hi_table) = hi_block_table {
            if hi_table.is_needed() {
                let pos = writer.stream_position()?;
                self.write_hi_block_table(writer, hi_table)?;
                Some(pos)
            } else {
                None
            }
        } else {
            None
        };

        // Calculate archive size
        let archive_size = writer.stream_position()?;

        // Write header at the beginning
        writer.seek(SeekFrom::Start(0))?;
        let header_params = HeaderWriteParams {
            archive_size,
            hash_table_pos,
            block_table_pos,
            hash_table_size,
            block_table_size,
            hi_block_table_pos,
            het_table_pos: None,
            bet_table_pos: None,
            _het_table_size: None,
            _bet_table_size: None,
            v4_data: None, // V1/V2 don't use v4_data
        };
        self.write_header(writer, &header_params)?;

        // TODO: For V4, implement proper MD5 calculation

        Ok(())
    }

    /// Write archive with HET/BET tables (v3+)
    fn write_archive_with_het_bet<W: Write + Seek + Read>(&self, writer: &mut W) -> Result<()> {
        let mut block_table_size = self.pending_files.len() as u32;

        // Account for attributes file if it will be generated
        if matches!(
            self.attributes_option,
            AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull
        ) {
            block_table_size += 1;
        }

        // Calculate sector size
        let sector_size = crate::calculate_sector_size(self.block_size);

        // Reserve space for header by seeking past it (we'll write it at the end)
        let header_size = self.version.header_size();
        writer.seek(SeekFrom::Start(header_size as u64))?;

        // We'll still need block table data for file information
        let mut block_table = BlockTable::new(block_table_size as usize)?;
        let mut hi_block_table = Some(HiBlockTable::new(block_table_size as usize));

        // Prepare to collect attributes if needed
        let collect_attributes = matches!(
            self.attributes_option,
            AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull
        );
        let mut collected_attributes = if collect_attributes {
            Some(Vec::new())
        } else {
            None
        };

        // Write all files and populate block table
        let mut actual_block_index = 0;
        for pending_file in self.pending_files.iter() {
            // Skip (attributes) file if it's being generated - we'll write it later
            if pending_file.archive_name == "(attributes)" && collect_attributes {
                continue;
            }

            let file_pos = writer.stream_position()?;

            // Read file data
            let file_data = match &pending_file.source {
                FileSource::Path(path) => fs::read(path)?,
                FileSource::Data(data) => data.clone(),
            };

            // Write file and get sizes
            let params = FileWriteParams {
                file_data: &file_data,
                archive_name: &pending_file.archive_name,
                compression: pending_file.compression,
                encrypt: pending_file.encrypt,
                use_fix_key: pending_file.use_fix_key,
                sector_size,
                file_pos,
            };

            let (compressed_size, flags, file_attr) = if collect_attributes {
                self.write_file_with_attributes(writer, &params)?
            } else {
                let (size, flags) = self.write_file(writer, &params)?;
                (size, flags, FileAttributes::new())
            };

            // Collect attributes if needed
            if let Some(ref mut attrs) = collected_attributes {
                attrs.push(file_attr);
            }

            // Add to block table
            let block_entry = BlockEntry {
                file_pos: file_pos as u32, // Low 32 bits
                compressed_size: compressed_size as u32,
                file_size: file_data.len() as u32,
                flags: flags | BlockEntry::FLAG_EXISTS,
            };

            // Store high 16 bits in hi-block table
            if let Some(ref mut hi_table) = hi_block_table {
                let high_bits = (file_pos >> 32) as u16;
                hi_table.set(actual_block_index, high_bits);
            }

            // Update block table entry
            if let Some(entry) = block_table.get_mut(actual_block_index) {
                *entry = block_entry;
            } else {
                return Err(Error::invalid_format("Block index out of bounds"));
            }

            actual_block_index += 1;
        }

        // Track if we have collected attributes
        let has_collected_attributes = collected_attributes.is_some();

        // For compatibility, also write classic tables
        let hash_table_size = self.calculate_hash_table_size();
        let mut hash_table = HashTable::new(hash_table_size as usize)?;

        // Populate hash table
        let mut hash_block_index = 0;
        for pending_file in self.pending_files.iter() {
            // Skip (attributes) file if it's being generated - already processed
            if pending_file.archive_name == "(attributes)" && has_collected_attributes {
                continue;
            }

            self.add_to_hash_table(
                &mut hash_table,
                &pending_file.archive_name,
                hash_block_index as u32,
                pending_file.locale,
            )?;

            hash_block_index += 1;
        }

        // Write attributes file if we collected them
        if let Some(attrs) = collected_attributes {
            self.write_attributes_file(
                writer,
                &mut hash_table,
                &mut block_table,
                attrs,
                actual_block_index,
            )?;
        }

        // Create HET table (now includes proper attributes file info)
        let het_table_pos = writer.stream_position()?;
        let (het_data, _het_header) = self.create_het_table_with_hash_table(&hash_table)?;
        let (het_table_size, het_table_md5) = self.write_het_table(writer, &het_data, true)?;

        // Create BET table (now includes proper attributes file info)
        let bet_table_pos = writer.stream_position()?;
        let (bet_data, _bet_header) = self.create_bet_table(&block_table)?;
        let (bet_table_size, bet_table_md5) = self.write_bet_table(writer, &bet_data, true)?;

        // Write hash table
        let hash_table_pos = writer.stream_position()?;
        let hash_table_md5 = self.write_hash_table(writer, &hash_table)?;

        // Write block table
        let block_table_pos = writer.stream_position()?;
        let block_table_md5 = self.write_block_table(writer, &block_table)?;

        // Write hi-block table if needed
        let (hi_block_table_pos, hi_block_table_md5) = if let Some(ref hi_table) = hi_block_table {
            if hi_table.is_needed() {
                let pos = writer.stream_position()?;
                let md5 = self.write_hi_block_table(writer, hi_table)?;
                (Some(pos), md5)
            } else {
                (None, [0u8; 16])
            }
        } else {
            (None, [0u8; 16])
        };

        // Calculate archive size
        let archive_size = writer.stream_position()?;

        // Save the current position (end of archive)
        let _archive_end_pos = writer.stream_position()?;

        // Write header at the beginning
        writer.seek(SeekFrom::Start(0))?;

        // For V4, we need to use the MD5 checksums calculated during table writes
        let actual_file_count = block_table_size; // This includes attributes file
        let v4_data = if self.version == FormatVersion::V4 {
            Some(MpqHeaderV4Data {
                hash_table_size_64: hash_table_size as u64 * 16, // 16 bytes per hash entry
                block_table_size_64: actual_file_count as u64 * 16, // 16 bytes per block entry (includes attributes)
                hi_block_table_size_64: if let Some(ref hi_table) = hi_block_table {
                    if hi_table.is_needed() {
                        actual_file_count as u64 * 2 // 2 bytes per hi-block entry (includes attributes)
                    } else {
                        0
                    }
                } else {
                    0
                },
                het_table_size_64: het_table_size,
                bet_table_size_64: bet_table_size,
                raw_chunk_size: 0x4000, // 16KB default as per StormLib
                md5_block_table: block_table_md5,
                md5_hash_table: hash_table_md5,
                md5_hi_block_table: hi_block_table_md5,
                md5_bet_table: bet_table_md5,
                md5_het_table: het_table_md5,
                md5_mpq_header: [0u8; 16], // Will be calculated after header write
            })
        } else {
            None
        };

        let header_params = HeaderWriteParams {
            archive_size,
            hash_table_pos,
            block_table_pos,
            hash_table_size,
            block_table_size: actual_file_count,
            hi_block_table_pos,
            het_table_pos: Some(het_table_pos),
            bet_table_pos: Some(bet_table_pos),
            _het_table_size: Some(het_table_size),
            _bet_table_size: Some(bet_table_size),
            v4_data,
        };

        // Write header
        self.write_header(writer, &header_params)?;

        // For V4, calculate and write the header MD5
        if self.version == FormatVersion::V4 {
            self.finalize_v4_header_md5(writer)?;
        }

        Ok(())
    }

    /// Write a single file to the archive and collect attributes
    fn write_file_with_attributes<W: Write>(
        &self,
        writer: &mut W,
        params: &FileWriteParams<'_>,
    ) -> Result<(usize, u32, FileAttributes)> {
        let (size, flags) = self.write_file(writer, params)?;

        // Create file attributes based on what we calculated
        let mut file_attr = FileAttributes::new();

        // CRC32 is calculated from uncompressed data
        if matches!(
            self.attributes_option,
            AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull
        ) {
            let crc32 = crc32fast::hash(params.file_data);
            file_attr.crc32 = Some(crc32);
        }

        // MD5 if requested
        if matches!(self.attributes_option, AttributesOption::GenerateFull) {
            let mut hasher = Md5::new();
            hasher.update(params.file_data);
            let md5_result = hasher.finalize();
            let mut md5_bytes = [0u8; 16];
            md5_bytes.copy_from_slice(&md5_result);
            file_attr.md5 = Some(md5_bytes);
        }

        // File time (use current time for now)
        if matches!(self.attributes_option, AttributesOption::GenerateFull) {
            // Convert current time to Windows FILETIME (100-nanosecond intervals since 1601-01-01)
            use std::time::{SystemTime, UNIX_EPOCH};
            let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let unix_seconds = duration.as_secs();
            // Windows epoch is 11644473600 seconds before Unix epoch
            let windows_seconds = unix_seconds + 11644473600;
            let filetime = windows_seconds * 10_000_000; // Convert to 100-nanosecond intervals
            file_attr.filetime = Some(filetime);
        }

        Ok((size, flags, file_attr))
    }

    /// Write a single file to the archive
    fn write_file<W: Write>(
        &self,
        writer: &mut W,
        params: &FileWriteParams<'_>,
    ) -> Result<(usize, u32)> {
        let FileWriteParams {
            file_data,
            archive_name,
            compression,
            encrypt,
            use_fix_key,
            sector_size,
            file_pos,
        } = params;
        let mut flags = 0u32;

        // For small files or if single unit is requested, write as single unit
        let is_single_unit = file_data.len() <= *sector_size;

        if is_single_unit {
            flags |= BlockEntry::FLAG_SINGLE_UNIT;

            // Set CRC flag early if enabled (needed for encryption key calculation)
            if self.generate_crcs {
                flags |= BlockEntry::FLAG_SECTOR_CRC;
            }

            // Compress if needed
            let compressed_data = if *compression != 0 && !file_data.is_empty() {
                log::debug!("Compressing {archive_name} with method 0x{compression:02X}");
                let compressed = compress(file_data, *compression)?;

                // The compress function now handles the compression byte prefix
                // and only returns compressed data if it's beneficial
                if compressed != *file_data {
                    // Compression was beneficial and the data now includes the method byte
                    log::debug!(
                        "Compression successful: {} -> {} bytes (including method byte)",
                        file_data.len(),
                        compressed.len()
                    );
                    flags |= BlockEntry::FLAG_COMPRESS;
                    compressed
                } else {
                    // Compression not beneficial, returned original data
                    log::debug!("Compression not beneficial, storing uncompressed");
                    file_data.to_vec()
                }
            } else {
                file_data.to_vec()
            };

            // Encrypt if needed
            let final_data = if *encrypt {
                flags |= BlockEntry::FLAG_ENCRYPTED;
                if *use_fix_key {
                    flags |= BlockEntry::FLAG_FIX_KEY;
                }
                let key =
                    self.calculate_file_key(archive_name, *file_pos, file_data.len() as u32, flags);
                let mut encrypted = compressed_data;
                self.encrypt_data(&mut encrypted, key);
                encrypted
            } else {
                compressed_data
            };

            // Write the data
            writer.write_all(&final_data)?;

            // Write CRC if enabled
            if self.generate_crcs {
                // MPQ uses ADLER32 for sector checksums
                let crc = adler2::adler32_slice(file_data);
                writer.write_u32_le(crc)?;
                log::debug!("Generated CRC for single unit file {archive_name}: 0x{crc:08X}");
            }

            // Return compressed size (NOT including CRC)
            Ok((final_data.len(), flags))
        } else {
            // Multi-sector file
            let sector_count = file_data.len().div_ceil(*sector_size);

            // Set CRC flag early if enabled (needed for encryption key calculation)
            if self.generate_crcs {
                flags |= BlockEntry::FLAG_SECTOR_CRC;
            }

            // Reserve space for sector offset table and CRC table if enabled
            let offset_table_size = (sector_count + 1) * 4;
            let crc_table_size = if self.generate_crcs {
                sector_count * 4
            } else {
                0
            };
            let data_start = offset_table_size + crc_table_size;

            let mut sector_offsets = vec![0u32; sector_count + 1];
            let mut sector_data = Vec::new();
            let mut sector_crcs = if self.generate_crcs {
                Vec::with_capacity(sector_count)
            } else {
                Vec::new()
            };

            // Process each sector
            for (i, offset) in sector_offsets.iter_mut().enumerate().take(sector_count) {
                let sector_start = i * *sector_size;
                let sector_end = ((i + 1) * *sector_size).min(file_data.len());
                let sector_bytes = &file_data[sector_start..sector_end];

                *offset = (data_start + sector_data.len()) as u32;

                // Calculate CRC for uncompressed sector if enabled
                if self.generate_crcs {
                    // MPQ uses ADLER32 for sector checksums
                    let crc = adler2::adler32_slice(sector_bytes);
                    sector_crcs.push(crc);
                }

                // Compress sector if needed
                let compressed_sector = if *compression != 0 && !sector_bytes.is_empty() {
                    // The compress function now handles the compression byte prefix
                    // and only returns compressed data if it's beneficial
                    let compressed = compress(sector_bytes, *compression)?;
                    if compressed != *sector_bytes {
                        // Compression was beneficial and the data now includes the method byte
                        flags |= BlockEntry::FLAG_COMPRESS;
                        compressed
                    } else {
                        // Compression not beneficial, returned original data
                        sector_bytes.to_vec()
                    }
                } else {
                    sector_bytes.to_vec()
                };

                sector_data.extend_from_slice(&compressed_sector);
            }

            // Set last offset
            sector_offsets[sector_count] = (data_start + sector_data.len()) as u32;

            // Log CRC generation if enabled
            if self.generate_crcs {
                log::debug!(
                    "Generated {} sector CRCs for file {}, first few: {:?}",
                    sector_count,
                    archive_name,
                    &sector_crcs[..5.min(sector_crcs.len())]
                );
            }

            // Encrypt if needed
            if *encrypt {
                flags |= BlockEntry::FLAG_ENCRYPTED;
                if *use_fix_key {
                    flags |= BlockEntry::FLAG_FIX_KEY;
                }
                let key =
                    self.calculate_file_key(archive_name, *file_pos, file_data.len() as u32, flags);

                // Save original offsets for sector encryption
                let original_offsets = sector_offsets.clone();

                // Encrypt sector offset table
                let offset_key = key.wrapping_sub(1);
                self.encrypt_data_u32(&mut sector_offsets, offset_key);

                // Encrypt each sector using the original (unencrypted) offsets
                let mut encrypted_sectors = Vec::new();
                for (i, offset_pair) in original_offsets.windows(2).enumerate() {
                    let start = (offset_pair[0] - data_start as u32) as usize;
                    let end = (offset_pair[1] - data_start as u32) as usize;

                    let mut sector = sector_data[start..end].to_vec();
                    let sector_key = key.wrapping_add(i as u32);
                    self.encrypt_data(&mut sector, sector_key);
                    encrypted_sectors.extend_from_slice(&sector);
                }

                sector_data = encrypted_sectors;
            }

            // Write sector offset table
            for offset in &sector_offsets {
                writer.write_u32_le(*offset)?;
            }

            // Write CRC table if enabled
            if self.generate_crcs {
                for crc in &sector_crcs {
                    writer.write_u32_le(*crc)?;
                }
            }

            // Write sector data
            writer.write_all(&sector_data)?;

            // Return size NOT including CRC table (offset table + sector data only)
            let total_size = offset_table_size + sector_data.len();
            Ok((total_size, flags))
        }
    }

    /// Write the attributes file to the archive
    fn write_attributes_file<W: Write + Seek>(
        &self,
        writer: &mut W,
        hash_table: &mut HashTable,
        block_table: &mut BlockTable,
        file_attributes: Vec<FileAttributes>,
        block_index: usize,
    ) -> Result<()> {
        // Check if we're actually generating attributes
        let flags = match self.attributes_option {
            AttributesOption::GenerateCrc32 => AttributeFlags::CRC32,
            AttributesOption::GenerateFull => {
                AttributeFlags::CRC32 | AttributeFlags::MD5 | AttributeFlags::FILETIME
            }
            _ => return Ok(()), // Should not happen due to earlier checks
        };

        // Create attributes structure
        let attributes = Attributes {
            version: Attributes::EXPECTED_VERSION,
            flags: AttributeFlags::new(flags),
            file_attributes,
            crc32: None,    // Phase 1 stub
            md5: None,      // Phase 1 stub
            filetime: None, // Phase 1 stub
        };

        // Convert to bytes
        let attributes_data = attributes.to_bytes()?;

        // Write the attributes file
        let file_pos = writer.stream_position()?;
        writer.write_all(&attributes_data)?;

        // Use the provided block index

        let block_entry = BlockEntry {
            file_pos: file_pos as u32,
            compressed_size: attributes_data.len() as u32,
            file_size: attributes_data.len() as u32,
            flags: BlockEntry::FLAG_EXISTS,
        };

        // Update the block table entry
        if let Some(entry) = block_table.get_mut(block_index) {
            *entry = block_entry;
        } else {
            return Err(Error::invalid_format(
                "Invalid block index for attributes file",
            ));
        }

        // Add to hash table
        self.add_to_hash_table(hash_table, "(attributes)", block_index as u32, 0)?;

        Ok(())
    }

    /// Add a file to the hash table
    fn add_to_hash_table(
        &self,
        hash_table: &mut HashTable,
        filename: &str,
        block_index: u32,
        locale: u16,
    ) -> Result<()> {
        let table_offset = hash_string(filename, hash_type::TABLE_OFFSET);
        let name_a = hash_string(filename, hash_type::NAME_A);
        let name_b = hash_string(filename, hash_type::NAME_B);

        let table_size = hash_table.size() as u32;
        let mut index = table_offset & (table_size - 1);

        // Linear probing to find empty slot
        loop {
            let entry = hash_table
                .get_mut(index as usize)
                .ok_or_else(|| Error::invalid_format("Hash table index out of bounds"))?;

            if entry.is_empty() {
                // Found empty slot
                *entry = HashEntry {
                    name_1: name_a,
                    name_2: name_b,
                    locale,
                    platform: 0, // Always 0 - platform codes are vestigial
                    block_index,
                };
                break;
            }

            // Check for duplicate
            if entry.name_1 == name_a && entry.name_2 == name_b && entry.locale == locale {
                return Err(Error::invalid_format(format!(
                    "Duplicate file in archive: {filename}"
                )));
            }

            // Move to next slot
            index = (index + 1) & (table_size - 1);
        }

        Ok(())
    }

    /// Write the hash table
    fn write_hash_table<W: Write>(
        &self,
        writer: &mut W,
        hash_table: &HashTable,
    ) -> Result<[u8; 16]> {
        // Convert to bytes for encryption
        let mut table_data = Vec::new();
        for entry in hash_table.entries() {
            table_data.write_u32_le(entry.name_1)?;
            table_data.write_u32_le(entry.name_2)?;
            table_data.write_u16_le(entry.locale)?;
            table_data.write_u16_le(entry.platform)?;
            table_data.write_u32_le(entry.block_index)?;
        }

        // Encrypt the table
        let key = hash_string("(hash table)", hash_type::FILE_KEY);
        self.encrypt_data(&mut table_data, key);

        // Calculate MD5 of encrypted data (for v4)
        let md5 = self.calculate_md5(&table_data);

        // Write encrypted table
        writer.write_all(&table_data)?;

        Ok(md5)
    }

    /// Write the block table
    fn write_block_table<W: Write>(
        &self,
        writer: &mut W,
        block_table: &BlockTable,
    ) -> Result<[u8; 16]> {
        // Convert to bytes for encryption
        let mut table_data = Vec::new();
        for entry in block_table.entries() {
            table_data.write_u32_le(entry.file_pos)?;
            table_data.write_u32_le(entry.compressed_size)?;
            table_data.write_u32_le(entry.file_size)?;
            table_data.write_u32_le(entry.flags)?;
        }

        // Encrypt the table
        let key = hash_string("(block table)", hash_type::FILE_KEY);
        self.encrypt_data(&mut table_data, key);

        // Calculate MD5 of encrypted data (for v4)
        let md5 = self.calculate_md5(&table_data);

        // Write encrypted table
        writer.write_all(&table_data)?;

        Ok(md5)
    }

    /// Write the hi-block table
    fn write_hi_block_table<W: Write>(
        &self,
        writer: &mut W,
        hi_block_table: &HiBlockTable,
    ) -> Result<[u8; 16]> {
        // Hi-block table is not encrypted
        let mut table_data = Vec::new();
        for &entry in hi_block_table.entries() {
            table_data.write_u16_le(entry)?;
        }

        // Calculate MD5 (for v4)
        let md5 = self.calculate_md5(&table_data);

        // Write table
        writer.write_all(&table_data)?;

        Ok(md5)
    }

    /// Write the MPQ header
    fn write_header<W: Write + Seek>(
        &self,
        writer: &mut W,
        params: &HeaderWriteParams,
    ) -> Result<()> {
        // Write signature
        writer.write_u32_le(crate::signatures::MPQ_ARCHIVE)?;

        // Write header size
        writer.write_u32_le(self.version.header_size())?;

        // Write archive size (32-bit for v1, deprecated in v2+)
        writer.write_u32_le(params.archive_size.min(u32::MAX as u64) as u32)?;

        // Write format version
        writer.write_u16_le(self.version as u16)?;

        // Write block size
        writer.write_u16_le(self.block_size)?;

        // Write table positions and sizes (low 32 bits)
        writer.write_u32_le(params.hash_table_pos as u32)?;
        writer.write_u32_le(params.block_table_pos as u32)?;
        writer.write_u32_le(params.hash_table_size)?;
        writer.write_u32_le(params.block_table_size)?;

        // Write version-specific fields
        match self.version {
            FormatVersion::V1 => {
                // No additional fields
            }
            FormatVersion::V2 => {
                // Hi-block table position
                writer.write_u64_le(params.hi_block_table_pos.unwrap_or(0))?;

                // High 16 bits of positions
                writer.write_u16_le((params.hash_table_pos >> 32) as u16)?; // hash_table_pos_hi
                writer.write_u16_le((params.block_table_pos >> 32) as u16)?; // block_table_pos_hi
            }
            FormatVersion::V3 => {
                // V2 fields
                writer.write_u64_le(params.hi_block_table_pos.unwrap_or(0))?; // hi_block_table_pos
                writer.write_u16_le((params.hash_table_pos >> 32) as u16)?; // hash_table_pos_hi
                writer.write_u16_le((params.block_table_pos >> 32) as u16)?; // block_table_pos_hi

                // V3 fields
                writer.write_u64_le(params.archive_size)?; // archive_size_64
                writer.write_u64_le(params.het_table_pos.unwrap_or(0))?; // het_table_pos
                writer.write_u64_le(params.bet_table_pos.unwrap_or(0))?; // bet_table_pos
            }
            FormatVersion::V4 => {
                // V2 fields
                writer.write_u64_le(params.hi_block_table_pos.unwrap_or(0))?; // hi_block_table_pos
                writer.write_u16_le((params.hash_table_pos >> 32) as u16)?; // hash_table_pos_hi
                writer.write_u16_le((params.block_table_pos >> 32) as u16)?; // block_table_pos_hi

                // V3 fields
                writer.write_u64_le(params.archive_size)?; // archive_size_64
                writer.write_u64_le(params.het_table_pos.unwrap_or(0))?; // het_table_pos
                writer.write_u64_le(params.bet_table_pos.unwrap_or(0))?; // bet_table_pos

                // V4 fields
                if let Some(v4_data) = &params.v4_data {
                    writer.write_u64_le(v4_data.hash_table_size_64)?;
                    writer.write_u64_le(v4_data.block_table_size_64)?;
                    writer.write_u64_le(v4_data.hi_block_table_size_64)?;
                    writer.write_u64_le(v4_data.het_table_size_64)?;
                    writer.write_u64_le(v4_data.bet_table_size_64)?;
                    writer.write_u32_le(v4_data.raw_chunk_size)?;

                    // Write MD5 hashes (all except header MD5 which is calculated later)
                    writer.write_all(&v4_data.md5_block_table)?;
                    writer.write_all(&v4_data.md5_hash_table)?;
                    writer.write_all(&v4_data.md5_hi_block_table)?;
                    writer.write_all(&v4_data.md5_bet_table)?;
                    writer.write_all(&v4_data.md5_het_table)?;
                    writer.write_all(&v4_data.md5_mpq_header)?;
                } else {
                    return Err(Error::invalid_format("V4 format requires v4_data"));
                }
            }
        }

        Ok(())
    }

    /// Calculate file encryption key
    fn calculate_file_key(&self, filename: &str, file_pos: u64, file_size: u32, flags: u32) -> u32 {
        let base_key = hash_string(filename, hash_type::FILE_KEY);

        if flags & BlockEntry::FLAG_FIX_KEY != 0 {
            // For FIX_KEY, use only the low 32 bits of the file position
            (base_key.wrapping_add(file_pos as u32)) ^ file_size
        } else {
            base_key
        }
    }

    /// Encrypt data in place
    pub fn encrypt_data(&self, data: &mut [u8], key: u32) {
        if data.is_empty() || key == 0 {
            return;
        }

        // Process full u32 chunks
        let (chunks, remainder) = data.split_at_mut((data.len() / 4) * 4);

        // Convert chunks to u32 values, encrypt, and write back
        let mut u32_buffer = Vec::with_capacity(chunks.len() / 4);
        for chunk in chunks.chunks_exact(4) {
            u32_buffer.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }

        encrypt_block(&mut u32_buffer, key);

        // Write encrypted u32s back to bytes
        for (i, &encrypted) in u32_buffer.iter().enumerate() {
            let bytes = encrypted.to_le_bytes();
            chunks[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }

        // Handle remaining bytes
        if !remainder.is_empty() {
            let mut last_dword = [0u8; 4];
            last_dword[..remainder.len()].copy_from_slice(remainder);

            let mut last_u32 = u32::from_le_bytes(last_dword);
            encrypt_block(
                std::slice::from_mut(&mut last_u32),
                key.wrapping_add((chunks.len() / 4) as u32),
            );

            let encrypted_bytes = last_u32.to_le_bytes();
            remainder.copy_from_slice(&encrypted_bytes[..remainder.len()]);
        }
    }
    /// Encrypt u32 data in place
    fn encrypt_data_u32(&self, data: &mut [u32], key: u32) {
        encrypt_block(data, key);
    }

    /// Calculate MD5 hash of data
    fn calculate_md5(&self, data: &[u8]) -> [u8; 16] {
        let mut hasher = Md5::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// Finalize V4 header by calculating and writing the header MD5
    fn finalize_v4_header_md5<W: Write + Seek + Read>(&self, writer: &mut W) -> Result<()> {
        // Read the header data (excluding the MD5 field itself)
        writer.seek(SeekFrom::Start(0))?;
        let header_size = self.version.header_size() as usize;
        let md5_size = 16;
        let header_data_size = header_size - md5_size; // 208 - 16 = 192 bytes

        let mut header_data = vec![0u8; header_data_size];
        writer.read_exact(&mut header_data)?;

        // Calculate MD5 of header data
        let header_md5 = self.calculate_md5(&header_data);

        // Write the MD5 at the end of the header (offset 0xC0 = 192)
        writer.seek(SeekFrom::Start(192))?;
        writer.write_all(&header_md5)?;

        Ok(())
    }

    /// Create HET table data using the final hash table (includes attributes file)
    fn create_het_table_with_hash_table(
        &self,
        hash_table: &HashTable,
    ) -> Result<(Vec<u8>, HetHeader)> {
        // Count actual files from the hash table
        let mut file_count = 0u32;

        // Extract filenames from hash table entries
        for entry in hash_table.entries() {
            if !entry.is_empty() {
                file_count += 1;
            }
        }

        // For simplicity, let's use the same logic as create_het_table but with proper file count
        let hash_table_entries = (file_count * 2).next_power_of_two();

        log::debug!(
            "Creating HET table from hash_table: {file_count} files, {hash_table_entries} hash entries"
        );

        // Create header
        let header = HetHeader {
            table_size: 0, // Will be calculated later
            max_file_count: file_count,
            hash_table_size: hash_table_entries, // Number of hash entries (in bytes)
            hash_entry_size: 8,                  // Always 8 bits for the name hash
            total_index_size: hash_table_entries * Self::calculate_bits_needed(file_count as u64),
            index_size_extra: 0,
            index_size: Self::calculate_bits_needed(file_count as u64),
            block_table_size: 0, // Not used
        };

        // Copy values from packed struct to avoid alignment issues
        let index_size = header.index_size;

        // Create hash table (8-bit name hashes)
        let mut het_hash_table = vec![0xFFu8; hash_table_entries as usize]; // Initialize with 0xFF (empty)

        // Create file indices array
        let file_indices_size = (header.total_index_size as usize).div_ceil(8);
        let mut file_indices = vec![0u8; file_indices_size]; // Initialize with 0

        // Pre-fill with invalid indices (all bits set)
        let invalid_index = (1u64 << index_size) - 1; // e.g., 0b111 for 3 bits = 7
        for i in 0..hash_table_entries {
            self.write_bit_entry(&mut file_indices, i as usize, invalid_index, index_size)?;
        }

        // Process files from the original pending_files plus attributes if present
        let mut file_index = 0;

        // Process pending files (excluding attributes to match write order)
        let collect_attributes = matches!(
            self.attributes_option,
            AttributesOption::GenerateCrc32 | AttributesOption::GenerateFull
        );

        for pending_file in self.pending_files.iter() {
            // Skip (attributes) file if it's being generated - we'll add it later
            if pending_file.archive_name == "(attributes)" && collect_attributes {
                continue;
            }

            let hash_bits = 8;
            let (hash, name_hash1) = het_hash(&pending_file.archive_name, hash_bits);

            // Calculate starting position for linear probing
            let start_index = (hash % hash_table_entries as u64) as usize;

            // Linear probing for collision resolution
            let mut current_index = start_index;
            loop {
                // Check if slot is empty (0xFF)
                if het_hash_table[current_index] == 0xFF {
                    // Store the 8-bit name hash
                    het_hash_table[current_index] = name_hash1;

                    // Store the file index in the bit-packed array
                    self.write_bit_entry(
                        &mut file_indices,
                        current_index,
                        file_index as u64,
                        index_size,
                    )?;

                    break;
                }

                current_index = (current_index + 1) % hash_table_entries as usize;
                if current_index == start_index {
                    return Err(Error::invalid_format("HET table full"));
                }
            }
            file_index += 1;
        }

        // Add attributes file if it was generated (use the same file_index that write_attributes_file used)
        if collect_attributes {
            let hash_bits = 8;
            let (hash, name_hash1) = het_hash("(attributes)", hash_bits);
            let start_index = (hash % hash_table_entries as u64) as usize;

            let mut current_index = start_index;
            loop {
                if het_hash_table[current_index] == 0xFF {
                    het_hash_table[current_index] = name_hash1;
                    self.write_bit_entry(
                        &mut file_indices,
                        current_index,
                        file_index as u64,
                        index_size,
                    )?;
                    break;
                }

                current_index = (current_index + 1) % hash_table_entries as usize;
                if current_index == start_index {
                    return Err(Error::invalid_format("HET table full"));
                }
            }
        }

        // Calculate sizes
        let het_header_size = std::mem::size_of::<HetHeader>();
        let data_size = het_header_size as u32 + hash_table_entries + file_indices_size as u32;
        let table_size = 12 + data_size; // Extended header (12 bytes) + data

        // Update header with final size
        let mut final_header = header;
        final_header.table_size = table_size;

        // Build the final result
        let mut result = Vec::with_capacity((12 + data_size) as usize);

        // Write extended header
        result.write_u32_le(0x1A544548)?; // "HET\x1A"
        result.write_u32_le(1)?; // version
        result.write_u32_le(data_size)?; // data_size

        // Write HET header
        result.write_u32_le(final_header.table_size)?;
        result.write_u32_le(final_header.max_file_count)?;
        result.write_u32_le(final_header.hash_table_size)?;
        result.write_u32_le(final_header.hash_entry_size)?;
        result.write_u32_le(final_header.total_index_size)?;
        result.write_u32_le(final_header.index_size_extra)?;
        result.write_u32_le(final_header.index_size)?;
        result.write_u32_le(final_header.block_table_size)?;

        // Write hash table and file indices
        result.extend_from_slice(&het_hash_table);
        result.extend_from_slice(&file_indices);

        log::debug!("HET table created with {file_count} files (including attributes)");

        Ok((result, final_header))
    }

    /// Write a bit-packed entry to a byte array
    fn write_bit_entry(
        &self,
        data: &mut [u8],
        index: usize,
        value: u64,
        bit_size: u32,
    ) -> Result<()> {
        let bit_offset = index * bit_size as usize;
        let byte_offset = bit_offset / 8;
        let bit_shift = bit_offset % 8;

        // Calculate how many bytes we actually need
        let bits_needed = bit_shift + bit_size as usize;
        let bytes_needed = bits_needed.div_ceil(8);

        if byte_offset + bytes_needed > data.len() {
            log::error!(
                "Bit entry out of bounds: index={}, bit_size={}, bit_offset={}, byte_offset={}, bytes_needed={}, data.len()={}",
                index,
                bit_size,
                bit_offset,
                byte_offset,
                bytes_needed,
                data.len()
            );
            return Err(Error::invalid_format("Bit entry out of bounds"));
        }

        // Read existing bits (limit to 8 bytes for u64)
        let mut existing = 0u64;
        let max_bytes = bytes_needed.min(8);
        for i in 0..max_bytes {
            if byte_offset + i < data.len() && i * 8 < 64 {
                existing |= (data[byte_offset + i] as u64) << (i * 8);
            }
        }

        // Clear the bits we're about to write
        let value_mask = if bit_size >= 64 {
            u64::MAX
        } else {
            (1u64 << bit_size) - 1
        };
        let mask = value_mask << bit_shift;
        existing &= !mask;

        // Write the new value
        existing |= (value & value_mask) << bit_shift;

        // Write back (limit to 8 bytes for u64)
        for i in 0..max_bytes {
            if byte_offset + i < data.len() && i * 8 < 64 {
                data[byte_offset + i] = (existing >> (i * 8)) as u8;
            }
        }

        Ok(())
    }

    /// Calculate the number of bits needed to represent a value
    fn calculate_bits_needed(max_value: u64) -> u32 {
        if max_value == 0 {
            1
        } else {
            (64 - max_value.leading_zeros()).max(1)
        }
    }

    /// Write HET table to the archive, returns the written size and MD5
    fn write_het_table<W: Write>(
        &self,
        writer: &mut W,
        data: &[u8],
        encrypt: bool,
    ) -> Result<(u64, [u8; 16])> {
        // HET table structure:
        // - Extended header (12 bytes) - NEVER encrypted
        // - Table data (rest) - can be compressed and/or encrypted

        if data.len() < 12 {
            return Err(Error::invalid_format("HET table data too small"));
        }

        // Split extended header and table data
        let (extended_header, table_data) = data.split_at(12);
        let mut processed_data = table_data.to_vec();

        log::debug!(
            "HET table before processing: extended_header len={}, table_data len={}",
            extended_header.len(),
            processed_data.len()
        );
        log::debug!(
            "HET table data (first 20 bytes): {:?}",
            &processed_data[..processed_data.len().min(20)]
        );

        // Compress if enabled and this is a v3+ archive
        if self.compress_tables && matches!(self.version, FormatVersion::V3 | FormatVersion::V4) {
            log::debug!("Compressing HET table data: {} -> ", processed_data.len());
            let compressed = compress(&processed_data, self.table_compression)?;
            log::debug!(
                "{} bytes ({}% reduction)",
                compressed.len(),
                (100 * (processed_data.len() - compressed.len()) / processed_data.len())
            );

            // Prepend compression type byte
            let mut compressed_with_type = Vec::with_capacity(1 + compressed.len());
            compressed_with_type.push(self.table_compression);
            compressed_with_type.extend_from_slice(&compressed);
            processed_data = compressed_with_type;
        }

        // Encrypt the data portion (after extended header)
        if encrypt {
            let key = hash_string("(hash table)", hash_type::FILE_KEY);
            log::debug!("Encrypting HET table with key: 0x{key:08X}");
            log::debug!(
                "Data size: {} bytes (multiple of 4: {})",
                processed_data.len(),
                processed_data.len() % 4 == 0
            );
            log::debug!(
                "Data before encryption (last 10 bytes): {:?}",
                &processed_data[processed_data.len().saturating_sub(10)..]
            );
            self.encrypt_data(&mut processed_data, key);
            log::debug!(
                "Data after encryption (last 10 bytes): {:?}",
                &processed_data[processed_data.len().saturating_sub(10)..]
            );
        }

        // Combine extended header with processed data
        let mut final_data = Vec::with_capacity(extended_header.len() + processed_data.len());
        final_data.extend_from_slice(extended_header);
        final_data.extend_from_slice(&processed_data);

        // Calculate MD5 of final data
        let md5 = self.calculate_md5(&final_data);

        let written_size = final_data.len() as u64;
        writer.write_all(&final_data)?;
        Ok((written_size, md5))
    }

    /// Create BET table data
    fn create_bet_table(&self, block_table: &BlockTable) -> Result<(Vec<u8>, BetHeader)> {
        // Get actual file count from block table entries (includes attributes if generated)
        let file_count = block_table.entries().len() as u32;

        // Analyze block table to determine optimal bit widths
        let mut max_file_pos = 0u64;
        let mut max_file_size = 0u64;
        let mut max_compressed_size = 0u64;
        let mut unique_flags = std::collections::HashSet::new();

        for i in 0..file_count as usize {
            if let Some(entry) = block_table.get(i) {
                max_file_pos = max_file_pos.max(entry.file_pos as u64);
                max_file_size = max_file_size.max(entry.file_size as u64);
                max_compressed_size = max_compressed_size.max(entry.compressed_size as u64);
                unique_flags.insert(entry.flags);
            }
        }

        // Calculate bit counts for each field
        let bit_count_file_pos = Self::calculate_bits_needed(max_file_pos);
        let bit_count_file_size = Self::calculate_bits_needed(max_file_size);
        let bit_count_cmp_size = Self::calculate_bits_needed(max_compressed_size);
        let bit_count_flag_index = if unique_flags.is_empty() {
            0
        } else {
            Self::calculate_bits_needed(unique_flags.len() as u64 - 1)
        };
        let bit_count_unknown = 0; // Not used

        // Calculate bit positions
        let bit_index_file_pos = 0;
        let bit_index_file_size = bit_index_file_pos + bit_count_file_pos;
        let bit_index_cmp_size = bit_index_file_size + bit_count_file_size;
        let bit_index_flag_index = bit_index_cmp_size + bit_count_cmp_size;
        let bit_index_unknown = bit_index_flag_index + bit_count_flag_index;

        // Calculate table entry size
        let table_entry_size = bit_index_unknown + bit_count_unknown;

        // Create flag array
        let mut flag_array: Vec<u32> = unique_flags.into_iter().collect();
        flag_array.sort();
        let flag_count = flag_array.len() as u32;

        // Create flag index map
        let mut flag_index_map = std::collections::HashMap::new();
        for (index, &flags) in flag_array.iter().enumerate() {
            flag_index_map.insert(flags, index as u32);
        }

        // Calculate table sizes
        let file_table_bits = file_count * table_entry_size;
        let file_table_size = file_table_bits.div_ceil(8); // Round up to bytes

        // BET hash information (simplified - we'll use 64-bit hashes)
        let bet_hash_size = 64;
        let total_bet_hash_size = file_count * bet_hash_size;
        let bet_hash_size_extra = 0;
        let bet_hash_array_size = total_bet_hash_size.div_ceil(8);

        // Create header (without extended header fields)
        let header = BetHeader {
            table_size: 0, // Will be calculated later
            file_count,
            unknown_08: 0x10,
            table_entry_size,
            bit_index_file_pos,
            bit_index_file_size,
            bit_index_cmp_size,
            bit_index_flag_index,
            bit_index_unknown,
            bit_count_file_pos,
            bit_count_file_size,
            bit_count_cmp_size,
            bit_count_flag_index,
            bit_count_unknown,
            total_bet_hash_size,
            bet_hash_size_extra,
            bet_hash_size,
            bet_hash_array_size,
            flag_count,
        };

        // Create file table
        let mut file_table = vec![0u8; file_table_size as usize];

        // Create BET hashes
        let mut bet_hashes = Vec::with_capacity(file_count as usize);

        // Fill tables
        for i in 0..file_count as usize {
            if let Some(entry) = block_table.get(i) {
                // Get flag index
                let flag_index = flag_index_map.get(&entry.flags).unwrap();

                // Pack entry data
                let mut entry_bits = 0u64;
                entry_bits |= (entry.file_pos as u64) << bit_index_file_pos;
                entry_bits |= (entry.file_size as u64) << bit_index_file_size;
                entry_bits |= (entry.compressed_size as u64) << bit_index_cmp_size;
                entry_bits |= (*flag_index as u64) << bit_index_flag_index;

                // Write to file table
                self.write_bit_entry(&mut file_table, i, entry_bits, table_entry_size)?;

                // Generate BET hash (Jenkins one-at-a-time hash of filename)
                // Note: BET uses Jenkins one-at-a-time, not hashlittle2 like HET
                let filename = if i < self.pending_files.len() {
                    &self.pending_files[i].archive_name
                } else {
                    // This must be the attributes file
                    "(attributes)"
                };
                let hash = jenkins_hash(filename);
                bet_hashes.push(hash);
            }
        }

        // Calculate final sizes
        let bet_header_size = std::mem::size_of::<BetHeader>();
        let flag_array_size = flag_count * 4;
        let data_size =
            bet_header_size as u32 + flag_array_size + file_table_size + bet_hash_array_size;
        let table_size = 12 + data_size; // Extended header (12 bytes) + data

        // Update header with final size
        let mut final_header = header;
        final_header.table_size = table_size;

        // Serialize everything
        let mut result = Vec::with_capacity((12 + data_size) as usize);

        // Write extended header first
        result.write_u32_le(0x1A544542)?; // "BET\x1A"
        result.write_u32_le(1)?; // version
        result.write_u32_le(data_size)?; // data_size

        // Then write the BET header
        result.write_u32_le(final_header.table_size)?;
        result.write_u32_le(final_header.file_count)?;
        result.write_u32_le(final_header.unknown_08)?;
        result.write_u32_le(final_header.table_entry_size)?;
        result.write_u32_le(final_header.bit_index_file_pos)?;
        result.write_u32_le(final_header.bit_index_file_size)?;
        result.write_u32_le(final_header.bit_index_cmp_size)?;
        result.write_u32_le(final_header.bit_index_flag_index)?;
        result.write_u32_le(final_header.bit_index_unknown)?;
        result.write_u32_le(final_header.bit_count_file_pos)?;
        result.write_u32_le(final_header.bit_count_file_size)?;
        result.write_u32_le(final_header.bit_count_cmp_size)?;
        result.write_u32_le(final_header.bit_count_flag_index)?;
        result.write_u32_le(final_header.bit_count_unknown)?;
        result.write_u32_le(final_header.total_bet_hash_size)?;
        result.write_u32_le(final_header.bet_hash_size_extra)?;
        result.write_u32_le(final_header.bet_hash_size)?;
        result.write_u32_le(final_header.bet_hash_array_size)?;
        result.write_u32_le(final_header.flag_count)?;

        // Write flag array
        for &flags in &flag_array {
            result.write_u32_le(flags)?;
        }

        // Write file table
        result.extend_from_slice(&file_table);

        // Write BET hashes (bit-packed)
        let mut hash_bytes = vec![0u8; bet_hash_array_size as usize];
        for (i, &hash) in bet_hashes.iter().enumerate() {
            self.write_bit_entry(&mut hash_bytes, i, hash, bet_hash_size)?;
        }
        result.extend_from_slice(&hash_bytes);

        Ok((result, final_header))
    }

    /// Write BET table to the archive, returns the written size and MD5
    fn write_bet_table<W: Write>(
        &self,
        writer: &mut W,
        data: &[u8],
        encrypt: bool,
    ) -> Result<(u64, [u8; 16])> {
        // BET table structure:
        // - Extended header (12 bytes) - NEVER encrypted
        // - Table data (rest) - can be compressed and/or encrypted

        if data.len() < 12 {
            return Err(Error::invalid_format("BET table data too small"));
        }

        // Split extended header and table data
        let (extended_header, table_data) = data.split_at(12);
        let mut processed_data = table_data.to_vec();

        // Compress if enabled and this is a v3+ archive
        if self.compress_tables && matches!(self.version, FormatVersion::V3 | FormatVersion::V4) {
            log::debug!("Compressing BET table data: {} -> ", processed_data.len());
            let compressed = compress(&processed_data, self.table_compression)?;
            log::debug!(
                "{} bytes ({}% reduction)",
                compressed.len(),
                (100 * (processed_data.len() - compressed.len()) / processed_data.len())
            );

            // Prepend compression type byte
            let mut compressed_with_type = Vec::with_capacity(1 + compressed.len());
            compressed_with_type.push(self.table_compression);
            compressed_with_type.extend_from_slice(&compressed);
            processed_data = compressed_with_type;
        }

        // Encrypt the data portion (after extended header)
        if encrypt {
            let key = hash_string("(block table)", hash_type::FILE_KEY);
            self.encrypt_data(&mut processed_data, key);
        }

        // Combine extended header with processed data
        let mut final_data = Vec::with_capacity(extended_header.len() + processed_data.len());
        final_data.extend_from_slice(extended_header);
        final_data.extend_from_slice(&processed_data);

        // Calculate MD5 of final data
        let md5 = self.calculate_md5(&final_data);

        let written_size = final_data.len() as u64;
        writer.write_all(&final_data)?;
        Ok((written_size, md5))
    }
}

impl Default for ArchiveBuilder {
    fn default() -> Self {
        Self::new()
    }
}
