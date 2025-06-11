# wow-adt Tests

This directory contains the test suite for the wow-adt library.

## Test Structure

### Unit Tests

Located in `src/` alongside the implementation:

- Chunk parsing tests
- Height map calculations
- Texture layer handling
- Water data parsing

### Integration Tests

Located in this directory:

- **`parser_tests.rs`** - Complete ADT parsing tests
- More tests to be added as features are implemented

## Running Tests

```bash
# Run all tests
cargo test -p wow-adt

# Run specific test file
cargo test -p wow-adt --test parser_tests

# Run with output
cargo test -p wow-adt -- --nocapture

# Run ignored tests (requires game files)
cargo test -p wow-adt -- --ignored
```

## Test Data

The tests use synthetic ADT data to avoid requiring game files.
Test utilities generate minimal valid ADT structures.

For tests with real ADT files:

1. Set `WOW_DATA_DIR` environment variable
2. Extract ADT files from MPQ archives
3. Run ignored tests

## Writing New Tests

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_parsing() {
        let data = create_test_chunk_data();
        let chunk = parse_chunk(&data).unwrap();
        assert_eq!(chunk.vertices.len(), 145);
    }
}
```

### Integration Test Example

```rust
#[test]
fn test_complete_adt_parsing() {
    let adt_data = create_minimal_adt();
    let adt = Adt::parse(&adt_data).unwrap();

    assert_eq!(adt.chunks.len(), 256);
    assert!(adt.validate().is_ok());
}

#[test]
#[ignore] // Requires game files
fn test_real_adt_file() {
    let path = std::env::var("WOW_DATA_DIR")
        .expect("Set WOW_DATA_DIR");

    let adt = Adt::load(path).unwrap();
    assert!(adt.validate().is_ok());
}
```

## Performance Tests

Performance-critical operations should be benchmarked:

```bash
cargo bench -p wow-adt
```

## Coverage

Generate coverage reports:

```bash
cargo tarpaulin -p wow-adt --out Html
```

## Common Test Scenarios

1. **Version Compatibility** - Test all supported game versions
2. **Chunk Boundaries** - Verify seamless chunk connections
3. **Height Interpolation** - Test terrain height calculations
4. **Texture Blending** - Verify alpha map handling
5. **Water Levels** - Test water data parsing
6. **Object Placement** - Verify M2/WMO references
