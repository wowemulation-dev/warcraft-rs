//! Database schema and migration management

use turso::Connection;

use super::connection::Result;

/// Current schema version
pub(super) const SCHEMA_VERSION: i64 = 1;

/// Initialize the database schema
pub(super) async fn init_schema(conn: &Connection) -> Result<()> {
    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON", ()).await?;

    // Create version table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )
    .await?;

    // Check current version
    let current_version: Option<i64> = {
        let mut rows = conn
            .query("SELECT MAX(version) FROM schema_version", ())
            .await?;
        match rows.next().await? {
            Some(row) => row.get::<Option<i64>>(0).ok().flatten(),
            None => None,
        }
    };

    // Apply migrations
    if current_version.is_none() || current_version.is_some_and(|v| v < 1) {
        migrate_v1(conn).await?;
    }

    Ok(())
}

/// Migration to version 1: Initial schema
async fn migrate_v1(conn: &Connection) -> Result<()> {
    // Unified table for all filename-hash mappings
    conn.execute(
        "CREATE TABLE IF NOT EXISTS filenames (
            id INTEGER PRIMARY KEY,
            filename TEXT NOT NULL UNIQUE,
            -- Traditional MPQ hashes
            hash_a INTEGER NOT NULL,
            hash_b INTEGER NOT NULL,
            hash_offset INTEGER NOT NULL,
            -- HET hashes (stored for multiple bit sizes)
            het_hash_40_file INTEGER,
            het_hash_40_name INTEGER,
            het_hash_48_file INTEGER,
            het_hash_48_name INTEGER,
            het_hash_56_file INTEGER,
            het_hash_56_name INTEGER,
            het_hash_64_file INTEGER,
            het_hash_64_name INTEGER,
            source TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )
    .await?;

    // Archive analysis results
    conn.execute(
        "CREATE TABLE IF NOT EXISTS archive_analysis (
            id INTEGER PRIMARY KEY,
            archive_path TEXT NOT NULL,
            archive_hash TEXT,
            analysis_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            mpq_version INTEGER,
            file_count INTEGER
        )",
        (),
    )
    .await?;

    // Files found in archives
    conn.execute(
        "CREATE TABLE IF NOT EXISTS archive_files (
            archive_id INTEGER REFERENCES archive_analysis(id),
            hash_a INTEGER NOT NULL,
            hash_b INTEGER NOT NULL,
            file_size INTEGER,
            compressed_size INTEGER,
            flags INTEGER,
            filename_id INTEGER REFERENCES filenames(id),
            PRIMARY KEY (archive_id, hash_a, hash_b)
        )",
        (),
    )
    .await?;

    // Create indices for fast lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_filename_hashes ON filenames(hash_a, hash_b)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_archive_files_hashes ON archive_files(hash_a, hash_b)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_archive_analysis_path ON archive_analysis(archive_path)",
        (),
    )
    .await?;

    // Indices for HET table lookups at different bit sizes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_het_40 ON filenames(het_hash_40_file, het_hash_40_name)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_het_48 ON filenames(het_hash_48_file, het_hash_48_name)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_het_56 ON filenames(het_hash_56_file, het_hash_56_name)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_het_64 ON filenames(het_hash_64_file, het_hash_64_name)",
        (),
    )
    .await?;

    // Record migration
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        turso::params![SCHEMA_VERSION],
    )
    .await?;

    Ok(())
}
