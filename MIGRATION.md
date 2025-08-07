# Migration Guide: warcraft-rs v0.3.0

This guide helps you migrate from previous versions to warcraft-rs v0.3.0, which includes significant improvements to ADT, WDT, and WDL parsing with comprehensive World of Warcraft version support.

## Summary of Changes

### Version 0.3.0 - Breaking Changes Release

This is a **MINOR VERSION BUMP** (0.2.1 → 0.3.0) due to breaking API changes. In pre-1.0 releases, minor version bumps indicate breaking changes.

### Major Improvements ✅
- **Complete WoW version support** - Vanilla through Mists of Pandaria
- **Automatic version detection** - No manual version specification needed
- **Split ADT file support** - Cataclysm+ `_tex0`, `_obj0`, `_obj1`, `_lod` files  
- **TrinityCore compliance** - Validated against server implementation
- **Enhanced chunk parsing** - MFBO, MAMP, MTXP, MH2O improvements

### Breaking Changes 🔄 (Requires Code Updates)
- **MFBO chunk structure** - Changed from 8 bytes to 36 bytes (correct format)
- **Version detection API** - Enhanced methods for chunk-based detection
- **WDT conditional chunks** - MWMO handling for Cataclysm+ compatibility
- **API reorganization** - Some internal structures moved/renamed

### Semantic Versioning Note
Since warcraft-rs is pre-1.0, we follow the convention that:
- **PATCH** (0.2.X) = Bug fixes, no breaking changes
- **MINOR** (0.X.0) = New features, **may include breaking changes**
- **MAJOR** (X.0.0) = Reserved for 1.0+ stable API

## ADT (Terrain) Changes

### MFBO Chunk Structure Fix

**What Changed:**
The MFBO (flight boundaries) chunk structure was corrected from an incorrect 8-byte format to the proper 36-byte format used by TrinityCore and WoW clients.

**Before (Incorrect):**
```rust
pub struct MfboChunk {
    pub min: u32,  // Wrong: single 4-byte value
    pub max: u32,  // Wrong: single 4-byte value  
}
// Total: 8 bytes (incorrect)
```

**After (Correct):**
```rust
pub struct MfboChunk {
    pub max: [i16; 9],  // Maximum flight boundary plane (9 coordinates)
    pub min: [i16; 9],  // Minimum flight boundary plane (9 coordinates)
}
// Total: 36 bytes (2 planes × 9 coordinates × 2 bytes each)
```

**Migration:**
```rust
// OLD CODE - will not compile
let mfbo = adt.mfbo().unwrap();
println!("Min: {}, Max: {}", mfbo.min, mfbo.max);

// NEW CODE - access coordinate arrays
let mfbo = adt.mfbo().unwrap();
println!("Min plane: {:?}", mfbo.min);
println!("Max plane: {:?}", mfbo.max);

// Access specific coordinates
println!("First min coord: {}", mfbo.min[0]);
println!("First max coord: {}", mfbo.max[0]);
```

### Version Detection Enhancements

**What Changed:**
Version detection now uses comprehensive chunk analysis instead of relying solely on MVER values (which are always 18 for all ADT versions).

**Before:**
```rust
// Limited version detection
let version = AdtVersion::from_mver(18)?; // Always returns Vanilla
```

**After:**
```rust  
// Automatic version detection from file
let adt = Adt::from_path("terrain.adt")?;
let version = adt.version(); // Automatically detects TBC, WotLK, Cataclysm, MoP

// Manual version detection from chunk presence  
let version = AdtVersion::detect_from_chunks_extended(
    true,  // has MFBO (TBC+)
    true,  // has MH2O (WotLK+) 
    false, // no MTFX
    false, // no MCCV
    true,  // has MTXP (MoP+)
    true,  // has MAMP (Cataclysm+)
);
assert_eq!(version, AdtVersion::MoP);
```

### New Split File Support

