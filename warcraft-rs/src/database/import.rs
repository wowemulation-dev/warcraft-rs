//! Import functionality for populating the database

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

use super::{Database, DatabaseError};
use super::lookup::HashLookup;

use wow_mpq::Archive;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Archive error: {0}")]
    Archive(#[from] wow_mpq::Error),
}

pub(super) type ImportResult<T> = std::result::Result<T, ImportError>;

/// Source types for importing filenames
#[derive(Debug, Clone, Copy)]
pub enum ImportSource {
    /// Import from a listfile (text file with one filename per line)
    Listfile,
    /// Import from an MPQ archive's internal listfile
    Archive,
    /// Scan a directory for filenames matching WoW patterns
    Directory,
}

/// Import statistics
#[derive(Debug, Default)]
pub struct ImportStats {
    pub files_processed: usize,
    pub new_entries: usize,
    pub updated_entries: usize,
    pub errors: usize,
}

/// Filename importer
#[derive(Debug)]
pub struct Importer<'a> {
    db: &'a Database,
}

impl<'a> Importer<'a> {
    /// Create a new importer
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Import filenames from a source
    pub async fn import(
        &self,
        path: &Path,
        source_type: ImportSource,
    ) -> ImportResult<ImportStats> {
        match source_type {
            ImportSource::Listfile => self.import_listfile(path).await,
            ImportSource::Archive => self.import_archive(path).await,
            ImportSource::Directory => self.import_directory(path).await,
        }
    }

    /// Import from a listfile
    async fn import_listfile(&self, path: &Path) -> ImportResult<ImportStats> {
        let mut stats = ImportStats::default();
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let source = format!("listfile:{}", path.display());

        let mut batch = Vec::new();

        for line in reader.lines() {
            stats.files_processed += 1;

            match line {
                Ok(filename) => {
                    let filename = filename.trim().to_string();
                    if !filename.is_empty() && !filename.starts_with('#') {
                        batch.push((filename, Some(source.clone())));

                        // Process in batches
                        if batch.len() >= 1000 {
                            match self.process_batch(&mut batch, &mut stats).await {
                                Ok(()) => {}
                                Err(e) => {
                                    log::error!("Error processing batch: {e}");
                                    stats.errors += batch.len();
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error reading line: {e}");
                    stats.errors += 1;
                }
            }
        }

        // Process remaining
        if !batch.is_empty() {
            self.process_batch(&mut batch, &mut stats).await?;
        }

        Ok(stats)
    }

    /// Import from an MPQ archive
    async fn import_archive(&self, path: &Path) -> ImportResult<ImportStats> {
        let mut stats = ImportStats::default();
        let mut archive = Archive::open(path)?;
        let source = format!("archive:{}", path.display());

        // Get list of files from the archive (sync I/O)
        let files = archive.list()?;
        stats.files_processed = files.len();

        let mut batch = Vec::new();

        for file in files {
            if !file.name.is_empty() && !file.name.starts_with('#') {
                batch.push((file.name, Some(source.clone())));

                if batch.len() >= 1000 {
                    self.process_batch(&mut batch, &mut stats).await?;
                }
            }
        }

        // Process remaining
        if !batch.is_empty() {
            self.process_batch(&mut batch, &mut stats).await?;
        }

        Ok(stats)
    }

    /// Import from a directory (scan for WoW file patterns)
    async fn import_directory(&self, path: &Path) -> ImportResult<ImportStats> {
        let mut stats = ImportStats::default();
        let source = format!("directory:{}", path.display());

        // Common WoW file patterns
        let patterns = [
            "**/*.blp",
            "**/*.m2",
            "**/*.wmo",
            "**/*.adt",
            "**/*.wdt",
            "**/*.wdl",
            "**/*.dbc",
            "**/*.db2",
            "**/*.anim",
            "**/*.skin",
            "**/*.bone",
            "**/*.phys",
            "**/*.skel",
            "**/*.mp3",
            "**/*.ogg",
            "**/*.wav",
            "**/*.txt",
            "**/*.xml",
            "**/*.lua",
            "**/*.toc",
            "**/*.zmp",
            "**/*.bls",
            "**/*.wfx",
            "**/*.lit",
        ];

        let mut batch = Vec::new();

        for pattern in &patterns {
            let pattern_path = path.join(pattern);
            if let Ok(entries) = glob::glob(&pattern_path.to_string_lossy()) {
                for entry in entries.filter_map(std::result::Result::ok) {
                    stats.files_processed += 1;

                    // Convert to relative path from the base directory
                    if let Ok(relative) = entry.strip_prefix(path) {
                        let filename = relative.to_string_lossy().replace('/', "\\");
                        batch.push((filename.to_string(), Some(source.clone())));

                        if batch.len() >= 1000 {
                            self.process_batch(&mut batch, &mut stats).await?;
                        }
                    }
                }
            }
        }

        // Process remaining
        if !batch.is_empty() {
            self.process_batch(&mut batch, &mut stats).await?;
        }

        Ok(stats)
    }

    /// Process a batch of filenames
    async fn process_batch(
        &self,
        batch: &mut Vec<(String, Option<String>)>,
        stats: &mut ImportStats,
    ) -> ImportResult<()> {
        let filenames: Vec<(&str, Option<&str>)> = batch
            .iter()
            .map(|(f, s)| (f.as_str(), s.as_deref()))
            .collect();

        match self.db.store_filenames(&filenames).await {
            Ok((new_count, updated_count)) => {
                stats.new_entries += new_count;
                stats.updated_entries += updated_count;
            }
            Err(e) => {
                log::error!("Error storing batch: {e}");
                stats.errors += batch.len();
                return Err(e.into());
            }
        }

        batch.clear();
        Ok(())
    }
}
