//! Compression and decompression algorithms for MPQ files

mod algorithms;
mod compress;
mod decompress;
mod methods;

// Re-export the main public API
pub use compress::compress;
pub use decompress::decompress;
pub use methods::{CompressionMethod, flags};
