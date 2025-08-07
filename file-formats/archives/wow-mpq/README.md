# wow-mpq

A high-performance, safe Rust implementation of the MPQ (Mo'PaQ) archive format
used by World of Warcraft and other Blizzard Entertainment games.

<div align="center">

[![Crates.io Version](https://img.shields.io/crates/v/wow-mpq)](https://crates.io/crates/wow-mpq)
[![docs.rs](https://img.shields.io/docsrs/wow-mpq)](https://docs.rs/wow-mpq)
[![License](https://img.shields.io/crates/l/wow-mpq.svg)](https://github.com/wowemulation-dev/warcraft-rs#license)

</div>

## Status

✅ **Production Ready** - Feature-complete MPQ implementation with 100% StormLib bidirectional compatibility

## Overview

MPQ archives are used by World of Warcraft (versions 1.x through 5.x) to store
game assets including models, textures, sounds, and data files. This crate provides
a pure Rust implementation for reading, creating, and managing MPQ archives with
comprehensive support for all format versions and features.

## Features

- 📖 **Complete Archive Reading** - All MPQ versions (v1-v4) with full feature coverage
- 🔨 **Archive Creation** - Build new archives with full control over format and compression
- ✏️ **Archive Modification** - Add, remove, and rename files with automatic listfile/attributes updates
- 🔧 **Archive Rebuilding** - Comprehensive rebuild with format upgrades and optimization
- 🗜️ **All Compression Algorithms** - Zlib, BZip2, LZMA, Sparse, ADPCM, PKWare, Huffman
- 🔐 **Full Cryptography** - File encryption/decryption, signature verification and generation
- 🔗 **Patch Chain Support** - Complete World of Warcraft patch archive management
- 📊 **Advanced Tables** - HET/BET tables for v3+ archives with optimal compression
- 🤝 **StormLib Compatibility** - 100% bidirectional compatibility with the reference implementation
- 🚀 **High Performance** - Efficient I/O, zero-copy where possible, comprehensive benchmarks
- ⚡ **Parallel Processing** - Multi-threaded extraction and validation for better performance

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wow-mpq = "0.2.1"
```

Or use cargo add:

```bash
cargo add wow-mpq
```

## Quick Start

```rust
use wow_mpq::{Archive, ArchiveBuilder, MutableArchive, AddFileOptions};

// Read an existing archive
let mut archive = Archive::open("Data/common.MPQ")?;

// List all files
for entry in archive.list()? {
    println!("{}: {} bytes", entry.name, entry.size);
}

// Extract a file
let data = archive.read_file("Interface\\FrameXML\\GlobalStrings.lua")?;

// Create a new archive
ArchiveBuilder::new()
    .add_file_data(b"Hello, Azeroth!".to_vec(), "readme.txt")
    .version(wow_mpq::FormatVersion::V2)
    .build("my_addon.mpq")?;

// Modify an existing archive
let mut mutable = MutableArchive::open("my_addon.mpq")?;
mutable.add_file_data(b"Updated content".as_ref(), "changelog.txt", AddFileOptions::default())?;
mutable.remove_file("old_file.txt")?;
mutable.rename_file("readme.txt", "README.txt")?;
mutable.flush()?; // Save all changes
```

## Supported Versions

- ✅ **Classic** (1.12.1) - Full support including patch archives
- ✅ **The Burning Crusade** (2.4.3) - Full support with v2 features
- ✅ **Wrath of the Lich King** (3.3.5a) - Full support with LZMA compression
- ✅ **Cataclysm** (4.3.4) - Full support with HET/BET tables
- ✅ **Mists of Pandaria** (5.4.8) - Full support with v4 format

## Advanced Features

### Patch Chain Support

Handle World of Warcraft's patch archive system with automatic priority-based file
resolution:

```rust
use wow_mpq::PatchChain;

let mut chain = PatchChain::new();
chain.add_archive("Data/common.MPQ", 0)?;      // Base priority
chain.add_archive("Data/patch.MPQ", 100)?;     // Patch priority
chain.add_archive("Data/patch-2.MPQ", 200)?;  // Higher priority

// Automatically gets the highest priority version
let data = chain.read_file("DBFilesClient\\Spell.dbc")?;
```

### Archive Rebuilding

Optimize and upgrade archives with comprehensive rebuild options:

```rust
use wow_mpq::{RebuildOptions, rebuild_archive};

let options = RebuildOptions::new()
    .target_version(FormatVersion::V4)           // Upgrade to v4
    .compression(CompressionType::Zlib)          // Change compression
    .remove_signatures(true)                     // Strip signatures
    .progress_callback(|current, total| {        // Track progress
        println!("Progress: {}/{}", current, total);
    });

rebuild_archive("old.mpq", "optimized.mpq", &options)?;
```

### Digital Signatures

Verify and generate archive signatures for integrity protection:

```rust
use wow_mpq::crypto::{generate_weak_signature, SignatureInfo, WEAK_SIGNATURE_FILE_SIZE};

// Verify existing signatures
let archive = Archive::open("signed.mpq")?;
match archive.verify_signature()? {
    SignatureStatus::None => println!("No signature"),
    SignatureStatus::Weak => println!("Weak signature (512-bit RSA)"),
    SignatureStatus::Strong => println!("Strong signature (2048-bit RSA)"),
    SignatureStatus::Invalid => println!("Invalid signature!"),
}

// Generate new weak signature
let archive_data = std::fs::read("archive.mpq")?;
let sig_info = SignatureInfo::new_weak(
    0,                               // Archive start
    archive_data.len() as u64,       // Archive size
    archive_data.len() as u64,       // Signature position
    WEAK_SIGNATURE_FILE_SIZE as u64, // Signature file size
    vec![],
);

let signature = generate_weak_signature(
    std::io::Cursor::new(&archive_data),
    &sig_info
)?;
```

### Debug Utilities

The crate includes comprehensive debug utilities for analyzing MPQ archives (requires `debug-utils` feature):

```rust
use wow_mpq::{Archive, debug};

// Enable the feature in Cargo.toml:
// wow-mpq = { version = "0.2.1", features = ["debug-utils"] }

let mut archive = Archive::open("example.mpq")?;

// Dump archive structure and metadata
debug::dump_archive_structure(&mut archive)?;

// Analyze compression methods used
debug::analyze_compression_methods(&mut archive)?;

// Dump table contents (hash table, block table)
debug::dump_hash_table(&archive)?;
debug::dump_block_table(&archive)?;

// Trace file extraction with detailed debugging
let config = debug::ExtractionTraceConfig {
    show_raw_data: true,
    show_decryption: true,
    show_decompression: true,
    max_raw_bytes: 256,
};
debug::trace_file_extraction(&mut archive, "example.txt", &config)?;

// Create hex dumps of binary data
let data = archive.read_file("binary.dat")?;
let hex_config = debug::HexDumpConfig::default();
println!("{}", debug::hex_dump(&data, &hex_config));
```

Run the debug example to analyze any MPQ archive:

```bash
# Basic analysis
cargo run --example debug_archive --features debug-utils -- archive.mpq

# Trace specific file extraction
cargo run --example debug_archive --features debug-utils -- archive.mpq "(listfile)"

# Show detailed table dumps
SHOW_TABLES=1 cargo run --example debug_archive --features debug-utils -- archive.mpq
```

## Performance

The crate includes comprehensive benchmarks showing excellent performance:

- **Archive Creation**: ~50-100 MB/s depending on compression
- **File Extraction**: ~200-500 MB/s for uncompressed files
- **Hash Calculation**: ~1-2 billion hashes/second
- **Compression**: Varies by algorithm (Zlib: ~50MB/s, LZMA: ~10MB/s)

## Examples

The crate includes numerous examples demonstrating real-world usage:

- `create_archive` - Basic archive creation
- `patch_chain_demo` - Working with WoW patch archives
- `wotlk_patch_chain_demo` - Complete WotLK patch handling
- `patch_analysis` - Analyze patch archive contents
- `signature_demo` - Digital signature generation and verification
- `debug_archive` - Debug utilities for analyzing MPQ internals
- And many more...

Run examples with:

```bash
cargo run --example patch_chain_demo
```

## Limitations

While achieving 100% StormLib compatibility for core functionality, the following
performance and specialized features are not implemented:

- **Memory-mapped I/O** - Standard I/O only (sufficient for most use cases)
- **Streaming API** - No chunked reading for very large files
- **Protected Archives** - Copy-protected MPQ support (rarely used)
- **Strong Signature Generation** - Requires private key not publicly available
- **Archive Compacting** - Use rebuild instead for optimization
- **Async I/O** - Synchronous operations only


## Compatibility Notes

### Blizzard Archives

Blizzard's MPQ implementation has some quirks that this library handles gracefully:

- **Attributes File Size**: All official Blizzard MPQs have attributes files that are
  exactly 28 bytes larger than the specification. This library detects and handles
  this discrepancy automatically.
- **Path Separators**: MPQ archives use backslashes (`\`) as path separators. While
  this library accepts forward slashes for convenience, they are automatically converted
  to backslashes internally.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
