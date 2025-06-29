//! Hash lookup functionality

use rusqlite::{OptionalExtension, params};

use super::Database;
use crate::database::{calculate_het_hashes, calculate_mpq_hashes};

type Result<T> = crate::database::connection::Result<T>;

/// Trait for traditional MPQ hash lookup operations
pub trait HashLookup {
    /// Look up a filename by its traditional MPQ hash values
    fn lookup_filename(&self, hash_a: u32, hash_b: u32) -> Result<Option<String>>;

    /// Look up multiple filenames by their hash values
    fn lookup_filenames(&self, hashes: &[(u32, u32)]) -> Result<Vec<(u32, u32, Option<String>)>>;

    /// Store a filename and calculate all its hashes
    fn store_filename(&self, filename: &str, source: Option<&str>) -> Result<()>;

    /// Store multiple filenames and return (new_entries, updated_entries)
    fn store_filenames(&self, filenames: &[(&str, Option<&str>)]) -> Result<(usize, usize)>;

    /// Check if a filename exists in the database
    fn filename_exists(&self, filename: &str) -> Result<bool>;

    /// Get statistics about unknown hashes in an archive
    fn get_unknown_hashes(&self, hashes: &[(u32, u32)]) -> Result<Vec<(u32, u32)>>;
}

/// Trait for HET hash lookup operations
pub trait HetHashLookup {
    /// Look up a filename by its HET hash values
    fn lookup_filename_het(
        &self,
        file_hash: u64,
        name_hash: u64,
        hash_bits: u8,
    ) -> Result<Option<String>>;

    /// Look up multiple filenames by their HET hash values
    fn lookup_filenames_het(
        &self,
        hashes: &[(u64, u64)],
        hash_bits: u8,
    ) -> Result<Vec<(u64, u64, Option<String>)>>;
}

impl HashLookup for Database {
    fn lookup_filename(&self, hash_a: u32, hash_b: u32) -> Result<Option<String>> {
        let mut stmt = self
            .connection()
            .prepare("SELECT filename FROM filenames WHERE hash_a = ?1 AND hash_b = ?2 LIMIT 1")?;

        Ok(stmt
            .query_row(params![hash_a, hash_b], |row| row.get(0))
            .optional()?)
    }

    fn lookup_filenames(&self, hashes: &[(u32, u32)]) -> Result<Vec<(u32, u32, Option<String>)>> {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(hashes.len());
        let conn = self.connection();

        // Use a prepared statement for efficiency
        let mut stmt = conn
            .prepare("SELECT filename FROM filenames WHERE hash_a = ?1 AND hash_b = ?2 LIMIT 1")?;

        for &(hash_a, hash_b) in hashes {
            let filename: Option<String> = stmt
                .query_row(params![hash_a, hash_b], |row| row.get(0))
                .optional()?;
            results.push((hash_a, hash_b, filename));
        }

        Ok(results)
    }

    fn store_filename(&self, filename: &str, source: Option<&str>) -> Result<()> {
        let (hash_a, hash_b, hash_offset) = calculate_mpq_hashes(filename);

        // Calculate HET hashes for common bit sizes
        let het_40 = calculate_het_hashes(filename, 40);
        let het_48 = calculate_het_hashes(filename, 48);
        let het_56 = calculate_het_hashes(filename, 56);
        let het_64 = calculate_het_hashes(filename, 64);

        self.connection().execute(
            "INSERT OR REPLACE INTO filenames (
                filename, hash_a, hash_b, hash_offset,
                het_hash_40_file, het_hash_40_name,
                het_hash_48_file, het_hash_48_name,
                het_hash_56_file, het_hash_56_name,
                het_hash_64_file, het_hash_64_name,
                source
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                filename,
                hash_a,
                hash_b,
                hash_offset,
                het_40.0 as i64,
                het_40.1 as i64,
                het_48.0 as i64,
                het_48.1 as i64,
                het_56.0 as i64,
                het_56.1 as i64,
                het_64.0 as i64,
                het_64.1 as i64,
                source
            ],
        )?;