**What's New:**
Cataclysm+ split file parsing for memory-optimized terrain loading.

**Usage:**
```rust
use wow_adt::split_adt::{SplitAdtParser, SplitAdtType};

// Parse texture data from split file
let tex_data = SplitAdtParser::parse_tex0(&mut reader)?;

// Parse object placement from split file
let obj_data = SplitAdtParser::parse_obj0(&mut reader)?;

// Detect file type from filename
let file_type = SplitAdtType::from_filename("Map_32_48_tex0.adt");
assert_eq!(file_type, SplitAdtType::Tex0);
```

### New Chunk Support

**MAMP Chunk (Cataclysm+):**
```rust
// Access texture amplifier values
if let Some(mamp) = adt.mamp() {
    println!("Texture amplifier: {}", mamp.value);
}
```

**MTXP Chunk (Mists of Pandaria+):**
```rust
// Access texture parameters
if let Some(mtxp) = adt.mtxp() {
    for param in &mtxp.entries {
        println!("Texture params: {:?}", param.params);
    }
}
```

## WDT (World Data Table) Changes

### Conditional MWMO Chunk Handling

**What Changed:**
WDT files now correctly handle MWMO chunk presence based on WoW version and map type, matching TrinityCore server behavior.

**Key Changes:**
- **Pre-Cataclysm**: All maps have MWMO chunks (even if empty)
- **Cataclysm+**: Only WMO-only maps have MWMO chunks, terrain maps don't

**Migration:**
```rust
// OLD CODE - assumed all maps have MWMO
let wmo_names = wdt.mwmo.unwrap().filenames;

// NEW CODE - check version and map type
if let Some(mwmo) = wdt.mwmo {
    // MWMO present - either pre-Cataclysm or WMO-only map
    let wmo_names = mwmo.filenames;
} else {
    // No MWMO - likely Cataclysm+ terrain map
    println!("Terrain map without global WMO");
}

// Check if map is WMO-only
if wdt.is_wmo_only() {
    // WMO-only maps should always have MWMO and MODF
    assert!(wdt.mwmo.is_some());
    assert!(wdt.modf.is_some());
}
```

### Enhanced Version Detection

**Before:**
```rust
// Manual version specification
let reader = WdtReader::new(file, WowVersion::Cataclysm);
```

**After:**
```rust
// Automatic version detection
let reader = WdtReader::new(file, WowVersion::Classic); // Initial hint
let wdt = reader.read()?; // Automatically detects actual version
println!("Detected version: {}", wdt.version());
```

## WDL (World Distance Lookup) Changes

### Improved Version Support

**What's New:**
Enhanced parsing for different WDL format versions with better chunk handling.

**Usage:**
```rust
use wow_wdl::{WdlParser, WdlVersion};

// Automatic version detection
let parser = WdlParser::new(); // Uses latest version support
let wdl = parser.parse(&mut reader)?;

// Check detected version  
match wdl.version {
    WdlVersion::Vanilla => println!("Classic WDL format"),
    WdlVersion::Wotlk => println!("Enhanced water support"),
    WdlVersion::Legion => println!("Modern ML chunk format"),
    _ => println!("Other version"),
}
```

## Testing Improvements

### New Test Categories

The test suite now includes comprehensive validation:

```bash
# Run version-specific tests
cargo test --package wow-adt --test lib -- scenarios::version_specific

# Run TrinityCore compliance tests  
cargo test --package wow-adt --test lib -- compliance::trinitycore

# Run all integration tests
cargo test --package wow-adt --test lib
```

## Performance Considerations

### Memory Usage
- **Split file support** reduces memory usage for selective loading
- **Lazy chunk parsing** improves startup time for large terrain files
- **Version detection caching** avoids redundant analysis

### Compatibility
- **TrinityCore compliance** ensures server compatibility
- **Automatic version detection** handles mixed-version content
- **Graceful degradation** for unsupported chunks

## Common Migration Patterns

