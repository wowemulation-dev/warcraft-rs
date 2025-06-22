# wow-mpq Examples

This directory contains examples demonstrating various features and use cases of the wow-mpq library.

## Running Examples

To run any example:

```bash
cargo run --example <example_name> [arguments]
```

For optimized performance:

```bash
cargo run --release --example <example_name> [arguments]
```

## Available Examples

### Basic Operations

- **`simple_list.rs`** - List contents of an MPQ archive

  ```bash
  cargo run --example simple_list path/to/archive.mpq
  ```

- **`list_archive_contents.rs`** - Detailed archive listing with file information

  ```bash
  cargo run --example list_archive_contents path/to/archive.mpq
  ```

### Archive Creation and Modification

- **`create_archive.rs`** - Comprehensive archive creation examples

  ```bash
  cargo run --example create_archive
  ```
  
  Demonstrates:
  - Basic archive creation
  - Files from disk
  - Custom compression and encryption
  - Attributes file generation
  - Different MPQ versions

- **`create_test_mpq.rs`** - Advanced CLI tool for creating test archives

  ```bash
  cargo run --example create_test_mpq -- minimal --version 2
  cargo run --example create_test_mpq -- compressed --compression zlib
  ```

- **`modify_archive.rs`** - Add, update, and remove files from an archive

  ```bash
  cargo run --example modify_archive
  ```

- **`bulk_modify.rs`** - Perform bulk operations on archives

  ```bash
  cargo run --example bulk_modify
  ```

### Patch Chains

World of Warcraft uses patch chains to apply updates:

- **`wow_patch_chains.rs`** - Comprehensive patch chain operations for all WoW versions

  ```bash
  cargo run --example wow_patch_chains
  ```
  
  Demonstrates the correct loading order for:
  - WoW 1.12.1 (Vanilla)
  - WoW 2.4.3 (The Burning Crusade)  
  - WoW 3.3.5a (Wrath of the Lich King)
  - WoW 4.3.4 (Cataclysm)
  - WoW 5.4.8 (Mists of Pandaria)

### Advanced Features

- **`signature_demo.rs`** - Work with MPQ digital signatures

  ```bash
  cargo run --example signature_demo
  ```

### Analysis and Debugging

- **`analyze_attributes.rs`** - Analyze (attributes) file structure

  ```bash
  cargo run --example analyze_attributes path/to/archive.mpq
  ```

- **`compare_archives.rs`** - Compare two MPQ archives

  ```bash
  cargo run --example compare_archives archive1.mpq archive2.mpq
  ```

- **`hash_algorithms_demo.rs`** - Demonstrate different hash algorithms

  ```bash
  cargo run --example hash_algorithms_demo
  ```

- **`verify_wow_files.rs`** - Verify integrity of WoW data files

  ```bash
  cargo run --example verify_wow_files /path/to/wow/Data
  ```

- **`patch_analysis.rs`** - General patch file analysis

  ```bash
  cargo run --example patch_analysis /path/to/patch.MPQ
  ```

### Compatibility Testing

- **`cross_compatibility_test.rs`** - Test cross-version compatibility

  ```bash
  cargo run --example cross_compatibility_test
  ```

- **`full_stormlib_compat_test.rs`** - Full StormLib compatibility test suite

  ```bash
  cargo run --example full_stormlib_compat_test
  ```

## Example Categories

### For Beginners

Start with these examples to understand basic MPQ operations:

1. `simple_list.rs` - Basic file listing
2. `create_archive.rs` - Comprehensive creation guide
3. `list_archive_contents.rs` - Detailed archive information

### For Game Modding

These examples are useful for WoW modding:

1. `wow_patch_chains.rs` - Understand WoW's loading system  
2. `modify_archive.rs` - Edit existing archives
3. `verify_wow_files.rs` - Validate game data

### For Advanced Users

Deep dive into MPQ internals:

1. `analyze_attributes.rs` - File metadata analysis
2. `signature_demo.rs` - Digital signature handling
3. `full_stormlib_compat_test.rs` - Compatibility verification

## Reorganization Note

This crate has been reorganized for better maintainability:

- **Consolidated Examples**: Reduced from 50+ to 15 focused examples
- **Test Organization**: Test-like examples moved to `tests/scenarios/`
- **Better Documentation**: Each example has clear purpose and usage

## Notes

- Some examples create temporary files for testing
- Examples requiring WoW data will show setup instructions if files not found
- Use `--release` mode for performance-critical operations
- Check individual example files for specific requirements
- Test files moved to `tests/` directory follow structured organization
