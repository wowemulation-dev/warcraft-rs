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

- **`create_archive.rs`** - Create a new MPQ archive

  ```bash
  cargo run --example create_archive
  ```

- **`modify_archive.rs`** - Add, update, and remove files from an archive

  ```bash
  cargo run --example modify_archive
  ```

- **`bulk_modify.rs`** - Perform bulk operations on archives

  ```bash
  cargo run --example bulk_modify
  ```

### Advanced Features

- **`signature_demo.rs`** - Work with MPQ digital signatures

  ```bash
  cargo run --example signature_demo
  ```

- **`create_encrypted_archive.rs`** - Create archives with encryption

  ```bash
  cargo run --example create_encrypted_archive
  ```

- **`create_archive_with_attributes.rs`** - Add (attributes) file metadata

  ```bash
  cargo run --example create_archive_with_attributes
  ```

### Patch Chains

World of Warcraft uses patch chains to apply updates. These examples demonstrate working with patch files:

- **`patch_chain_demo.rs`** - Basic patch chain operations

  ```bash
  cargo run --example patch_chain_demo /path/to/wow/Data
  ```

- **`wotlk_patch_chain_demo.rs`** - WotLK-specific patch handling

  ```bash
  cargo run --example wotlk_patch_chain_demo /path/to/wotlk/Data
  ```

- **`cata_patch_chain_demo.rs`** - Cataclysm patch chain features

  ```bash
  cargo run --example cata_patch_chain_demo /path/to/cata/Data
  ```

- **`mop_patch_chain_demo.rs`** - Mists of Pandaria patch handling

  ```bash
  cargo run --example mop_patch_chain_demo /path/to/mop/Data
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

- **`verify_wow_files.rs`** - Verify integrity of WoW data files

  ```bash
  cargo run --example verify_wow_files /path/to/wow/Data
  ```

### Compatibility Testing

- **`cross_compatibility_test.rs`** - Test cross-version compatibility

  ```bash
  cargo run --example cross_compatibility_test
  ```

- **`test_stormlib_hash_size.rs`** - Verify StormLib compatibility

  ```bash
  cargo run --example test_stormlib_hash_size
  ```

### Performance and Compression

- **`test_recompression.rs`** - Test different compression methods

  ```bash
  cargo run --example test_recompression
  ```

- **`test_huffman_compression_analysis.rs`** - Analyze Huffman compression

  ```bash
  cargo run --example test_huffman_compression_analysis
  ```

## Example Categories

### For Beginners

Start with these examples to understand basic MPQ operations:

1. `simple_list.rs`
2. `create_archive.rs`
3. `list_archive_contents.rs`

### For Game Modding

These examples are useful for WoW modding:

1. `patch_chain_demo.rs`
2. `modify_archive.rs`
3. `verify_wow_files.rs`

### For Advanced Users

Deep dive into MPQ internals:

1. `analyze_attributes.rs`
2. `signature_demo.rs`
3. `test_huffman_compression_analysis.rs`

## Notes

- Many examples create temporary files for testing
- Some examples require actual WoW data files
- Use `--release` mode for performance-critical operations
- Check individual example files for specific requirements
