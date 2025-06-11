# wow-mpq Tests

This directory contains the comprehensive test suite for the wow-mpq library.

## Test Structure

The tests are organized into several categories:

### `/common`

Shared test utilities and helper functions used across different test modules.

### `/component`

Unit tests for individual components:

- **compression/** - Tests for each compression algorithm (ZLIB, BZIP2, Huffman, etc.)
- **io/** - I/O operation tests (cursor writes, file operations)
- **security/** - Cryptography and security tests (hashing, CRC, signatures)
- **tables/** - Hash table and HET/BET table tests

### `/integration`

Integration tests that verify complete workflows:

- **archive/** - Full archive operations (creation, modification, attributes)
- **compression/** - Compression API integration
- **security/** - Digital signature workflows
- **special_files/** - Tests for (listfile), (attributes), etc.

### `/compliance`

Compatibility and compliance tests:

- **stormlib/** - StormLib compatibility verification
- **performance/** - Performance benchmarks and regression tests

### `/scenarios`

Real-world usage scenarios:

- **real_world/** - Tests with actual game data patterns
- **round_trip/** - Read/write/read verification
- **stress/** - Edge cases and stress testing

## Running Tests

### Run all tests

```bash
cargo test -p wow-mpq
```

### Run specific test category

```bash
# Component tests only
cargo test -p wow-mpq component::

# Integration tests only
cargo test -p wow-mpq integration::

# Specific module
cargo test -p wow-mpq compression::
```

### Run with verbose output

```bash
cargo test -p wow-mpq -- --nocapture
```

### Run ignored tests (requires game data)

```bash
cargo test -p wow-mpq -- --ignored
```

## StormLib Comparison Tests

The `/stormlib_comparison` directory contains C++ test programs that verify compatibility with the StormLib reference implementation.

### Building StormLib tests

```bash
cd tests/stormlib_comparison
./compile_and_run.sh
```

### Individual StormLib tests

- `test_archive_creation_comparison` - Compare archive creation
- `test_archive_modification` - Test modification compatibility
- `test_attributes_parsing` - Verify attributes handling
- `test_huffman_compression_investigation` - Huffman codec comparison

## Test Data

### Generated Test Data

Most tests generate their own test data using utilities in `test_utils/`:

- `data_generator.rs` - Creates test files with patterns
- `mpq_builder.rs` - Helper for building test archives
- `wow_data.rs` - WoW-specific test data patterns

### Real Game Data

Some tests (marked with `#[ignore]`) require actual WoW data files. Set the following environment variables:

- `WOW_DATA_DIR` - Path to WoW Data directory
- `WOW_VERSION` - Version being tested (e.g., "3.3.5a")

## Writing New Tests

### Component Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_my_feature() {
        // Test implementation
    }
}
```

### Integration Test Template

```rust
use wow_mpq::*;
use tempfile::TempDir;

#[test]
fn test_complete_workflow() {
    let temp_dir = TempDir::new().unwrap();
    // Test implementation
}
```

## Performance Testing

Performance-sensitive tests should use criterion benchmarks instead:

```bash
cargo bench -p wow-mpq
```

## Coverage

To generate test coverage report:

```bash
cargo tarpaulin -p wow-mpq --out Html
```

## Common Issues

1. **File descriptor limits** - Some tests create many files. Increase ulimit if needed.
2. **Disk space** - Stress tests may require several GB of temporary space.
3. **StormLib dependency** - C++ comparison tests require StormLib to be installed.

## Test Categories by Priority

### Critical (Always Run)

- Basic archive operations
- Compression/decompression
- File integrity

### Important (CI/CD)

- Version compatibility
- Security features
- Special file handling

### Extended (Manual)

- StormLib comparison
- Stress testing
- Performance benchmarks
