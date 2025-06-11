//! Error types for the DBC parser.

use std::io;
use thiserror::Error;

/// Errors that can occur when parsing a DBC file
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error occurred
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Invalid DBC header
    #[error("Invalid DBC header: {0}")]
    InvalidHeader(String),

    /// Invalid DBC record
    #[error("Invalid DBC record: {0}")]
    InvalidRecord(String),

    /// Invalid string block
    #[error("Invalid string block: {0}")]
    InvalidStringBlock(String),

    /// Schema validation error
    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    /// Out of bounds error
    #[error("Out of bounds: {0}")]
    OutOfBounds(String),

    /// Type conversion error
    #[error("Type conversion error: {0}")]
    TypeConversion(String),
}