### Pattern 1: MFBO Structure Access
```rust
// OLD
if let Some(mfbo) = adt.mfbo() {
    let range = mfbo.max - mfbo.min; // Won't compile
}

// NEW  
if let Some(mfbo) = adt.mfbo() {
    for i in 0..9 {
        let range = mfbo.max[i] - mfbo.min[i];
        println!("Coordinate {}: range {}", i, range);
    }
}
```

### Pattern 2: Version-Aware Code
```rust
// OLD - manual version tracking
let version = WowVersion::Cataclysm;
match version {
    WowVersion::Cataclysm => { /* handle split files */ }
    _ => { /* handle monolithic files */ }
}

// NEW - automatic detection
let adt = Adt::from_path("terrain.adt")?;
match adt.version() {
    AdtVersion::Cataclysm | AdtVersion::MoP => {
        // Handle modern format with potential split files
        if let Some(mamp) = adt.mamp() {
            println!("Has texture amplifiers");
        }
    }
    AdtVersion::WotLK => {
        // Handle WotLK water features
        if let Some(mh2o) = adt.mh2o() {
            println!("Has enhanced water");
        }
    }
    _ => {
        // Handle earlier versions
    }
}
```

### Pattern 3: Error Handling
```rust
// OLD - basic error handling
let adt = Adt::from_path(path).expect("Failed to parse");

// NEW - comprehensive error handling
let adt = match Adt::from_path(path) {
    Ok(adt) => adt,
    Err(AdtError::UnsupportedVersion(v)) => {
        eprintln!("Unsupported ADT version: {}", v);
        return;
    }
    Err(AdtError::InvalidChunkSize { chunk, size, expected }) => {
        eprintln!("Invalid {} chunk: got {} bytes, expected {}", chunk, size, expected);
        return;  
    }
    Err(e) => {
        eprintln!("Parse error: {}", e);
        return;
    }
};
```

## Validation and Testing

### Validate Your Migration

1. **Compile Check**: Ensure your code compiles with the new API
2. **Test Suite**: Run your existing tests to catch API changes
3. **Integration Test**: Test with real ADT/WDT/WDL files from different WoW versions
4. **Performance Test**: Verify memory usage and parsing speed

### Compatibility Testing

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn test_mfbo_migration() {
        // Test that MFBO parsing works with new structure
        let adt = Adt::from_path("test_tbc.adt").unwrap();
        if let Some(mfbo) = adt.mfbo() {
            assert_eq!(mfbo.max.len(), 9);
            assert_eq!(mfbo.min.len(), 9);
            // Verify coordinate values are reasonable
            for coord in &mfbo.max {
                assert!(*coord > -10000 && *coord < 10000);
            }
        }
    }

    #[test]
    fn test_version_detection_migration() {
        // Test automatic version detection
        let adt = Adt::from_path("test_mop.adt").unwrap();
        assert_eq!(adt.version(), AdtVersion::MoP);
        
        // Verify version-specific features are detected
        assert!(adt.mfbo().is_some()); // TBC+
        assert!(adt.mh2o().is_some()); // WotLK+ 
        assert!(adt.mamp().is_some()); // Cataclysm+
        assert!(adt.mtxp().is_some()); // MoP+
    }
}
```

## Getting Help

If you encounter issues during migration:

1. **Check the examples** in the updated documentation
2. **Review the test suite** for usage patterns  
3. **Open an issue** with specific migration questions
4. **Consult TrinityCore** for server-side behavior questions

## Summary

This release significantly improves World of Warcraft file format support with better accuracy, comprehensive version handling, and TrinityCore compliance. While there are breaking changes, they fix incorrect implementations and provide much more robust parsing capabilities.

The migration effort is primarily around:
- Updating MFBO chunk access patterns  
- Leveraging automatic version detection
- Handling conditional chunk presence appropriately

The result is a more reliable, accurate, and feature-complete warcraft-rs library.