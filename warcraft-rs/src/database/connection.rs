//! Database connection management using turso (async SQLite)

use directories::ProjectDirs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::schema;

/// Errors that can occur during database operations
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Database error from turso
    #[error("Database error: {0}")]
    Database(#[from] turso::Error),

    /// Filesystem I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// User's home directory could not be determined
    #[error("Home directory not found")]
    NoHomeDirectory,
}

pub(super) type Result<T> = std::result::Result<T, DatabaseError>;

/// Database connection wrapper.
///
/// Holds both the `turso::Database` (which must outlive the connection) and
/// a `turso::Connection` used for all queries.
#[derive(Debug)]
pub struct Database {
    /// The turso database handle -- must outlive `conn`.
    _db: turso::Database,
    conn: turso::Connection,
    path: PathBuf,
}

impl Database {
    /// Open or create a database at the default location
    pub async fn open_default() -> Result<Self> {
        let path = Self::default_path()?;
        Self::open(&path).await
    }

    /// Open or create a database at the specified path
    pub async fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = turso::Builder::new_local(&path.to_string_lossy())
            .build()
            .await?;
        let conn = db.connect()?;

        schema::init_schema(&conn).await?;

        Ok(Self {
            _db: db,
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
    pub fn connection(&self) -> &turso::Connection {
        &self.conn
    }

    /// Get the database file path
    pub fn path(&self) -> &Path {
        &self.path
    }
}
