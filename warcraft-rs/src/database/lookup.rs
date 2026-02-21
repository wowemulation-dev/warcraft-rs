//! Hash lookup functionality

use wow_mpq::{calculate_het_hashes, calculate_mpq_hashes};

use super::Database;

type Result<T> = super::connection::Result<T>;

// These traits are only used within this crate, so auto-trait bounds on the
// returned futures are not a concern.

/// Trait for traditional MPQ hash lookup operations
#[allow(async_fn_in_trait, dead_code)]
pub trait HashLookup {
    /// Look up a filename by its traditional MPQ hash values
    async fn lookup_filename(&self, hash_a: u32, hash_b: u32) -> Result<Option<String>>;

    /// Look up multiple filenames by their hash values
    async fn lookup_filenames(
        &self,
        hashes: &[(u32, u32)],
    ) -> Result<Vec<(u32, u32, Option<String>)>>;

    /// Store a filename and calculate all its hashes
    async fn store_filename(&self, filename: &str, source: Option<&str>) -> Result<()>;

    /// Store multiple filenames and return (new_entries, updated_entries)
    async fn store_filenames(&self, filenames: &[(&str, Option<&str>)]) -> Result<(usize, usize)>;

    /// Check if a filename exists in the database
    async fn filename_exists(&self, filename: &str) -> Result<bool>;

    /// Get hashes that have no known filename in the database
    async fn get_unknown_hashes(&self, hashes: &[(u32, u32)]) -> Result<Vec<(u32, u32)>>;
}

/// Trait for HET hash lookup operations
#[allow(async_fn_in_trait, dead_code)]
pub trait HetHashLookup {
    /// Look up a filename by its HET hash values
    async fn lookup_filename_het(
        &self,
        file_hash: u64,
        name_hash: u64,
        hash_bits: u8,
    ) -> Result<Option<String>>;

    /// Look up multiple filenames by their HET hash values
    async fn lookup_filenames_het(
        &self,
        hashes: &[(u64, u64)],
        hash_bits: u8,
    ) -> Result<Vec<(u64, u64, Option<String>)>>;
}

impl HashLookup for Database {
    async fn lookup_filename(&self, hash_a: u32, hash_b: u32) -> Result<Option<String>> {
        let conn = self.connection();
        let mut rows = conn
            .query(
                "SELECT filename FROM filenames WHERE hash_a = ?1 AND hash_b = ?2 LIMIT 1",
                turso::params![i64::from(hash_a), i64::from(hash_b)],
            )
            .await?;

        match rows.next().await? {
            Some(row) => Ok(Some(row.get::<String>(0)?)),
            None => Ok(None),
        }
    }

    async fn lookup_filenames(
        &self,
        hashes: &[(u32, u32)],
    ) -> Result<Vec<(u32, u32, Option<String>)>> {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(hashes.len());

        for &(hash_a, hash_b) in hashes {
            let filename = self.lookup_filename(hash_a, hash_b).await?;
            results.push((hash_a, hash_b, filename));
        }

        Ok(results)
    }

