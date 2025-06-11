# wow-wmo Tests

This directory contains tests for the WMO (World Map Object) file format parser.

## Test Structure

*Note: This crate is still in development. Tests will be added as features are implemented.*

### Planned Test Categories

- **Unit Tests** - Individual chunk parsers
- **Integration Tests** - Complete WMO file parsing
- **Version Tests** - Compatibility across game versions
- **Validation Tests** - File structure validation

## Running Tests

```bash
# Run all tests
cargo test -p wow-wmo

# Run with output
cargo test -p wow-wmo -- --nocapture
```

## Test Data

Tests will use minimal synthetic WMO data to avoid dependencies on game files.

For testing with real WMO files, use environment variables:

- `WMO_TEST_FILE` - Path to a test WMO file
- `WOW_DATA_DIR` - Path to WoW data directory

## Writing Tests

Follow the existing patterns in other file format crates:

- Use synthetic data where possible
- Mark tests requiring game data with `#[ignore]`
- Test both parsing and writing operations
- Verify round-trip compatibility
