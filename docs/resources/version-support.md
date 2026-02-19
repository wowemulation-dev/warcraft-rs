# Version Support

World of Warcraft client version compatibility and file format changes.

## Supported Client Versions

| Expansion | Version | Patch | Build | Status |
|-----------|---------|-------|-------|--------|
| Classic (Vanilla) | 1.12.1 | 1.12.1.5875 | 5875 | ✅ Supported |
| The Burning Crusade | 2.4.3 | 2.4.3.8606 | 8606 | ✅ Supported |
| Wrath of the Lich King | 3.3.5 | 3.3.5a.12340 | 12340 | ✅ Supported |
| Cataclysm | 4.3.4 | 4.3.4.15595 | 15595 | ✅ Supported |
| Mists of Pandaria | 5.4.8 | 5.4.8.18414 | 18414 | ✅ Supported |

## File Format Versions

### MPQ Archives

| Version | Client | Changes | wow-mpq Support |
|---------|--------|---------|-----------------|
| v1 | 1.x - 3.x | Original format, hash table, block table | ✅ Supported |
| v2 | 3.x+ | Extended attributes, larger files | ✅ Supported |
| v3 | 4.x+ | HET/BET tables, increased hash table size | ✅ Supported |
| v4 | 5.x+ | 64-bit file support, MD5 checksums | ✅ Supported |

**Note:** wow-mpq has bidirectional compatibility with StormLib (the reference C++ implementation) and support for official Blizzard WoW archives.

### M2 Models

| Version | Client | Major Changes |
|---------|--------|---------------|
| 256-257 | 1.x | Original format |
| 260-263 | 2.x | Particle emitters update |
| 264 | 3.0+ | .skin/.anim file separation |
| 272 | 3.3+ | Extended animations |
| 273 | 4.0+ | .phys physics data |
| 274 | 4.x+ | New texture types |
| 276 | 5.x+ | Improved bone structure |

### ADT Terrain

| Version | Client | Changes |
|---------|--------|---------|
| 18 | 1.x - 2.x | Original MCNK format |
| 20 | 3.x | Destructible doodads |
| 21 | 4.x | Terrain streaming, flight |
| 23 | 5.x | New texture blending |

### BLP Textures

| Version | Client | Format Support |
|---------|--------|----------------|
| BLP1 | 1.x - 2.x | JPEG compression, palettized |
| BLP2 | 3.x+ | DXT compression, mipmaps |

### DBC Database

| Client | Records | String Encoding | Features |
|--------|---------|-----------------|----------|
| 1.x | Fixed size | ASCII | Basic structure |
| 2.x | Fixed size | UTF-8 | Extended fields |
| 3.x | Fixed size | UTF-8 | Localization support |
| 4.x | Fixed size | UTF-8 | New index format |
| 5.x | Fixed size | UTF-8 | Compressed strings |

## Version Detection

Each crate handles version detection differently:

- **MPQ**: Format version is read from the archive header (v1-v4)
- **M2**: Header version field distinguishes expansions; MD20 vs MD21 magic separates legacy from chunked format
- **ADT**: MVER chunk is always 18; actual version is detected from chunk presence (MFBO, MH2O, MAMP, MTXP)
- **BLP**: File magic is `BLP1` or `BLP2`
- **WMO**: MVER chunk version number increases with expansions (17-27)
- **WDT/WDL**: Version detection via chunk analysis

### File Magic Numbers

| Format | Magic | Notes |
|--------|-------|-------|
| MPQ | `MPQ\x1A` | All versions |
| M2 (Legacy) | `MD20` | Pre-Legion |
| M2 (Chunked) | `MD21` | Legion+ |
| BLP1 | `BLP1` | Classic, TBC |
| BLP2 | `BLP2` | WotLK+ |
| WMO/ADT/WDT/WDL | `RVER` (MVER reversed) | Chunk-based |

## Best Practices

1. Use each crate's version enum for version-aware code
2. Let the parser detect versions automatically where possible
3. Test with files from multiple WoW client versions
4. Check optional chunk presence rather than assuming version

## See Also

- [File Format Reference](../formats/README.md)
