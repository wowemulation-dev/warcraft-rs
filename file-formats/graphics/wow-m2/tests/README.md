# WoW M2 Integration Tests

This directory contains integration tests for the wow-m2 crate.

## Test Organization

- `parser_tests.rs` - Tests for parsing M2 files from different WoW versions
- `converter_tests.rs` - Tests for version conversion functionality
- `skin_tests.rs` - Tests for M2 skin file parsing and handling
- `anim_tests.rs` - Tests for animation file parsing

## Running Tests

```bash
# Run all integration tests
cargo test --test '*'

# Run specific test file
cargo test --test parser_tests
```

## Test Data

Integration tests use synthetic test data created programmatically to avoid dependencies on copyrighted game files.
