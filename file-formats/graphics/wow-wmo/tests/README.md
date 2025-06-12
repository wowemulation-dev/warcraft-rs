# wow-wmo Tests

This directory contains tests for the WMO (World Map Object) file format parser.

## Test Structure

### Unit Tests

Located in `parser_tests.rs`:

- **Version parsing** - WMO version detection and compatibility
- **Flag handling** - WMO and group flags manipulation
- **Type structures** - Basic WMO data types (Vec3, Color, BoundingBox)
- **Material system** - WMO material properties and flags
- **Lighting system** - Light types, properties, and parameters
- **Doodad system** - Doodad definitions and sets
- **Portal system** - Portal geometry and properties
- **Group system** - WMO group headers and batches
- **Texture handling** - Texture coordinates and special texture values

## Running Tests

```bash
# Run all tests
cargo test -p wow-wmo

# Run with output
cargo test -p wow-wmo -- --nocapture
```

## Test Coverage

The current test suite covers:

- **Version compatibility** - Classic through Legion WMO versions
- **Flag operations** - WMO flags, material flags, and group flags  
- **Data types** - Core types like Vec3, Color, BoundingBox, ChunkId
- **Component structures** - Headers, materials, lights, doodads, portals
- **Special handling** - Texture marker values and validation
- **Group data** - Group headers, texture coordinates, render batches

## Test Data

Tests use synthetic WMO data structures to avoid dependencies on game files.
All test data is generated at runtime using minimal valid WMO components.

For testing with real WMO files (future):

- `WMO_TEST_FILE` - Path to a test WMO file
- `WOW_DATA_DIR` - Path to WoW data directory

## Writing Tests

Current patterns in the test suite:

- Use synthetic data structures
- Test individual components in isolation
- Verify flag operations and data integrity
- Test edge cases and special values
- Mark future tests requiring game data with `#[ignore]`
