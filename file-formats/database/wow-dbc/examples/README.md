# wow-dbc Examples

This directory contains examples demonstrating various features of the wow-dbc crate.

## Running Examples

All examples can be run using `cargo run --example <name>`:

```bash
# Run the comprehensive example
cargo run --example comprehensive

# Run the lazy loading example
cargo run --example lazy_loading

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

### lazy_loading

Shows how to use the lazy loading API for memory-efficient parsing of large DBC files.

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
