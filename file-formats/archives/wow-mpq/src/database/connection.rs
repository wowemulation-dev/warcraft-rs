//! Database connection management

use directories::ProjectDirs;
use rusqlite::{Connection, Result as SqlResult};
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::schema;

/// Errors that can occur during database operations
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// SQLite database error
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Filesystem I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// User's home directory could not be determined
    #[error("Home directory not found")]
    NoHomeDirectory,
}

pub(super) type Result<T> = std::result::Result<T, DatabaseError>;

/// Database connection wrapper
#[derive(Debug)]
pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    /// Open or create a database at the default location
    pub fn open_default() -> Result<Self> {
        let path = Self::default_path()?;
        Self::open(&path)
    }

    /// Open or create a database at the specified path
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        schema::init_schema(&conn)?;

        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }

    /// Get the default database path
    pub fn default_path() -> Result<PathBuf> {
        if let Some(proj_dirs) = ProjectDirs::from("network", "kogito", "warcraft-rs") {
            let data_dir = proj_dirs.data_dir();
            Ok(data_dir.join("mpq-hashes.db"))
        } else {
            Err(DatabaseError::NoHomeDirectory)
        }
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get a mutable reference to the underlying connection
    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Get the database file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Begin a transaction
    pub fn transaction(&mut self) -> SqlResult<rusqlite::Transaction<'_>> {
        self.conn.transaction()
    }

    /// Execute a batch of SQL statements
    pub fn execute_batch(&self, sql: &str) -> SqlResult<()> {
        self.conn.execute_batch(sql)
    }
}
