# wow-cdbc Examples

This directory contains examples demonstrating various features of the wow-cdbc crate.

## Running Examples

All examples can be run using `cargo run --example <name>`:

```bash
# Run the comprehensive example
cargo run --example comprehensive

# Run the simple comprehensive example
cargo run --example comprehensive_simple

# Run the schema discovery example
cargo run --example schema_discovery

# Run with a specific DBC file
cargo run --example comprehensive -- /path/to/file.dbc
```

## Available Examples

### comprehensive

Demonstrates all major features of the DBC parser including:

- Parsing with code-defined schema
- Lazy loading for large files
- Schema discovery
- Export to various formats
- Performance comparisons

### comprehensive_simple

A simplified version of the comprehensive example that demonstrates core DBC parsing functionality without all the advanced features.

### schema_discovery

Demonstrates automatic schema discovery for unknown DBC formats.

## Test Data

Examples will look for test DBC files in the following locations:

- Current directory
- `test_data/` subdirectory
- WoW data paths (if available)

You can provide your own DBC files by passing the path as a command-line argument.

## DBD Files

The examples also demonstrate DBD (Database Definition) file support. DBD files can be obtained from:

- [WoWDBDefs Repository](https://github.com/wowdev/WoWDBDefs)

Place DBD files in a `definitions/` subdirectory to use them with the examples.
