# Migration Guide

This guide documents breaking API changes across warcraft-rs releases.

Since warcraft-rs is pre-1.0, minor version bumps may include breaking changes:
- **PATCH** (0.x.Y) = Bug fixes, no breaking changes
- **MINOR** (0.Y.0) = New features, may include breaking changes
- **MAJOR** (Y.0.0) = Reserved for 1.0+ stable API

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
use wow_adt::{parse_adt, ParsedAdt};

let parsed = parse_adt(&mut reader)?;
if let ParsedAdt::Root(root) = parsed {
    if let Some(mfbo) = &root.flight_bounds {
        println!("Min plane: {:?}", mfbo.min);
        println!("Max plane: {:?}", mfbo.max);
    }
}
```

### Version Detection Enhancements

**What Changed:**
Version detection now uses chunk analysis instead of relying solely on MVER values (which are always 18 for all ADT versions).

**Current API:**
```rust
use wow_adt::{parse_adt, ParsedAdt, AdtVersion};

// Automatic version detection from chunk presence
let parsed = parse_adt(&mut reader)?;
let version = parsed.version(); // Detects VanillaEarly through MoP

// Or detect from chunk discovery
use wow_adt::{discover_chunks, AdtVersion};
let discovery = discover_chunks(&mut reader)?;
let version = AdtVersion::from_discovery(&discovery);
```

### Split File Support

Cataclysm+ split file parsing. The `parse_adt` function auto-detects file type:

```rust
use wow_adt::{parse_adt, ParsedAdt};

let parsed = parse_adt(&mut reader)?;
match parsed {
    ParsedAdt::Root(root) => { /* main terrain file */ }
    ParsedAdt::Tex0(tex) => { /* texture data */ }
    ParsedAdt::Obj0(obj) => { /* object placements */ }
    ParsedAdt::Lod(lod) => { /* level-of-detail data */ }
    _ => {}
}
```

### Version-Specific Chunks

Access version-specific optional chunks on `RootAdt`:

```rust
if let ParsedAdt::Root(root) = parsed {
    // TBC+: flight bounds
    if let Some(mfbo) = &root.flight_bounds { /* ... */ }
    // WotLK+: water data
    if let Some(mh2o) = &root.water_data { /* ... */ }
    // Cataclysm+: texture amplifier
    if let Some(mamp) = &root.texture_amplifier { /* ... */ }
    // MoP+: texture params
    if let Some(mtxp) = &root.texture_params { /* ... */ }
}
```

## WDT (World Data Table) Changes

### Conditional MWMO Chunk Handling

**What Changed:**
WDT files now correctly handle MWMO chunk presence based on WoW version and map type, matching TrinityCore server behavior.

**Key Changes:**
- **Pre-Cataclysm**: All maps have MWMO chunks (even if empty)
- **Cataclysm+**: Only WMO-only maps have MWMO chunks, terrain maps don't

**Current API:**
```rust
use wow_wdt::{WdtReader, version::WowVersion};

let reader = WdtReader::new(BufReader::new(file), WowVersion::Cataclysm);
let wdt = reader.read()?;

// Check if map is WMO-only
if wdt.is_wmo_only() {
    println!("WMO-only map");
}
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

The test suite now includes validation for all categories:

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

### Pattern 1: Version-Aware ADT Code
```rust
use wow_adt::{parse_adt, ParsedAdt, AdtVersion};

let parsed = parse_adt(&mut reader)?;
if let ParsedAdt::Root(root) = &parsed {
    match root.version {
        AdtVersion::Cataclysm | AdtVersion::MoP => {
            if root.texture_amplifier.is_some() {
                println!("Has texture amplifiers");
            }
        }
        AdtVersion::WotLK => {
            if root.water_data.is_some() {
                println!("Has enhanced water");
            }
        }
        _ => {}
    }
}
```

### Pattern 2: Error Handling
```rust
use wow_adt::{parse_adt, AdtError};

match parse_adt(&mut reader) {
    Ok(parsed) => { /* use parsed */ }
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

Run the test suite to verify parsing:

```bash
# Run all ADT tests
cargo test --package wow-adt

# Run all WDT tests
cargo test --package wow-wdt

# Run all WDL tests
cargo test --package wow-wdl
```

## Getting Help

If you encounter issues during migration:

1. **Check the examples** in the updated documentation
2. **Review the test suite** for usage patterns  
3. **Open an issue** with specific migration questions
4. **Consult TrinityCore** for server-side behavior questions

## Summary

This release improves World of Warcraft file format support with better accuracy, full version handling, and TrinityCore compliance. While there are breaking changes, they fix incorrect implementations and provide more reliable parsing.

The migration effort is primarily around:
- Updating MFBO chunk access patterns  
- Leveraging automatic version detection
- Handling conditional chunk presence appropriately

The result is a more reliable, accurate, and feature-complete warcraft-rs library.