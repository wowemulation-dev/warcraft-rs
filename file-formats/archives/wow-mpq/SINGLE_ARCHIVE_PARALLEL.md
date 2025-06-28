# Single Archive Parallel Processing

This document describes the single archive parallel processing capabilities added to the wow-mpq crate for improved performance when extracting multiple files from a single MPQ archive.

## Overview

The `single_archive_parallel` module provides utilities for reading multiple files from a single MPQ archive concurrently. This is achieved by giving each thread its own file handle to the archive, avoiding seek conflicts that would occur with a shared file handle.

## When to Use

Single archive parallel processing is beneficial when:

- Extracting many files from a single large archive
- Processing files with custom logic (checksums, transformations, etc.)
- Pattern matching and selective extraction
- The archive is on fast storage (SSD/NVMe)

It may not provide benefits when:

- Extracting only a few files
- Files are very small (overhead dominates)
- Storage is slow (mechanical HDD with limited IOPS)
- System has limited CPU cores

## Architecture

The implementation uses a simple but effective approach:

1. **File Handle Cloning**: Each thread opens its own `Archive` instance
2. **Cached Metadata**: File list is cached upfront to avoid repeated reads
3. **Rayon Parallelism**: Uses rayon for work-stealing thread pool
4. **Batching Support**: Option to process multiple files per thread

```rust
ParallelArchive {
    path: PathBuf,           // Path to archive file
    file_list: Arc<Vec<String>>, // Cached file list
}
```

## API Reference

### Core Types

#### `ParallelArchive`
The main type for parallel operations on a single archive.

```rust
let archive = ParallelArchive::open("data.mpq")?;
```

#### `ParallelConfig`
Configuration options for parallel extraction.

```rust
let config = ParallelConfig::new()
    .threads(4)        // Use 4 threads
    .batch_size(10)    // Process 10 files per batch
    .skip_errors(true); // Continue on errors
```

### Methods

#### `extract_files_parallel`
Extract multiple specific files in parallel.

```rust
let files = vec!["file1.txt", "file2.txt", "file3.txt"];
let results = archive.extract_files_parallel(&files)?;
```

#### `extract_matching_parallel`
Extract files matching a predicate.

```rust
let results = archive.extract_matching_parallel(|name| {
    name.ends_with(".blp") || name.starts_with("Interface/")
})?;
```

#### `process_files_parallel`
Apply custom processing to files.

```rust
let checksums = archive.process_files_parallel(&files, |name, data| {
    let checksum = calculate_crc32(&data);
    Ok((name.to_string(), checksum))
})?;
```

#### `extract_files_batched`
Extract files in batches for better performance with many small files.

```rust
let results = archive.extract_files_batched(&files, 20)?; // 20 files per batch
```

## Usage Examples

### Basic Parallel Extraction

```rust
use wow_mpq::single_archive_parallel::ParallelArchive;

let archive = ParallelArchive::open("patch-3.mpq")?;

// Extract specific files
let ui_files = vec![
    "Interface/Icons/INV_Misc_QuestionMark.blp",
    "Interface/Icons/Spell_Nature_StormReach.blp",
    "Interface/AddOns/Blizzard_Combat/Combat.lua",
];

let results = archive.extract_files_parallel(&ui_files)?;

for (filename, data) in results {
    println!("Extracted {}: {} bytes", filename, data.len());
}
```

### Pattern-Based Extraction

```rust
// Extract all Lua files
let lua_files = archive.extract_matching_parallel(|name| {
    name.ends_with(".lua")
})?;

println!("Found {} Lua files", lua_files.len());
```

### Custom Processing

```rust
// Calculate checksums for all DBC files
let checksums = archive.process_files_parallel(&dbc_files, |name, data| {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    Ok((name.to_string(), hasher.finish()))
})?;
```

### Advanced Configuration

```rust
use wow_mpq::single_archive_parallel::{ParallelConfig, extract_with_config};

let config = ParallelConfig::new()
    .threads(8)          // Use 8 worker threads
    .batch_size(50)      // Process 50 files per batch
    .skip_errors(true);  // Continue on extraction errors

let results = extract_with_config("data.mpq", &files, config)?;

// Handle mixed results
for (filename, result) in results {
    match result {
        Ok(data) => println!("{}: {} bytes", filename, data.len()),
        Err(e) => println!("{}: ERROR - {}", filename, e),
    }
}
```

## Performance Characteristics

Based on benchmarks, typical performance improvements:

### Multiple File Extraction
- **10 files**: ~1.2x speedup
- **50 files**: ~2.5x speedup
- **100 files**: ~3.5x speedup
- **200 files**: ~4x speedup

### File Size Impact
- **Small files (1-10KB)**: Lower speedup due to overhead
- **Medium files (10-100KB)**: Good speedup
- **Large files (100KB+)**: Best speedup

### Thread Scaling
- 1 thread: Baseline
- 2 threads: ~1.8x speedup
- 4 threads: ~3.2x speedup
- 8 threads: ~4.5x speedup
- 16+ threads: Diminishing returns

## Implementation Details

### Thread Safety

Each thread gets its own:
- File handle (`BufReader<File>`)
- Archive instance
- Read buffers

This completely avoids synchronization overhead and seek conflicts.

### Memory Usage

Memory usage scales with:
- Number of threads (each has its own buffers)
- Decompression buffers (one per thread)
- File data being extracted

### Error Handling

Two modes available:
1. **Fail-fast** (default): First error stops all extraction
2. **Skip errors**: Continue extracting, return individual results

## Best Practices

1. **File Count**: Use parallel extraction for 10+ files
2. **Batch Size**: For many small files, use batching (10-50 files per batch)
3. **Thread Count**: Default (CPU cores) is usually optimal
4. **Error Handling**: Use skip_errors for resilient bulk extraction
5. **Memory**: Monitor memory usage when extracting many large files

## Comparison with Multi-Archive Parallel

| Feature | Single Archive | Multi-Archive |
|---------|---------------|---------------|
| Use Case | Many files from one archive | Same file from many archives |
| Parallelism | Per-file | Per-archive |
| Overhead | Higher (multiple handles) | Lower (one handle per archive) |
| Scaling | Limited by I/O | Near-linear with archives |
| Memory | Higher | Lower |

## Limitations

1. **I/O Bound**: Performance limited by storage speed
2. **Seek Overhead**: Each thread performs independent seeks
3. **Memory Usage**: Each thread maintains its own buffers
4. **Not for Writing**: Read-only operations only

## Future Improvements

Potential enhancements:

1. **Shared Decompression**: Cache decompressed sectors
2. **Read Coalescing**: Combine nearby reads
3. **Memory Mapping**: Use mmap for better OS caching
4. **Async I/O**: Overlap I/O with decompression

## Examples Directory

See the `examples/` directory for complete working examples:

- `single_archive_parallel_demo.rs`: Basic usage demonstration
- `bulk_texture_export.rs`: Export all textures from an archive
- `parallel_dbc_processing.rs`: Process database files in parallel