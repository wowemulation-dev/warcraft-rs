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

- **`create_test_mpq.rs`** - Create test MPQ archives for testing

  ```bash
  cargo run --example create_test_mpq
  ```

- **`create_comparison_archives.rs`** - Create archives for compatibility testing

  ```bash
  cargo run --example create_comparison_archives
  ```

- **`create_compressed_tables.rs`** - Create archives with compressed hash/block tables

  ```bash
  cargo run --example create_compressed_tables
  ```

- **`modify_stormlib_archive.rs`** - Modify archives for StormLib compatibility testing

  ```bash
  cargo run --example modify_stormlib_archive
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

- **`analyze_blizzard_attributes.rs`** - Analyze Blizzard-specific attribute data

  ```bash
  cargo run --example analyze_blizzard_attributes path/to/archive.mpq
  ```

- **`analyze_v4_header.rs`** - Analyze MPQ version 4 header structure

  ```bash
  cargo run --example analyze_v4_header path/to/v4-archive.mpq
  ```

- **`compare_archives.rs`** - Compare two MPQ archives

  ```bash
  cargo run --example compare_archives archive1.mpq archive2.mpq
  ```

- **`debug_het_bet_creation.rs`** - Debug HET/BET table creation process

  ```bash
  cargo run --example debug_het_bet_creation
  ```

- **`hash_algorithms_demo.rs`** - Demonstrate different hash algorithms

  ```bash
  cargo run --example hash_algorithms_demo
  ```

- **`verify_wow_files.rs`** - Verify integrity of WoW data files

  ```bash
  cargo run --example verify_wow_files /path/to/wow/Data
  ```

- **`comprehensive_archive_verification.rs`** - Comprehensive archive integrity verification

  ```bash
  cargo run --example comprehensive_archive_verification path/to/archive.mpq
  ```

- **`random_archive_verification.rs`** - Random archive verification testing

  ```bash
  cargo run --example random_archive_verification
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

- **`test_stormlib_hash_size.rs`** - Verify StormLib compatibility

  ```bash
  cargo run --example test_stormlib_hash_size
  ```

- **`create_test_archive_for_stormlib.rs`** - Create test archives for StormLib comparison

  ```bash
  cargo run --example create_test_archive_for_stormlib
  ```

- **`test_archivebuilder_v3_simple.rs`** - Test archive builder v3 compatibility

  ```bash
  cargo run --example test_archivebuilder_v3_simple
  ```

- **`test_hybrid_v3_approach.rs`** - Test hybrid v3 compatibility approach

  ```bash
  cargo run --example test_hybrid_v3_approach
  ```

- **`test_v3_modification_systematic.rs`** - Systematic v3 modification testing

  ```bash
  cargo run --example test_v3_modification_systematic
  ```

- **`test_modification_issue.rs`** - Test specific modification issues

  ```bash
  cargo run --example test_modification_issue
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

- **`test_huffman_roundtrip.rs`** - Test Huffman compression roundtrip

  ```bash
  cargo run --example test_huffman_roundtrip
  ```

- **`test_adpcm_audio_files.rs`** - Test ADPCM audio file compression

  ```bash
  cargo run --example test_adpcm_audio_files
  ```

### Version-Specific Testing

- **`test_all_wow_versions_comprehensive.rs`** - Comprehensive testing across all WoW versions

  ```bash
  cargo run --example test_all_wow_versions_comprehensive /path/to/wow-versions
  ```

- **`test_cataclysm_files_comprehensive.rs`** - Comprehensive Cataclysm file testing

  ```bash
  cargo run --example test_cataclysm_files_comprehensive /path/to/cata/Data
  ```

- **`test_mop_files_comprehensive.rs`** - Comprehensive Mists of Pandaria file testing

  ```bash
  cargo run --example test_mop_files_comprehensive /path/to/mop/Data
  ```

- **`test_tbc_files_comprehensive.rs`** - Comprehensive Burning Crusade file testing

  ```bash
  cargo run --example test_tbc_files_comprehensive /path/to/tbc/Data
  ```

- **`test_wotlk_files_comprehensive.rs`** - Comprehensive Wrath of the Lich King file testing

  ```bash
  cargo run --example test_wotlk_files_comprehensive /path/to/wotlk/Data
  ```

### Patch Analysis

- **`patch_analysis.rs`** - General patch file analysis

  ```bash
  cargo run --example patch_analysis /path/to/patch.MPQ
  ```

- **`tbc_patch_analysis.rs`** - Burning Crusade patch analysis

  ```bash
  cargo run --example tbc_patch_analysis /path/to/tbc/Data
  ```

- **`tbc_patch_chain_demo.rs`** - TBC patch chain operations

  ```bash
  cargo run --example tbc_patch_chain_demo /path/to/tbc/Data
  ```

- **`patch_chain_dbc_demo.rs`** - DBC extraction from patch chains

  ```bash
  cargo run --example patch_chain_dbc_demo /path/to/wow/Data
  ```

- **`wow_patch_chains.rs`** - General WoW patch chain operations

  ```bash
  cargo run --example wow_patch_chains /path/to/wow/Data
  ```

### Development and Testing

- **`generate_test_data.rs`** - Generate test data for development

  ```bash
  cargo run --example generate_test_data
  ```

- **`test_specific_files.rs`** - Test specific file operations

  ```bash
  cargo run --example test_specific_files
  ```

- **`test_archive_structure_analysis_wowmpq.rs`** - Analyze archive structure

  ```bash
  cargo run --example test_archive_structure_analysis_wowmpq path/to/archive.mpq
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
