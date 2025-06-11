# wow-wdl Tests

This directory contains tests for the WDL (World Data Low-resolution) parser.

## Test Structure

### Integration Tests

- **`integration_test.rs`** - Complete WDL parsing and validation tests

### Unit Tests

Unit tests are located in the source files:

- Parser tests in `src/parser.rs`
- Validation tests in `src/validation.rs`
- Conversion tests in `src/conversion.rs`

## Running Tests

```bash
# Run all tests
cargo test -p wow-wdl

# Run integration tests only
cargo test -p wow-wdl --test integration_test

# Run with output
cargo test -p wow-wdl -- --nocapture
```

## Test Coverage

The test suite covers:

- WDL chunk parsing (MVER, MWMO, MWID, MODF, MAOF, MARE, MAHO)
- Version compatibility (Classic through MoP)
- Height data validation
- Water level parsing
- Area information
- Round-trip read/write operations

## Test Data

Tests use synthetic data to ensure reproducibility:

- Minimal valid WDL structures
- Edge cases (empty data, maximum values)
- Version-specific formats

## Writing New Tests

### Integration Test Pattern

```rust
#[test]
fn test_wdl_feature() {
    let wdl_data = create_test_wdl();
    let wdl = Wdl::parse(&wdl_data).unwrap();

    // Test specific feature
    assert_eq!(wdl.get_height(32, 48).unwrap(), 100.0);
}
```

### Property-based Testing

Consider using proptest for fuzz testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_height_bounds(x in 0u32..64, y in 0u32..64) {
        let wdl = create_test_wdl();
        let height = wdl.get_height(x, y).unwrap();
        assert!(height >= -1000.0 && height <= 1000.0);
    }
}
```

## Performance Tests

For performance-critical operations:

```bash
cargo bench -p wow-wdl
```

## Known Test Scenarios

1. **Empty WDL** - Valid file with no height data
2. **Full Coverage** - All 64x64 tiles with data
3. **Sparse Data** - Only some tiles have data
4. **Water Levels** - Various water configurations
5. **Version Migration** - Converting between formats
