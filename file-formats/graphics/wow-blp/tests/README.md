# wow-blp Tests

This directory contains tests for the wow-blp crate.

## Test Structure

Following the standardized test organization:

### `/component`
Unit tests for individual components (when added)

### `/integration`
- `format_tests.rs` - High-level tests for BLP functionality
  - Round-trip conversion tests
  - Format compatibility tests
  - Error handling tests

### `/scenarios`
Real-world usage scenarios (when added)

### `/compliance`
Compatibility and compliance tests (when added)

## Running Tests

```bash
# Run all tests
cargo test -p wow-blp

# Run with output
cargo test -p wow-blp -- --nocapture

# Run specific test
cargo test -p wow-blp test_round_trip_blp1_raw1
```

## Test Data

Note: The original parser tests that relied on proprietary Blizzard BLP files have been removed. The current tests use synthetic test data to verify functionality.

## Adding New Tests

When adding new tests:

1. Use synthetic test data when possible
2. Test all major code paths
3. Include error cases
4. Document what each test verifies

Example test structure:

```rust
#[test]
fn test_feature() {
    // Arrange
    let test_data = create_test_data();
    
    // Act
    let result = process_data(test_data);
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_value);
}
```