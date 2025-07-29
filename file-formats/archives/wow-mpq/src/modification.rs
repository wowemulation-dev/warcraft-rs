//! Archive modification support
//!
//! This module provides functionality for modifying existing MPQ archives,
//! including adding, removing, and renaming files, as well as compacting
//! archives to reclaim space from deleted files.

use crate::{
    Archive, ArchiveBuilder, Error, ListfileOption, Result,
    compression::{self, CompressionMethod, compress},
    crypto::{encrypt_block, hash_string, hash_type},
    header::FormatVersion,
    special_files::{AttributeFlags, Attributes, FileAttributes},
    tables::{BetHeader, BlockEntry, BlockTable, HashEntry, HashTable, HetHeader, HiBlockTable},
};
use bytes::Bytes;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Options for adding files to an archive
#[derive(Debug, Clone)]
pub struct AddFileOptions {
    /// Compression method to use
    pub compression: CompressionMethod,
    /// Whether to encrypt the file
    pub encrypt: bool,
    /// Whether to use FIX_KEY encryption (adjusts key by block position)
    pub fix_key: bool,
    /// Whether to replace existing file (default: true)
    pub replace_existing: bool,
    /// Locale code (default: 0 = neutral)
    pub locale: u16,
    /// Platform code (default: 0 = all)
    pub platform: u8,
}

impl Default for AddFileOptions {
    fn default() -> Self {
        Self {
            compression: CompressionMethod::Zlib,
            encrypt: false,
            fix_key: false,
            replace_existing: true,
            locale: 0,
            platform: 0,
        }
    }
}

impl AddFileOptions {
    /// Create new default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set compression method
    pub fn compression(mut self, method: CompressionMethod) -> Self {
        self.compression = method;
        self
    }

    /// Enable encryption
    pub fn encrypt(mut self) -> Self {
        self.encrypt = true;
        self
    }

    /// Enable FIX_KEY encryption
    pub fn fix_key(mut self) -> Self {
        self.fix_key = true;
        self.encrypt = true; // FIX_KEY implies encryption
        self
    }

    /// Set whether to replace existing files
    pub fn replace_existing(mut self, replace: bool) -> Self {
        self.replace_existing = replace;
        self
    }

    /// Set locale code
    pub fn locale(mut self, locale: u16) -> Self {
        self.locale = locale;
        self
    }
}

/// A mutable handle to an MPQ archive that supports modification operations
#[derive(Debug)]
pub struct MutableArchive {
    /// Path to the archive file
    _path: PathBuf,
    /// The underlying read-only archive (we'll need to make parts mutable)
    archive: Archive,
    /// File handle opened for read/write
    file: File,
    /// Cached mutable hash table
    hash_table: Option<HashTable>,
    /// Cached mutable block table
    block_table: Option<BlockTable>,
    /// Cached mutable hi-block table
    _hi_block_table: Option<HiBlockTable>,
    /// Whether changes have been made
    dirty: bool,
    /// Track next available file offset to prevent overlaps
    next_file_offset: Option<u64>,
    /// Track block table reuse for special files
    _special_file_blocks: HashMap<String, u32>,
    /// Track whether attributes need updating
    attributes_dirty: bool,
    /// Track modified blocks for CRC calculation (block_index -> filename)
    modified_blocks: HashMap<u32, String>,
    /// Updated HET table position for V3+ archives
    updated_het_pos: Option<u64>,
    /// Updated BET table position for V3+ archives  
    updated_bet_pos: Option<u64>,
    /// Updated hash table position for V3+ archives
    updated_hash_table_pos: Option<u64>,
    /// Updated block table position for V3+ archives
    updated_block_table_pos: Option<u64>,
}

impl MutableArchive {
    /// Open an archive for modification
    ///
    /// This opens the archive file with read/write permissions and loads
    /// the existing archive structure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_mpq::MutableArchive;
    ///
    /// let mut archive = MutableArchive::open("data.mpq")?;
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Open the archive for reading first
        let archive = Archive::open(&path)?;

        // Open file for read/write
        let file = OpenOptions::new().read(true).write(true).open(&path)?;