    async fn store_filename(&self, filename: &str, source: Option<&str>) -> Result<()> {
        let (hash_a, hash_b, hash_offset) = calculate_mpq_hashes(filename);

        // Calculate HET hashes for common bit sizes
        let het_40 = calculate_het_hashes(filename, 40);
        let het_48 = calculate_het_hashes(filename, 48);
        let het_56 = calculate_het_hashes(filename, 56);
        let het_64 = calculate_het_hashes(filename, 64);

        self.connection()
            .execute(
                "INSERT OR REPLACE INTO filenames (
                    filename, hash_a, hash_b, hash_offset,
                    het_hash_40_file, het_hash_40_name,
                    het_hash_48_file, het_hash_48_name,
                    het_hash_56_file, het_hash_56_name,
                    het_hash_64_file, het_hash_64_name,
                    source
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                turso::params![
                    filename,
                    i64::from(hash_a),
                    i64::from(hash_b),
                    i64::from(hash_offset),
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
            )
            .await?;

        Ok(())
    }

    async fn store_filenames(
        &self,
        filenames: &[(&str, Option<&str>)],
    ) -> Result<(usize, usize)> {
        if filenames.is_empty() {
            return Ok((0, 0));
        }

        let conn = self.connection();
        let mut new_entries = 0;
        let mut updated_entries = 0;

        for &(filename, source) in filenames {
            // Check if entry exists
            let mut rows = conn
                .query(
                    "SELECT 1 FROM filenames WHERE filename = ?1",
                    turso::params![filename],
                )
                .await?;
            let exists = rows.next().await?.is_some();

            let (hash_a, hash_b, hash_offset) = calculate_mpq_hashes(filename);
            let het_40 = calculate_het_hashes(filename, 40);
            let het_48 = calculate_het_hashes(filename, 48);
            let het_56 = calculate_het_hashes(filename, 56);
            let het_64 = calculate_het_hashes(filename, 64);

            conn.execute(
                "INSERT OR REPLACE INTO filenames (
                    filename, hash_a, hash_b, hash_offset,
                    het_hash_40_file, het_hash_40_name,
                    het_hash_48_file, het_hash_48_name,
                    het_hash_56_file, het_hash_56_name,
                    het_hash_64_file, het_hash_64_name,
                    source
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                turso::params![
                    filename,
                    i64::from(hash_a),
                    i64::from(hash_b),
                    i64::from(hash_offset),
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
            )
            .await?;

            if exists {
                updated_entries += 1;
            } else {
                new_entries += 1;
            }
        }

        Ok((new_entries, updated_entries))
    }

    async fn filename_exists(&self, filename: &str) -> Result<bool> {
        let (hash_a, hash_b, _) = calculate_mpq_hashes(filename);

        let mut rows = self
            .connection()
            .query(
                "SELECT COUNT(*) FROM filenames WHERE hash_a = ?1 AND hash_b = ?2",
                turso::params![i64::from(hash_a), i64::from(hash_b)],
            )
            .await?;

        match rows.next().await? {
            Some(row) => {
                let count: i64 = row.get(0)?;
                Ok(count > 0)
            }
            None => Ok(false),
        }
    }

    async fn get_unknown_hashes(&self, hashes: &[(u32, u32)]) -> Result<Vec<(u32, u32)>> {
        let mut unknown = Vec::new();

        for &(hash_a, hash_b) in hashes {
            if self.lookup_filename(hash_a, hash_b).await?.is_none() {
                unknown.push((hash_a, hash_b));
            }
        }

        Ok(unknown)
    }
}

impl HetHashLookup for Database {
    async fn lookup_filename_het(
        &self,
        file_hash: u64,
        name_hash: u64,
        hash_bits: u8,
    ) -> Result<Option<String>> {
        let (column_file, column_name) = match hash_bits {
            40 => ("het_hash_40_file", "het_hash_40_name"),
            48 => ("het_hash_48_file", "het_hash_48_name"),
            56 => ("het_hash_56_file", "het_hash_56_name"),
            64 => ("het_hash_64_file", "het_hash_64_name"),
            _ => return Ok(None),
        };

        let query = format!(
            "SELECT filename FROM filenames WHERE {column_file} = ?1 AND {column_name} = ?2 LIMIT 1"
        );

        let mut rows = self
            .connection()
            .query(&query, turso::params![file_hash as i64, name_hash as i64])
            .await?;

        match rows.next().await? {
            Some(row) => Ok(Some(row.get::<String>(0)?)),
            None => Ok(None),
        }
    }

    async fn lookup_filenames_het(
        &self,
        hashes: &[(u64, u64)],
        hash_bits: u8,
    ) -> Result<Vec<(u64, u64, Option<String>)>> {
        if hashes.is_empty() {
            return Ok(Vec::new());
        }

        let valid_bits = matches!(hash_bits, 40 | 48 | 56 | 64);
        if !valid_bits {
            return Ok(hashes.iter().map(|_| (0, 0, None)).collect());
        }

        let mut results = Vec::with_capacity(hashes.len());

        for &(file_hash, name_hash) in hashes {
            let filename = self
                .lookup_filename_het(file_hash, name_hash, hash_bits)
                .await?;
            results.push((file_hash, name_hash, filename));
        }

        Ok(results)
    }
}
