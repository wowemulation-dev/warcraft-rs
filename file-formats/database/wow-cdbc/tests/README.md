# wow-cdbc Tests

This directory contains the test suite for the wow-cdbc crate.

## Test Structure

- `integration_tests.rs` - End-to-end tests for complete DBC parsing workflows
- `compatibility_tests.rs` - Tests ensuring compatibility with WoWDev standards

## Running Tests

```bash
# Run all tests
cargo test -p wow-cdbc

# Run with output for debugging
cargo test -p wow-cdbc -- --nocapture

# Run a specific test
cargo test -p wow-cdbc test_header_parsing

# Run tests with all features enabled
cargo test -p wow-cdbc --all-features
```

## Test Data

Tests use synthetic DBC data generated at runtime. This ensures tests are:

- Reproducible
- Don't require external files
- Cover edge cases systematically

## Compatibility Testing

The compatibility tests verify:

- Header format compliance with WoWDev.wiki specifications
- String block handling matches community standards
- Field type sizing is correct
- Multi-version support works as expected
- Error handling follows expected patterns

## Adding New Tests

When adding new tests:

1. Use the test data generation utilities for consistency
2. Test both success and error cases
3. Verify compatibility with WoWDev standards
4. Consider testing across multiple DBC versions

## Ignored Tests

Some tests are marked as `#[ignore]` because they require:

- Improvements to schema discovery algorithms
- Complex array detection logic
- Future format support

Run ignored tests with:

```bash
cargo test -p wow-cdbc -- --ignored
```
