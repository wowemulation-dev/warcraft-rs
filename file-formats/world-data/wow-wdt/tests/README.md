# wow-wdt Tests

This directory contains the comprehensive test suite for wow-wdt.

## Test Structure

### Integration Tests

- **`integration_tests.rs`** - Core WDT parsing and manipulation
- **`comprehensive_validation.rs`** - Extensive validation testing

### Unit Tests

Unit tests are embedded in source files:

- Version detection in `src/version.rs`
- Chunk parsing in `src/chunks/`
- Conversion logic in `src/conversion.rs`

## Running Tests

```bash
# Run all tests
cargo test -p wow-wdt

# Run specific test file
cargo test -p wow-wdt --test integration_tests

# Run with output
cargo test -p wow-wdt -- --nocapture

# Run benchmarks
cargo bench -p wow-wdt
```

## Test Coverage

The test suite thoroughly covers:

- All chunk types (MVER, MPHD, MAIN, MAID, MODF, MWMO)
- Version compatibility (Classic through MoP+)
- Tile existence checking
- WMO object placement
- Area ID assignments
- Flag combinations
- Edge cases and error conditions

## Test Patterns

### Basic Parsing Test

```rust
#[test]
fn test_wdt_parsing() {
    let data = create_minimal_wdt();
    let wdt = Wdt::parse(&data).unwrap();

    assert_eq!(wdt.version, 18);
    assert!(wdt.validate().is_ok());
}
```

### Tile Testing

```rust
#[test]
fn test_tile_operations() {
    let mut wdt = Wdt::new();

    // Set tile existence
    wdt.set_tile(32, 48, true);
    assert!(wdt.tile_exists(32, 48));

    // Set area ID (Cataclysm+)
    wdt.set_area_id(32, 48, 1519);
    assert_eq!(wdt.get_area_id(32, 48), Some(1519));
}
```

### Version Compatibility

```rust
#[test]
fn test_version_conversion() {
    let classic_wdt = create_classic_wdt();
    let cata_wdt = classic_wdt.convert_to(Version::Cataclysm)?;

    // Verify MAID chunk was added
    assert!(cata_wdt.has_area_ids());
}
```

## Synthetic Test Data

Tests use carefully crafted synthetic data:

- Minimal valid WDT structures
- Maximum complexity scenarios
- Corrupted data for error testing
- Version-specific formats

## Performance Testing

Critical operations are benchmarked:

- Large WDT parsing (all tiles present)
- Tile lookup performance
- WMO object iteration
- Serialization/deserialization

## Common Test Scenarios

1. **Empty Map** - No tiles exist
2. **Full Map** - All 64x64 tiles exist
3. **Sparse Map** - Random tile distribution
4. **Global WMO** - Map with single large WMO
5. **Many Objects** - Hundreds of WMO placements
6. **Version Migration** - Converting between formats

## Validation Tests

Comprehensive validation includes:

- Chunk size verification
- Offset boundary checks
- Flag compatibility
- String table validation
- Object placement bounds
