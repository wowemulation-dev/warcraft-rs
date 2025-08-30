# M2 Version Findings

## Summary of M2 Format Changes Across WoW Versions

Based on systematic testing across WoW versions 1.12.1 through 5.4.8, here are the key findings:

### Version Distribution

| WoW Version | Expansion | M2 Version | Skin Storage |
|------------|-----------|------------|--------------|
| 1.12.1 | Vanilla | 256 | Embedded in M2 |
| 2.4.3 | TBC | 260 | Embedded in M2 |  
| 3.3.5a | WotLK | 264 | External .skin files |
| 4.3.4 | Cataclysm | 272 | External .skin files |
| 5.4.8 | MoP | 272 | External .skin files |

### Key Format Changes

#### Pre-WotLK (Versions ≤ 260)
- **Embedded Skins**: All skin profile data stored within the M2 file
- **Single File**: No external dependencies for model geometry
- **Views Array**: Points to embedded M2SkinProfile structures within the M2 file
- **Offset Interpretation**: All offsets are relative to the M2 file start

#### WotLK and Later (Versions ≥ 264)  
- **External Skins**: Skin profiles moved to separate .skin files
- **Multiple Files Required**: 
  - Main .m2 file contains bones, animations, textures
  - .skin files (ModelName00.skin, ModelName01.skin, etc.) contain geometry LODs
- **Views Array**: Now just a count; actual skin data in external files
- **Offset Interpretation**: Offsets in .skin files are relative to that .skin file

### Critical Implementation Issues Found

1. **Python Parser Issues with Embedded Skins**:
   - The M2SkinProfile structure at views offset appears corrupted
   - Getting invalid values like `submeshes=M2Array(count=196609, offset=0)`
   - Likely reading at wrong offset or structure layout differs for embedded skins

2. **Missing .skin File Extraction**:
   - WotLK+ models show version 272 but .skin files aren't being extracted
   - The .skin files may be in different MPQ archives or use different naming

3. **Parser Timeout Issues**:
   - Large character models (2.5MB+) cause parser timeouts
   - Need optimization for handling large embedded skin data

### Rust Parser Success vs Python Parser Issues

The Rust parser correctly:
- Identifies 4 skin profiles in HumanMale.m2
- Produces valid triangle indices: [428, 429, 562, ...]
- Handles both embedded and external skin formats

The Python parser struggles with:
- Reading correct M2SkinProfile structure for embedded skins
- Interpreting the views array offset correctly
- Handling the different layouts between versions

### Next Steps

1. **Fix Embedded Skin Parsing**: 
   - Research actual memory layout of embedded M2SkinProfile
   - The views offset may not directly point to M2SkinProfile structure

2. **Improve .skin File Extraction**:
   - Check if .skin files use different naming conventions
   - They may be in separate MPQ archives (e.g., patch MPQs)

3. **Cross-Reference with WMVx**:
   - WMVx only uses views[0] for all skin profiles
   - This suggests all LODs share the same base geometry structure

4. **Version-Specific Handling**:
   - Implement separate parsing paths for embedded vs external skins
   - Handle version-specific structure differences