        Ok(Self {
            _path: path,
            archive,
            file,
            hash_table: None,
            block_table: None,
            _hi_block_table: None,
            dirty: false,
            next_file_offset: None,
            _special_file_blocks: HashMap::new(),
            attributes_dirty: false,
            modified_blocks: HashMap::new(),
            updated_het_pos: None,
            updated_bet_pos: None,
            updated_hash_table_pos: None,
            updated_block_table_pos: None,
        })
    }

    /// Get immutable access to the underlying archive
    ///
    /// This allows reading files and querying archive information while
    /// the archive is open for modification.
    pub fn archive(&self) -> &Archive {
        &self.archive
    }

    /// Get mutable access to the underlying archive
    ///
    /// This allows operations that require mutable access such as reading files
    /// and listing entries. This is safe because MutableArchive has exclusive
    /// ownership of the Archive.
    pub fn archive_mut(&mut self) -> &mut Archive {
        &mut self.archive
    }

    /// Debug helper to check internal state
    pub fn debug_state(&self) -> (Option<usize>, Option<usize>) {
        let block_count = self.block_table.as_ref().map(|t| t.entries().len());
        let hash_count = self.hash_table.as_ref().map(|t| t.size());
        (block_count, hash_count)
    }

    /// Read a file from the archive
    ///
    /// This method checks the modified state first, then falls back to the original archive.
    /// This ensures that renamed files can still be read correctly.
    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>> {
        // Try to read using the current modified state first
        match self.read_current_file(name) {
            Ok(data) => Ok(data),
            Err(Error::FileNotFound(_)) => {
                // Fall back to original archive if file not found in modified state
                self.archive.read_file(name)
            }
            Err(e) => Err(e),
        }
    }

    /// List files in the archive
    ///
    /// This is a convenience method that delegates to the underlying Archive.
    /// It allows listing files from a MutableArchive without having to call
    /// archive_mut() explicitly.
    pub fn list(&mut self) -> Result<Vec<crate::FileEntry>> {
        self.archive.list()
    }

    /// Find a file in the archive
    ///
    /// This method checks the modified state first, then falls back to the original archive.
    /// This ensures that removed/renamed files are handled correctly.
    pub fn find_file(&mut self, name: &str) -> Result<Option<crate::FileInfo>> {
        // Normalize the filename
        let normalized_name = name.replace('/', "\\");

        // Load tables if not already cached
        self.ensure_tables_loaded()?;

        // Check if file exists in our modified state
        match self.find_file_entry(&normalized_name)? {
            Some((hash_index, hash_entry)) => {
                // File exists in modified state, get block info
                let block_table = self
                    .block_table
                    .as_ref()
                    .or_else(|| self.archive.block_table())
                    .ok_or_else(|| Error::InvalidFormat("No block table".to_string()))?;

                let block_index = hash_entry.block_index as usize;
                if let Some(block_entry) = block_table.entries().get(block_index) {
                    let file_pos = self.archive.archive_offset() + block_entry.file_pos as u64;
                    Ok(Some(crate::FileInfo {
                        filename: normalized_name,
                        hash_index,
                        block_index,
                        file_pos,
                        compressed_size: block_entry.compressed_size as u64,
                        file_size: block_entry.file_size as u64,
                        flags: block_entry.flags,
                        locale: hash_entry.locale,
                    }))
                } else {
                    Ok(None)
                }
            }
            None => {
                // File doesn't exist in modified state - it may have been deleted
                Ok(None)
            }
        }
    }

    /// Verify the digital signature of the archive
    ///
    /// This is a convenience method that delegates to the underlying Archive.
    pub fn verify_signature(&mut self) -> Result<crate::SignatureStatus> {
        self.archive.verify_signature()
    }

    /// Load and cache attributes from the (attributes) file
    ///
    /// This is a convenience method that delegates to the underlying Archive.
    pub fn load_attributes(&mut self) -> Result<()> {
        self.archive.load_attributes()
    }

    /// Add a file from disk to the archive
    ///
    /// # Parameters
    /// - `source_path`: Path to the file on disk to add
    /// - `archive_name`: Name for the file within the archive
    /// - `options`: Options controlling compression, encryption, etc.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wow_mpq::{MutableArchive, AddFileOptions};
    ///
    /// let mut archive = MutableArchive::open("data.mpq")?;
    ///
    /// // Add with default options
    /// archive.add_file("texture.blp", "Textures\\new.blp", Default::default())?;
    ///
    /// // Add with custom options
    /// let options = AddFileOptions::new()
    ///     .compression(wow_mpq::compression::CompressionMethod::Lzma)
    ///     .encrypt();
    /// archive.add_file("model.m2", "Models\\character.m2", options)?;
    /// # Ok::<(), wow_mpq::Error>(())
    /// ```
    pub fn add_file<P: AsRef<Path>>(
        &mut self,
        source_path: P,
        archive_name: &str,
        options: AddFileOptions,
    ) -> Result<()> {
        // Read file data
        let mut file_data = Vec::new();
        File::open(source_path)?.read_to_end(&mut file_data)?;

        self.add_file_data(&file_data, archive_name, options)
    }

    /// Add a file from memory to the archive
    ///
    /// # Parameters
    /// - `data`: File data to add
    /// - `archive_name`: Name for the file within the archive
    /// - `options`: Options controlling compression, encryption, etc.
    pub fn add_file_data(
        &mut self,
        data: &[u8],
        archive_name: &str,
        options: AddFileOptions,
    ) -> Result<()> {
        // Normalize the archive name (convert forward slashes to backslashes)
        let archive_name = archive_name.replace('/', "\\");

        // Load tables if not already cached
        self.ensure_tables_loaded()?;

        // Check if we're updating a special file (listfile/attributes)
        let is_internal_update = archive_name == "(listfile)" || archive_name == "(attributes)";

        // Check if file exists and if we should replace it
        let existing_block_index =
            if let Some((hash_index, entry)) = self.find_file_entry(&archive_name)? {
                if !options.replace_existing {
                    return Err(Error::FileExists(archive_name));
                }
                // Mark the existing entry as deleted for now
                if let Some(hash_table) = &mut self.hash_table {
                    hash_table.get_mut(hash_index).unwrap().block_index = HashEntry::EMPTY_DELETED;
                }

                // If this is a special file update, remember its block index for reuse
                if is_internal_update {
                    Some(entry.block_index)
                } else {
                    None
                }
            } else {
                None
            };

        // Determine block index - reuse for special files, allocate new for regular files
        let block_index = if let Some(existing_idx) = existing_block_index {
            existing_idx
        } else {
            self.block_table.as_ref().unwrap().entries().len() as u32
        };

        // Find where to place the file (append to end for now)
        let file_offset = self.get_archive_end_offset()?;

        // Compress the file data if requested
        let (compressed_data, compressed_size, flags) =
            self.prepare_file_data(data, &archive_name, &options)?;

        // Write the file data to the archive
        self.file.seek(SeekFrom::Start(file_offset))?;
        self.file.write_all(&compressed_data)?;

        // Update next file offset for subsequent files in this session
        let next_offset = file_offset + compressed_data.len() as u64;
        let aligned_next = (next_offset + 511) & !511; // Align to 512-byte boundary
        self.next_file_offset = Some(aligned_next);

        // Add block table entry
        let relative_pos = (file_offset - self.archive.archive_offset()) as u32;
        let block_entry = BlockEntry {
            file_pos: relative_pos,
            compressed_size: compressed_size as u32,
            file_size: data.len() as u32, // Original unpadded size
            flags,
        };

        // Add or update block table entry
        if let Some(block_table) = &mut self.block_table {
            if existing_block_index.is_some() {
                // Update existing entry
                if let Some(entry) = block_table.get_mut(block_index as usize) {
                    *entry = block_entry;
                }
            } else {
                // Add new entry
                let old_entries = block_table.entries();
                let new_size = old_entries.len() + 1;
                let mut new_table = BlockTable::new_mut(new_size)?;

                // Copy old entries
                for (i, entry) in old_entries.iter().enumerate() {
                    if let Some(new_entry) = new_table.get_mut(i) {
                        *new_entry = *entry;
                    }
                }

                // Add new entry
                if let Some(new_entry) = new_table.get_mut(new_size - 1) {
                    *new_entry = block_entry;
                }

                *block_table = new_table;
            }
        }

        // Add to hash table
        self.add_to_hash_table(&archive_name, block_index, options.locale)?;

        // Track this block as modified (for attributes CRC calculation)
        if archive_name != "(attributes)" {
            self.modified_blocks
                .insert(block_index, archive_name.clone());
        }

        // Update (listfile) if present (but not if we're adding the listfile itself)
        if archive_name != "(listfile)" && !is_internal_update {
            self.update_listfile(&archive_name)?;
        }

        // Mark attributes as needing update (unless we're updating attributes itself)
        if archive_name != "(attributes)" {
            self.attributes_dirty = true;
        }

        self.dirty = true;
        Ok(())
    }

    /// Read the current state of a file (considering modifications)
    fn read_current_file(&mut self, filename: &str) -> Result<Vec<u8>> {
        // First check if we have a modified version
        if let Some((_, entry)) = self.find_file_entry(filename)? {
            let block_idx = entry.block_index as usize;
            if let Some(block_table) = &self.block_table {
                if let Some(block) = block_table.entries().get(block_idx) {
                    // Read from our file handle
                    let file_pos = self.archive.archive_offset() + block.file_pos as u64;
                    self.file.seek(SeekFrom::Start(file_pos))?;

                    let mut data = vec![0u8; block.compressed_size as usize];
                    self.file.read_exact(&mut data)?;

                    // Handle decompression/decryption if needed
                    // For now, assume (listfile) is uncompressed/unencrypted
                    if block.is_compressed() || block.is_encrypted() {
                        // This would need proper decompression/decryption
                        return self.archive.read_file(filename);
                    }

                    // Truncate to actual file size
                    data.truncate(block.file_size as usize);
                    return Ok(data);
                }
            }
        }

        // Fall back to original archive
        self.archive.read_file(filename)
    }

    /// Remove a file from the archive
    ///
    /// This marks the file as deleted in the hash table. The space is not
    /// reclaimed until `compact()` is called.
    ///
    /// # Parameters
    /// - `archive_name`: Name of the file to remove
    pub fn remove_file(&mut self, archive_name: &str) -> Result<()> {
        // Normalize the archive name
        let archive_name = archive_name.replace('/', "\\");

        // Load tables if not already cached
        self.ensure_tables_loaded()?;

        // Find the file entry
        let (hash_index, _) = self
            .find_file_entry(&archive_name)?
            .ok_or_else(|| Error::FileNotFound(archive_name.clone()))?;

        // Mark as deleted in hash table
        if let Some(hash_table) = &mut self.hash_table {
            hash_table.get_mut(hash_index).unwrap().block_index = HashEntry::EMPTY_DELETED;
        }

        // Update (listfile) to remove the filename
        self.remove_from_listfile(&archive_name)?;

        // Mark attributes as needing update
        self.attributes_dirty = true;

        self.dirty = true;
        Ok(())
    }

    /// Rename a file in the archive
    ///
    /// # Parameters
    /// - `old_name`: Current name of the file
    /// - `new_name`: New name for the file
    pub fn rename_file(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        // Normalize names
        let old_name = old_name.replace('/', "\\");
        let new_name = new_name.replace('/', "\\");

        // Load tables if not already cached
        self.ensure_tables_loaded()?;

        // Check if source exists
        let (old_hash_index, old_entry) = self
            .find_file_entry(&old_name)?
            .ok_or_else(|| Error::FileNotFound(old_name.clone()))?;

        // Check if destination already exists
        if self.find_file_entry(&new_name)?.is_some() {
            return Err(Error::FileExists(new_name));
        }

        // Get the block index from the old entry
        let block_index = old_entry.block_index;
        let locale = old_entry.locale;

        // Remove old hash entry
        if let Some(hash_table) = &mut self.hash_table {
            hash_table.get_mut(old_hash_index).unwrap().block_index = HashEntry::EMPTY_DELETED;
        }

        // Add new hash entry
        self.add_to_hash_table(&new_name, block_index, locale)?;

        // Update (listfile)
        self.remove_from_listfile(&old_name)?;
        self.update_listfile(&new_name)?;

        // Mark attributes as needing update
        self.attributes_dirty = true;

        self.dirty = true;
        Ok(())
    }

    /// Compact the archive to reclaim space from deleted files
    ///
    /// This creates a new archive file with all active files copied over,
    /// removing any gaps from deleted files.
    pub fn compact(&mut self) -> Result<()> {
        use std::fs;
        use tempfile::NamedTempFile;

        // Ensure tables are loaded
        self.ensure_tables_loaded()?;

        // Create a temporary file in the same directory as the archive
        let archive_dir = self
            ._path
            .parent()
            .ok_or_else(|| Error::InvalidFormat("Invalid archive path".to_string()))?;
        let temp_file = NamedTempFile::new_in(archive_dir)?;
        let temp_path = temp_file.path().to_path_buf();

        // Get archive header info
        let header = self.archive.header();
        let format_version = header.format_version;

        // Create a new archive builder
        let mut builder = ArchiveBuilder::new()
            .version(format_version)
            .listfile_option(ListfileOption::Generate);

        // First, get the list of files if available
        let file_list = self.list().ok();

        // Collect all active files
        let mut files_to_copy = Vec::new();
        if let Some(hash_table) = &self.hash_table {
            for (hash_idx, entry) in hash_table.entries().iter().enumerate() {
                if !entry.is_valid() || entry.is_deleted() {
                    continue;
                }

                let block_idx = entry.block_index as usize;
                if let Some(block_table) = &self.block_table {
                    if let Some(block) = block_table.entries().get(block_idx) {
                        // Find the file name from listfile or generate a placeholder
                        let filename = if let Some(ref list) = file_list {
                            list.iter()
                                .find(|e| {
                                    // Match by verifying the hash values
                                    let name_hash1 = hash_string(&e.name, hash_type::NAME_A);
                                    let name_hash2 = hash_string(&e.name, hash_type::NAME_B);
                                    entry.name_1 == name_hash1 && entry.name_2 == name_hash2
                                })
                                .map(|e| e.name.clone())
                        } else {
                            None
                        };

                        let filename = filename.unwrap_or_else(|| {
                            // Generate placeholder name if not found in listfile
                            format!("file_{:08X}_{:08X}", entry.name_1, entry.name_2)
                        });

                        files_to_copy.push((hash_idx, block_idx, filename, *entry, *block));
                    }
                }
            }
        }

        // Add all active files to the new archive
        for (_, _, filename, hash_entry, block_entry) in &files_to_copy {
            // Skip internal files that will be handled automatically by the builder
            if filename == "(listfile)" || filename == "(attributes)" || filename == "(signature)" {
                continue;
            }

            // Read the file data
            let file_data = match self.read_file(filename) {
                Ok(data) => data,
                Err(_) => {
                    // Skip files we can't read
                    log::warn!("Skipping file {filename} during compaction (read error)");
                    continue;
                }
            };

            // Determine compression and encryption from block flags
            let compression = if block_entry.is_compressed() {
                // Try to determine specific compression method
                // For now, default to Zlib if compressed
                compression::flags::ZLIB
            } else {
                0 // No compression
            };

            let encrypt = block_entry.is_encrypted();

            // Add file to builder
            builder = builder.add_file_data_with_options(
                file_data,
                filename,
                compression,
                encrypt,
                hash_entry.locale,
            );
        }

        // Build the new archive
        builder.build(&temp_path)?;

        // Close our current file handle by dropping the field (take ownership)
        let _ = std::mem::replace(&mut self.file, File::open(&temp_path)?);

        // Replace the original file with the compacted one
        fs::rename(&temp_path, &self._path)?;

        // Re-open the compacted archive
        self.archive = Archive::open(&self._path)?;
        self.file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self._path)?;

        // Reset cached tables
        self.hash_table = None;
        self.block_table = None;
        self._hi_block_table = None;
        self.dirty = false;
        self.next_file_offset = None;
        self.attributes_dirty = false;
        self.modified_blocks.clear();

        Ok(())
    }

    /// Flush any pending changes to disk
    ///
    /// This ensures all modifications are written to the archive file.
    /// This is automatically called when the archive is dropped.
    pub fn flush(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        // Update attributes if needed
        if self.attributes_dirty {
            self.update_attributes()?;
        }

        // Write updated tables
        self.write_tables()?;

        // Update archive header
        self.update_header()?;

        self.file.sync_all()?;
        self.dirty = false;

        Ok(())
    }

    /// Update the (attributes) file with current file information
    fn update_attributes(&mut self) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Check if (attributes) exists
        if self.archive.find_file("(attributes)")?.is_none() {
            // No attributes file to update
            return Ok(());
        }

        // Read existing attributes
        let attrs_data = self.read_current_file("(attributes)")?;
        let block_count = self
            .block_table
            .as_ref()
            .map(|t| t.entries().len())
            .unwrap_or_else(|| {
                self.archive
                    .block_table()
                    .map(|t| t.entries().len())
                    .unwrap_or(0)
            });

        let mut attrs = match Attributes::parse(&Bytes::from(attrs_data), block_count) {
            Ok(a) => a,
            Err(_) => {
                // If we can't parse existing attributes, create new ones
                Attributes {
                    version: Attributes::EXPECTED_VERSION,
                    flags: AttributeFlags::new(AttributeFlags::CRC32 | AttributeFlags::FILETIME),
                    file_attributes: vec![FileAttributes::new(); block_count],
                }
            }
        };

        // Ensure we have enough attribute entries
        while attrs.file_attributes.len() < block_count {
            attrs.file_attributes.push(FileAttributes::new());
        }

        // If attributes vector is larger than block count, truncate it
        if attrs.file_attributes.len() > block_count {
            attrs.file_attributes.truncate(block_count);
        }

        // Get current timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Convert to Windows FILETIME (100ns intervals since 1601-01-01)
        let filetime = (now + 11644473600) * 10_000_000;

        // Update attributes for modified files
        // First collect the files to process to avoid borrowing issues
        let modified_files: Vec<(u32, String)> = self
            .modified_blocks
            .iter()
            .map(|(&idx, name)| (idx, name.clone()))
            .collect();

        for (block_idx, filename) in modified_files {
            let block_idx = block_idx as usize;
            if block_idx >= block_count {
                continue;
            }

            // Update timestamp for modified files
            if attrs.flags.has_filetime() {
                attrs.file_attributes[block_idx].filetime = Some(filetime);
            }

            // Calculate CRC32 if enabled
            if attrs.flags.has_crc32() && filename != "(listfile)" {
                // Read the uncompressed file data to calculate CRC
                match self.read_current_file(&filename) {
                    Ok(data) => {
                        // Calculate CRC32 using standard algorithm
                        let crc = crc32fast::hash(&data);
                        attrs.file_attributes[block_idx].crc32 = Some(crc);
                    }
                    Err(_) => {
                        // If we can't read the file, keep existing CRC or set to 0
                        if attrs.file_attributes[block_idx].crc32.is_none() {
                            attrs.file_attributes[block_idx].crc32 = Some(0);
                        }
                    }
                }
            }

            // MD5 calculation would go here if we had the flag set
            if attrs.flags.has_md5() && filename != "(listfile)" {
                // For now, preserve existing MD5 or set to zeros
                if attrs.file_attributes[block_idx].md5.is_none() {
                    attrs.file_attributes[block_idx].md5 = Some([0u8; 16]);
                }
            }
        }

        // Also update timestamps for all other valid files (not modified)
        if attrs.flags.has_filetime() {
            if let Some(hash_table) = &self.hash_table {
                for entry in hash_table.entries() {
                    if !entry.is_valid() {
                        continue;
                    }

                    let block_idx = entry.block_index as usize;
                    if block_idx >= block_count {
                        continue;
                    }

                    // Only update if not already updated above
                    if !self.modified_blocks.contains_key(&entry.block_index) {
                        attrs.file_attributes[block_idx].filetime = Some(filetime);
                    }
                }
            }
        }

        // Convert attributes back to bytes
        let new_attrs_data = attrs.to_bytes()?;

        // Update the attributes file
        let options = AddFileOptions::new()
            .compression(CompressionMethod::None)
            .replace_existing(true);

        self.add_file_data(&new_attrs_data, "(attributes)", options)?;

        self.attributes_dirty = false;
        Ok(())
    }

    /// Ensure tables are loaded and cached
    fn ensure_tables_loaded(&mut self) -> Result<()> {
        if self.hash_table.is_none() {
            // Clone the hash table from the archive
            if let Some(table) = self.archive.hash_table() {
                let entries = table.entries();
                let mut new_table = HashTable::new_mut(entries.len())?;
                // Copy entries
                for (i, entry) in entries.iter().enumerate() {
                    if let Some(new_entry) = new_table.get_mut(i) {
                        *new_entry = *entry;
                    }
                }
                self.hash_table = Some(new_table);
            } else {
                return Err(Error::InvalidFormat("No hash table in archive".to_string()));
            }
        }

        if self.block_table.is_none() {
            // Clone the block table from the archive
            if let Some(table) = self.archive.block_table() {
                let entries = table.entries();
                let mut new_table = BlockTable::new_mut(entries.len())?;
                // Copy entries
                for (i, entry) in entries.iter().enumerate() {
                    if let Some(new_entry) = new_table.get_mut(i) {
                        *new_entry = *entry;
                    }
                }
                self.block_table = Some(new_table);
            } else {
                return Err(Error::InvalidFormat(
                    "No block table in archive".to_string(),
                ));
            }
        }

        // TODO: Handle hi-block table for large archives

        Ok(())
    }

    /// Find a file entry in the hash table
    fn find_file_entry(&self, archive_name: &str) -> Result<Option<(usize, HashEntry)>> {
        let hash_table = self
            .hash_table
            .as_ref()
            .or_else(|| self.archive.hash_table())
            .ok_or_else(|| Error::InvalidFormat("No hash table".to_string()))?;

        let name_hash1 = hash_string(archive_name, hash_type::NAME_A);
        let name_hash2 = hash_string(archive_name, hash_type::NAME_B);
        let start_index = hash_string(archive_name, hash_type::TABLE_OFFSET) as usize;

        let table_size = hash_table.entries().len();
        let mut index = start_index & (table_size - 1);

        // Linear probing to find the file
        loop {
            let entry = &hash_table.entries()[index];

            // Check if this is our file
            if entry.is_valid() && entry.name_1 == name_hash1 && entry.name_2 == name_hash2 {
                return Ok(Some((index, *entry)));
            }

            // If we hit an empty entry that was never used, file doesn't exist
            if entry.is_empty() {
                return Ok(None);
            }

            // Continue to next entry
            index = (index + 1) & (table_size - 1);

            // If we've wrapped around to where we started, file doesn't exist
            if index == (start_index & (table_size - 1)) {
                return Ok(None);
            }
        }
    }

    /// Get the current end offset of the archive
    fn get_archive_end_offset(&mut self) -> Result<u64> {
        // If we've already calculated the next offset, use it
        if let Some(offset) = self.next_file_offset {
            return Ok(offset);
        }

        // Calculate the end of the archive data, accounting for table positions
        let header = self.archive.header();
        let archive_offset = self.archive.archive_offset();

        // Find the maximum position used by tables
        let hash_table_end =
            archive_offset + header.get_hash_table_pos() + (header.hash_table_size * 16) as u64; // Each hash entry is 16 bytes
        let block_table_end =
            archive_offset + header.get_block_table_pos() + (header.block_table_size * 16) as u64; // Each block entry is 16 bytes

        // Also check existing file data positions
        let mut max_file_end = 0u64;
        if let Some(block_table) = self.archive.block_table() {
            for entry in block_table.entries() {
                if entry.flags & BlockEntry::FLAG_EXISTS != 0 {
                    let file_end =
                        archive_offset + entry.file_pos as u64 + entry.compressed_size as u64;
                    max_file_end = max_file_end.max(file_end);
                }
            }
        }

        // Check new files added in this session
        if let Some(block_table) = &self.block_table {
            for entry in block_table.entries() {
                if entry.flags & BlockEntry::FLAG_EXISTS != 0 {
                    let file_end =
                        archive_offset + entry.file_pos as u64 + entry.compressed_size as u64;
                    max_file_end = max_file_end.max(file_end);
                }
            }
        }

        // Return the maximum of all end positions, aligned to sector boundary (512 bytes)
        let end_offset = hash_table_end.max(block_table_end).max(max_file_end);
        let aligned_offset = (end_offset + 511) & !511; // Align to 512-byte boundary

        // Cache this for subsequent calls in the same session
        self.next_file_offset = Some(aligned_offset);

        Ok(aligned_offset)
    }

    /// Prepare file data for writing (compress and encrypt if needed)
    fn prepare_file_data(
        &self,
        data: &[u8],
        archive_name: &str,
        options: &AddFileOptions,
    ) -> Result<(Vec<u8>, usize, u32)> {
        let mut flags = BlockEntry::FLAG_EXISTS;
        let mut output_data = data.to_vec();

        // Compress if requested
        if options.compression != CompressionMethod::None {
            // Convert CompressionMethod to u8 flag
            let compression_flag = match options.compression {
                CompressionMethod::None => 0,
                CompressionMethod::Huffman => compression::flags::HUFFMAN,
                CompressionMethod::Zlib => compression::flags::ZLIB,
                CompressionMethod::Implode => compression::flags::IMPLODE,
                CompressionMethod::PKWare => compression::flags::PKWARE,
                CompressionMethod::BZip2 => compression::flags::BZIP2,
                CompressionMethod::Sparse => compression::flags::SPARSE,
                CompressionMethod::AdpcmMono => compression::flags::ADPCM_MONO,
                CompressionMethod::AdpcmStereo => compression::flags::ADPCM_STEREO,
                CompressionMethod::Lzma => compression::flags::LZMA,
                CompressionMethod::Multiple(flags) => flags,
            };

            let compressed = compress(data, compression_flag)?;
            if compressed.len() < data.len() {
                output_data = compressed;
                flags |= BlockEntry::FLAG_COMPRESS;
            }
        }

        // Encrypt if requested
        if options.encrypt {
            let key = if options.fix_key {
                // For FIX_KEY, we need the block position
                // This is a simplified version - real implementation would adjust by block
                hash_string(archive_name, hash_type::FILE_KEY)
            } else {
                hash_string(archive_name, hash_type::FILE_KEY)
            };

            // Remember original length before padding (reserved for future use)
            let _original_len = output_data.len();

            // Pad to 4-byte boundary for encryption
            while output_data.len() % 4 != 0 {
                output_data.push(0);
            }

            // Convert to u32s for encryption
            let mut u32_buffer: Vec<u32> = output_data
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            encrypt_block(&mut u32_buffer, key);

            // Convert back to bytes, but preserve original length info
            output_data.clear();
            for &value in &u32_buffer {
                output_data.extend_from_slice(&value.to_le_bytes());
            }

            // For encrypted files, the compressed_size should include padding,
            // but file_size should be the original unpadded size
            // This will be handled in the block entry creation

            flags |= BlockEntry::FLAG_ENCRYPTED;
            if options.fix_key {
                flags |= BlockEntry::FLAG_FIX_KEY;
            }
        }

        // For simplicity, we're treating all files as single unit for now
        // Real implementation would handle multi-sector files
        flags |= BlockEntry::FLAG_SINGLE_UNIT;

        let output_len = output_data.len();
        Ok((output_data, output_len, flags))
    }

    /// Add entry to hash table
    fn add_to_hash_table(&mut self, filename: &str, block_index: u32, locale: u16) -> Result<()> {
        let hash_table = self
            .hash_table
            .as_mut()
            .ok_or_else(|| Error::InvalidFormat("No hash table".to_string()))?;

        let table_offset = hash_string(filename, hash_type::TABLE_OFFSET);
        let name_a = hash_string(filename, hash_type::NAME_A);
        let name_b = hash_string(filename, hash_type::NAME_B);

        let table_size = hash_table.size() as u32;
        let mut index = table_offset & (table_size - 1);

        // Linear probing to find empty or deleted slot
        loop {
            let entry = hash_table.get_mut(index as usize).ok_or_else(|| {
                Error::InvalidFormat("Hash table index out of bounds".to_string())
            })?;

            if entry.is_empty() || entry.is_deleted() {
                // Found empty or deleted slot
                *entry = HashEntry {
                    name_1: name_a,
                    name_2: name_b,
                    locale,
                    platform: 0, // Always 0 - platform codes are vestigial
                    block_index,
                };
                break;
            }

            // Move to next slot
            index = (index + 1) & (table_size - 1);
        }

        Ok(())
    }

    /// Update the (listfile) with a new filename
    fn update_listfile(&mut self, filename: &str) -> Result<()> {
        // Check if (listfile) exists
        if self.archive.find_file("(listfile)")?.is_none() {
            return Ok(()); // No listfile to update
        }

        // Read existing listfile content (from current state, not original)
        let mut current_content = match self.read_current_file("(listfile)") {
            Ok(data) => String::from_utf8_lossy(&data).to_string(),
            Err(_) => String::new(), // If can't read, start fresh
        };

        // Add new filename if not already present
        let filename_line = filename.to_string();
        if !current_content.contains(&filename_line) {
            if !current_content.ends_with('\n') && !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(&filename_line);
            current_content.push('\n');

            // Write updated listfile back
            let options = AddFileOptions::new()
                .compression(CompressionMethod::None) // Keep listfile uncompressed
                .replace_existing(true);

            self.add_file_data(current_content.as_bytes(), "(listfile)", options)?;
        }

        Ok(())
    }

    /// Remove a filename from the (listfile)
    fn remove_from_listfile(&mut self, filename: &str) -> Result<()> {
        // Check if (listfile) exists
        if self.archive.find_file("(listfile)")?.is_none() {
            return Ok(()); // No listfile to update
        }

        // Read existing listfile content (from current state, not original)
        let current_content = match self.read_current_file("(listfile)") {
            Ok(data) => String::from_utf8_lossy(&data).to_string(),
            Err(_) => return Ok(()), // If can't read, nothing to remove
        };

        // Remove the filename line
        let lines: Vec<&str> = current_content
            .lines()
            .filter(|line| line.trim() != filename)
            .collect();

        // Write updated listfile back if content changed
        let new_content = lines.join("\n");
        if new_content != current_content.trim() {
            let mut final_content = new_content;
            if !final_content.is_empty() {
                final_content.push('\n');
            }

            let options = AddFileOptions::new()
                .compression(CompressionMethod::None)
                .replace_existing(true);

            self.add_file_data(final_content.as_bytes(), "(listfile)", options)?;
        }

        Ok(())
    }

    /// Write updated tables back to the archive
    fn write_tables(&mut self) -> Result<()> {
        let header = self.archive.header();

        // For V3+ archives, we need to rebuild the entire table structure
        // to maintain the correct order: HET, BET, Hash, Block
        if header.format_version >= FormatVersion::V3 {
            return self.write_tables_v3_plus();
        }

        // For V1/V2 archives, use the original simple approach
        let archive_offset = self.archive.archive_offset();

        // Write hash table
        if let Some(hash_table) = &self.hash_table {
            let hash_table_pos = archive_offset + header.hash_table_pos as u64;
            self.file.seek(SeekFrom::Start(hash_table_pos))?;

            // Convert to bytes and encrypt
            let mut table_data = Vec::new();
            for entry in hash_table.entries() {
                table_data.extend_from_slice(&entry.name_1.to_le_bytes());
                table_data.extend_from_slice(&entry.name_2.to_le_bytes());
                table_data.extend_from_slice(&entry.locale.to_le_bytes());
                table_data.extend_from_slice(&entry.platform.to_le_bytes());
                table_data.extend_from_slice(&entry.block_index.to_le_bytes());
            }

            // Encrypt the table
            let key = hash_string("(hash table)", hash_type::FILE_KEY);
            let mut u32_buffer: Vec<u32> = table_data
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();
            encrypt_block(&mut u32_buffer, key);

            // Write back
            for &value in &u32_buffer {
                self.file.write_all(&value.to_le_bytes())?;
            }
        }

        // Write block table
        if let Some(block_table) = &self.block_table {
            let block_table_pos = archive_offset + header.block_table_pos as u64;
            self.file.seek(SeekFrom::Start(block_table_pos))?;

            // Convert to bytes and encrypt
            let mut table_data = Vec::new();
            for entry in block_table.entries() {
                table_data.extend_from_slice(&entry.file_pos.to_le_bytes());
                table_data.extend_from_slice(&entry.compressed_size.to_le_bytes());
                table_data.extend_from_slice(&entry.file_size.to_le_bytes());
                table_data.extend_from_slice(&entry.flags.to_le_bytes());
            }

            // Encrypt the table
            let key = hash_string("(block table)", hash_type::FILE_KEY);
            let mut u32_buffer: Vec<u32> = table_data
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();
            encrypt_block(&mut u32_buffer, key);

            // Write back
            for &value in &u32_buffer {
                self.file.write_all(&value.to_le_bytes())?;
            }
        }

        Ok(())
    }

    /// Write tables for V3+ archives with correct ordering
    fn write_tables_v3_plus(&mut self) -> Result<()> {
        let hash_table = self
            .hash_table
            .as_ref()
            .ok_or_else(|| Error::invalid_format("Hash table not loaded for V3+ table write"))?;
        let block_table = self
            .block_table
            .as_ref()
            .ok_or_else(|| Error::invalid_format("Block table not loaded for V3+ table write"))?;

        // Find the end of file data to start writing tables
        let current_pos = self.file.stream_position()?;
        let archive_offset = self.archive.archive_offset();

        // Write HET table first (correct order for V3+)
        let het_pos = current_pos - archive_offset;
        let (het_data, _het_header) = self.create_het_table_from_hash_table(hash_table)?;
        self.file.write_all(&het_data)?;

        // Write BET table second
        let bet_pos = self.file.stream_position()? - archive_offset;
        let (bet_data, _bet_header) = self.create_bet_table_from_block_table(block_table)?;
        self.file.write_all(&bet_data)?;

        // Write hash table third
        let hash_table_pos = self.file.stream_position()? - archive_offset;
        let mut table_data = Vec::new();
        for entry in hash_table.entries() {
            table_data.extend_from_slice(&entry.name_1.to_le_bytes());
            table_data.extend_from_slice(&entry.name_2.to_le_bytes());
            table_data.extend_from_slice(&entry.locale.to_le_bytes());
            table_data.extend_from_slice(&entry.platform.to_le_bytes());
            table_data.extend_from_slice(&entry.block_index.to_le_bytes());
        }

        // Encrypt the hash table
        let key = hash_string("(hash table)", hash_type::FILE_KEY);
        let mut u32_buffer: Vec<u32> = table_data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        encrypt_block(&mut u32_buffer, key);

        // Write encrypted hash table
        for &value in &u32_buffer {
            self.file.write_all(&value.to_le_bytes())?;
        }

        // Write block table fourth
        let block_table_pos = self.file.stream_position()? - archive_offset;
        let mut table_data = Vec::new();
        for entry in block_table.entries() {
            table_data.extend_from_slice(&entry.file_pos.to_le_bytes());
            table_data.extend_from_slice(&entry.compressed_size.to_le_bytes());
            table_data.extend_from_slice(&entry.file_size.to_le_bytes());
            table_data.extend_from_slice(&entry.flags.to_le_bytes());
        }

        // Encrypt the block table
        let key = hash_string("(block table)", hash_type::FILE_KEY);
        let mut u32_buffer: Vec<u32> = table_data
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        encrypt_block(&mut u32_buffer, key);

        // Write encrypted block table
        for &value in &u32_buffer {
            self.file.write_all(&value.to_le_bytes())?;
        }

        // Store all the updated positions for header update
        self.updated_het_pos = Some(het_pos);
        self.updated_bet_pos = Some(bet_pos);
        self.updated_hash_table_pos = Some(hash_table_pos);
        self.updated_block_table_pos = Some(block_table_pos);

        Ok(())
    }

    /// Create HET table data from hash table (simplified version of ArchiveBuilder logic)
    fn create_het_table_from_hash_table(
        &self,
        hash_table: &HashTable,
    ) -> Result<(Vec<u8>, HetHeader)> {
        use crate::crypto::het_hash;

        // Count actual files from the hash table
        let mut file_count = 0u32;
        for entry in hash_table.entries() {
            if !entry.is_empty() {
                file_count += 1;
            }
        }

        let hash_table_entries = (file_count * 2).max(16).next_power_of_two();

        // Create header
        let header = HetHeader {
            table_size: 0, // Will be calculated later
            max_file_count: file_count,
            hash_table_size: hash_table_entries,
            hash_entry_size: 8,
            total_index_size: hash_table_entries * Self::calculate_bits_needed(file_count as u64),
            index_size_extra: 0,
            index_size: Self::calculate_bits_needed(file_count as u64),
            block_table_size: 0,
        };

        let index_size = header.index_size;

        // Create hash table and file indices arrays
        let mut het_hash_table = vec![0xFFu8; hash_table_entries as usize];
        let file_indices_size = (header.total_index_size as usize).div_ceil(8);
        let mut file_indices = vec![0u8; file_indices_size];

        // Pre-fill with invalid indices
        let invalid_index = (1u64 << index_size) - 1;
        for i in 0..hash_table_entries {
            self.write_bit_entry(&mut file_indices, i as usize, invalid_index, index_size)?;
        }

        // Process files from hash table
        let mut file_index = 0;
        for entry in hash_table.entries() {
            if !entry.is_empty() {
                // Reconstruct filename from hash (this is an approximation)
                let filename = format!("file_{file_index}"); // Placeholder - in real implementation we'd need to track filenames

                let hash_bits = 8;
                let (hash, name_hash1) = het_hash(&filename, hash_bits);
                let start_index = (hash % hash_table_entries as u64) as usize;

                // Linear probing for collision resolution
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
                file_index += 1;
            }
        }

        // Build the result with extended header
        let het_header_size = std::mem::size_of::<HetHeader>();
        let data_size = het_header_size as u32 + hash_table_entries + file_indices_size as u32;
        let table_size = 12 + data_size;

        let mut final_header = header;
        final_header.table_size = table_size;

        let mut result = Vec::with_capacity((12 + data_size) as usize);

        // Write extended header
        result.extend_from_slice(&0x1A544548u32.to_le_bytes()); // "HET\x1A"
        result.extend_from_slice(&1u32.to_le_bytes()); // version
        result.extend_from_slice(&data_size.to_le_bytes()); // data_size

        // Write HET header
        result.extend_from_slice(&final_header.table_size.to_le_bytes());
        result.extend_from_slice(&final_header.max_file_count.to_le_bytes());
        result.extend_from_slice(&final_header.hash_table_size.to_le_bytes());
        result.extend_from_slice(&final_header.hash_entry_size.to_le_bytes());
        result.extend_from_slice(&final_header.total_index_size.to_le_bytes());
        result.extend_from_slice(&final_header.index_size_extra.to_le_bytes());
        result.extend_from_slice(&final_header.index_size.to_le_bytes());
        result.extend_from_slice(&final_header.block_table_size.to_le_bytes());

        // Write hash table and file indices
        result.extend_from_slice(&het_hash_table);
        result.extend_from_slice(&file_indices);

        Ok((result, final_header))
    }

    /// Create BET table data from block table (simplified version)
    fn create_bet_table_from_block_table(
        &self,
        block_table: &BlockTable,
    ) -> Result<(Vec<u8>, BetHeader)> {
        use crate::crypto::jenkins_hash;

        let file_count = block_table.entries().len() as u32;

        // Analyze block table to determine optimal bit widths (simplified)
        let bit_count_file_pos = 32; // Use full 32 bits for simplicity
        let bit_count_file_size = 32;
        let bit_count_cmp_size = 32;
        let bit_count_flag_index = 8; // Assume max 256 unique flag combinations
        let table_entry_size =
            bit_count_file_pos + bit_count_file_size + bit_count_cmp_size + bit_count_flag_index;

        let header = BetHeader {
            table_size: 0, // Will be calculated later
            file_count,
            unknown_08: 0x10,
            table_entry_size,
            bit_index_file_pos: 0,
            bit_index_file_size: bit_count_file_pos,
            bit_index_cmp_size: bit_count_file_pos + bit_count_file_size,
            bit_index_flag_index: bit_count_file_pos + bit_count_file_size + bit_count_cmp_size,
            bit_index_unknown: table_entry_size,
            bit_count_file_pos,
            bit_count_file_size,
            bit_count_cmp_size,
            bit_count_flag_index,
            bit_count_unknown: 0,
            total_bet_hash_size: file_count * 64, // 64-bit hashes
            bet_hash_size_extra: 0,
            bet_hash_size: 64,
            bet_hash_array_size: file_count * 8, // 8 bytes per 64-bit hash
            flag_count: 1,                       // Simplified: assume all files have same flags
        };

        // Create simplified BET table
        let bet_header_size = std::mem::size_of::<BetHeader>();
        let data_size = bet_header_size as u32 + 4 + (file_count * 12); // header + flag array + file table + hashes
        let table_size = 12 + data_size;

        let mut final_header = header;
        final_header.table_size = table_size;

        let mut result = Vec::with_capacity((12 + data_size) as usize);

        // Write extended header
        result.extend_from_slice(&0x1A544542u32.to_le_bytes()); // "BET\x1A"
        result.extend_from_slice(&1u32.to_le_bytes()); // version
        result.extend_from_slice(&data_size.to_le_bytes()); // data_size

        // Write BET header (simplified)
        result.extend_from_slice(&final_header.table_size.to_le_bytes());
        result.extend_from_slice(&final_header.file_count.to_le_bytes());
        result.extend_from_slice(&final_header.unknown_08.to_le_bytes());
        result.extend_from_slice(&final_header.table_entry_size.to_le_bytes());

        // Write remaining header fields (simplified)
        for _ in 0..15 {
            // Fill remaining header fields with zeros
            result.extend_from_slice(&0u32.to_le_bytes());
        }

        // Write flag array (simplified)
        result.extend_from_slice(&0u32.to_le_bytes()); // Single flag value

        // Write simplified file table and hashes
        for (i, entry) in block_table.entries().iter().enumerate() {
            result.extend_from_slice(&entry.file_pos.to_le_bytes());
            result.extend_from_slice(&entry.file_size.to_le_bytes());
            result.extend_from_slice(&entry.compressed_size.to_le_bytes());

            // Generate a hash for this file (placeholder)
            let hash = jenkins_hash(&format!("file_{i}"));
            result.extend_from_slice(&hash.to_le_bytes());
        }

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

        let bits_needed = bit_shift + bit_size as usize;
        let bytes_needed = bits_needed.div_ceil(8);

        if byte_offset + bytes_needed > data.len() {
            return Err(Error::invalid_format("Bit entry out of bounds"));
        }

        // Read existing bits
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

        // Write back
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

    /// Update the archive header
    fn update_header(&mut self) -> Result<()> {
        let archive_offset = self.archive.archive_offset();
        let mut header = self.archive.header().clone();
        let mut needs_update = false;

        // Update block table size if it has grown
        if let Some(block_table) = &self.block_table {
            let new_size = block_table.entries().len() as u32;
            if new_size != header.block_table_size {
                header.block_table_size = new_size;
                needs_update = true;
            }
        }

        if needs_update {
            // Seek to header position
            self.file.seek(SeekFrom::Start(archive_offset))?;

            // Write the header
            self.file.write_all(b"MPQ\x1A")?; // Signature
            self.file.write_all(&header.header_size.to_le_bytes())?;
            self.file.write_all(&header.archive_size.to_le_bytes())?;
            self.file
                .write_all(&(header.format_version as u16).to_le_bytes())?;
            self.file.write_all(&header.block_size.to_le_bytes())?;

            // Use updated positions if available (for V3+), otherwise use original
            let hash_pos = self
                .updated_hash_table_pos
                .unwrap_or(header.hash_table_pos as u64) as u32;
            let block_pos = self
                .updated_block_table_pos
                .unwrap_or(header.block_table_pos as u64) as u32;

            self.file.write_all(&hash_pos.to_le_bytes())?;
            self.file.write_all(&block_pos.to_le_bytes())?;
            self.file.write_all(&header.hash_table_size.to_le_bytes())?;
            self.file
                .write_all(&header.block_table_size.to_le_bytes())?;

            // Write extended fields for v2+
            if header.format_version >= FormatVersion::V2 {
                self.file
                    .write_all(&header.hi_block_table_pos.unwrap_or(0).to_le_bytes())?;
                self.file
                    .write_all(&header.hash_table_pos_hi.unwrap_or(0).to_le_bytes())?;
                self.file
                    .write_all(&header.block_table_pos_hi.unwrap_or(0).to_le_bytes())?;
            }

            // Write v3+ fields
            if header.format_version >= FormatVersion::V3 {
                self.file
                    .write_all(&header.archive_size_64.unwrap_or(0).to_le_bytes())?;

                // Use updated positions if available, otherwise use original
                let het_pos = self.updated_het_pos.or(header.het_table_pos).unwrap_or(0);
                let bet_pos = self.updated_bet_pos.or(header.bet_table_pos).unwrap_or(0);

                self.file.write_all(&het_pos.to_le_bytes())?;
                self.file.write_all(&bet_pos.to_le_bytes())?;
            }
        }

        Ok(())
    }
}

impl Drop for MutableArchive {
    fn drop(&mut self) {
        // Attempt to flush changes on drop, but ignore errors
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_file_options() {
        let options = AddFileOptions::new()
            .compression(CompressionMethod::Lzma)
            .encrypt()
            .locale(0x409); // en-US

        assert_eq!(options.compression, CompressionMethod::Lzma);
        assert!(options.encrypt);
        assert_eq!(options.locale, 0x409);
    }

    #[test]
    fn test_fix_key_enables_encryption() {
        let options = AddFileOptions::new().fix_key();
        assert!(options.encrypt);
        assert!(options.fix_key);
    }
}