        Ok(())
    }

    fn store_filenames(&self, filenames: &[(&str, Option<&str>)]) -> Result<(usize, usize)> {
        if filenames.is_empty() {
            return Ok((0, 0));
        }

        let conn = self.connection();
        let mut new_entries = 0;
        let mut updated_entries = 0;

        // Check existing entries first
        let mut check_stmt = conn.prepare("SELECT 1 FROM filenames WHERE filename = ?1")?;

        // Use a prepared statement for insert/update
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO filenames (
                filename, hash_a, hash_b, hash_offset,
                het_hash_40_file, het_hash_40_name,
                het_hash_48_file, het_hash_48_name,
                het_hash_56_file, het_hash_56_name,
                het_hash_64_file, het_hash_64_name,
                source
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )?;

        for (filename, source) in filenames {
            // Check if entry exists
            let exists: bool = check_stmt
                .query_row(params![filename], |_| Ok(true))
                .optional()?
                .is_some();

            let (hash_a, hash_b, hash_offset) = calculate_mpq_hashes(filename);
            let het_40 = calculate_het_hashes(filename, 40);
            let het_48 = calculate_het_hashes(filename, 48);
            let het_56 = calculate_het_hashes(filename, 56);
            let het_64 = calculate_het_hashes(filename, 64);

            stmt.execute(params![
                filename,
                hash_a,
                hash_b,
                hash_offset,
                het_40.0 as i64,
                het_40.1 as i64,
                het_48.0 as i64,
                het_48.1 as i64,
                het_56.0 as i64,
                het_56.1 as i64,
                het_64.0 as i64,
                het_64.1 as i64,
                source
            ])?;

            if exists {
                updated_entries += 1;
            } else {
                new_entries += 1;
            }
        }

        Ok((new_entries, updated_entries))
    }

    fn filename_exists(&self, filename: &str) -> Result<bool> {
        let (hash_a, hash_b, _) = calculate_mpq_hashes(filename);

        let count: i64 = self.connection().query_row(
            "SELECT COUNT(*) FROM filenames WHERE hash_a = ?1 AND hash_b = ?2",
            params![hash_a, hash_b],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    fn get_unknown_hashes(&self, hashes: &[(u32, u32)]) -> Result<Vec<(u32, u32)>> {
        let mut unknown = Vec::new();

        for &(hash_a, hash_b) in hashes {
            if self.lookup_filename(hash_a, hash_b)?.is_none() {
                unknown.push((hash_a, hash_b));
            }
        }

        Ok(unknown)
    }
}

impl HetHashLookup for Database {
    fn lookup_filename_het(
        &self,
        file_hash: u64,
        name_hash: u64,
        hash_bits: u8,
    ) -> Result<Option<String>> {
        let column_file = match hash_bits {
            40 => "het_hash_40_file",
            48 => "het_hash_48_file",
            56 => "het_hash_56_file",
            64 => "het_hash_64_file",
            _ => return Ok(None), // Unsupported bit size
        };

        let column_name = match hash_bits {
            40 => "het_hash_40_name",
            48 => "het_hash_48_name",
            56 => "het_hash_56_name",
            64 => "het_hash_64_name",
            _ => return Ok(None),
        };

        let query = format!(
            "SELECT filename FROM filenames WHERE {} = ?1 AND {} = ?2 LIMIT 1",
            column_file, column_name
        );

        let mut stmt = self.connection().prepare(&query)?;

        Ok(stmt
            .query_row(params![file_hash as i64, name_hash as i64], |row| {
                row.get(0)
            })
            .optional()?)
    }

    fn lookup_filenames_het(
        &self,
        hashes: &[(u64, u64)],
        hash_bits: u8,
    ) -> Result<Vec<(u64, u64, Option<String>)>> {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }

        let column_file = match hash_bits {
            40 => "het_hash_40_file",
            48 => "het_hash_48_file",
            56 => "het_hash_56_file",
            64 => "het_hash_64_file",
            _ => return Ok(vec![(0, 0, None); hashes.len()]),
        };

        let column_name = match hash_bits {
            40 => "het_hash_40_name",
            48 => "het_hash_48_name",
            56 => "het_hash_56_name",
            64 => "het_hash_64_name",
            _ => return Ok(vec![(0, 0, None); hashes.len()]),
        };

        let mut results = Vec::with_capacity(hashes.len());
        let conn = self.connection();

        let query = format!(
            "SELECT filename FROM filenames WHERE {} = ?1 AND {} = ?2 LIMIT 1",
            column_file, column_name
        );
        let mut stmt = conn.prepare(&query)?;

        for &(file_hash, name_hash) in hashes {
            let filename: Option<String> = stmt
                .query_row(params![file_hash as i64, name_hash as i64], |row| {
                    row.get(0)
                })
                .optional()?;
            results.push((file_hash, name_hash, filename));
        }

        Ok(results)
    }
}
