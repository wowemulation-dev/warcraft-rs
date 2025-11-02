//! MPQ patch file support
//!
//! This module implements support for binary patch files (PTCH format) used in
//! Cataclysm and later WoW expansions. Patch files contain binary diffs that must
//! be applied to base files to reconstruct the final file.
//!
//! # Automatic Patch Handling
//!
//! **In most cases, you don't need to use this module directly.** The [`PatchChain`](crate::PatchChain)
//! automatically detects and applies patch files when reading files from a chain of archives.
//!
//! ```rust,no_run
//! use wow_mpq::PatchChain;
//!
//! let mut chain = PatchChain::new();
//! chain.add_archive("base.MPQ", 0)?;
//! chain.add_archive("patch-1.MPQ", 100)?;
//!
//! // If the file exists as a patch in patch-1.MPQ, it will be automatically
//! // applied to the base file from base.MPQ
//! let data = chain.read_file("some/file.txt")?;
//! # Ok::<(), wow_mpq::Error>(())
//! ```
//!
//! # Patch File Format
//!
//! Patch files use the PTCH format with three main sections:
//!
//! 1. **PTCH Header** - Basic metadata (sizes, signatures)
//! 2. **MD5 Block** - Checksums for verification
//! 3. **XFRM Block** - Patch data (COPY or BSD0 type)
//!
//! # Patch Types
//!
//! - **COPY** (`0x59504f43`) - Simple file replacement
//! - **BSD0** (`0x30445342`) - Binary diff using bsdiff40 algorithm
//!
//! # Manual Patch Application
//!
//! For advanced use cases where you need to manually parse and apply patches:
//!
//! ```rust,no_run
//! use wow_mpq::patch::{PatchFile, apply_patch};
//!
//! // Read patch file data (bypasses normal Archive::read_file rejection)
//! let patch_data = /* get patch data from archive */
//! # vec![0u8; 100];
//! let patch = PatchFile::parse(&patch_data)?;
//!
//! // Apply patch to base file
//! let base_data = /* get base file data */
//! # vec![0u8; 100];
//! let patched = apply_patch(&patch, &base_data)?;
//! # Ok::<(), wow_mpq::Error>(())
//! ```

mod apply;
mod header;

pub use apply::apply_patch;
pub use header::{PatchFile, PatchHeader, PatchType};
