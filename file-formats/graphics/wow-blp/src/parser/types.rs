pub use super::error::Error;

/// Result type for BLP parsing operations
pub type ParseResult<T> = Result<T, Error>;
