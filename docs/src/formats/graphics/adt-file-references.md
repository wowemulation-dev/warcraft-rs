# ADT File References

This document details all file references found in ADT (terrain) files and how they reference external assets.

## Overview

ADT files contain references to various external file formats used to construct the game world. These references are stored in specific chunks and use different methods to identify the external files.

## File Reference Chunks

### 1. MTEX - Texture References
- **File Format Referenced**: BLP (texture files)
- **Storage Method**: Null-terminated filename strings
- **Structure**: Sequential list of texture filenames
- **Example**: `Tileset\Elwynn\ElwynnGrass01.blp`

### 2. MMDX - Model References
- **File Format Referenced**: M2 (model files)
- **Storage Method**: Null-terminated filename strings
- **Structure**: Sequential list of model filenames
- **Example**: `World\Azeroth\Elwynn\PassiveDoodads\Trees\ElwynnTree01.m2`

### 3. MMID - Model ID Mapping
- **Purpose**: Maps model instances to filenames in MMDX
- **Storage Method**: 32-bit offsets into the MMDX chunk
- **Usage**: Each offset points to the start of a filename string in MMDX

### 4. MWMO - WMO References
- **File Format Referenced**: WMO (World Map Object files)
- **Storage Method**: Null-terminated filename strings
- **Structure**: Sequential list of WMO filenames
- **Example**: `World\wmo\Azeroth\Buildings\Stormwind\Stormwind.wmo`

### 5. MWID - WMO ID Mapping
- **Purpose**: Maps WMO instances to filenames in MWMO
- **Storage Method**: 32-bit offsets into the MWMO chunk
- **Usage**: Each offset points to the start of a filename string in MWMO

### 6. MDDF - Doodad (M2) Placement
- **Purpose**: Places M2 models in the world
- **Reference Method**: Uses `name_id` field which is an index into MMID
- **Additional Data**: Position, rotation, scale, flags, unique ID
- **Structure**: 
  ```rust
  pub struct DoodadPlacement {
      pub name_id: u32,      // Index into MMID list
      pub unique_id: u32,    // Unique instance identifier
      pub position: [f32; 3],
      pub rotation: [f32; 3],
      pub scale: f32,
      pub flags: u16,
  }
  ```

### 7. MODF - WMO Placement
- **Purpose**: Places WMO objects in the world
- **Reference Method**: Uses `name_id` field which is an index into MWID
- **Additional Data**: Position, rotation, bounding box, flags, doodad set, name set
- **Structure**:
  ```rust
  pub struct ModelPlacement {
      pub name_id: u32,           // Index into MWID list
      pub unique_id: u32,         // Unique instance identifier
      pub position: [f32; 3],
      pub rotation: [f32; 3],
      pub bounds_min: [f32; 3],
      pub bounds_max: [f32; 3],
      pub flags: u16,
      pub doodad_set: u16,
      pub name_set: u16,
      pub padding: u16,
  }
  ```

### 8. MCNK Texture Layers
- **Purpose**: References textures for terrain painting
- **Reference Method**: Uses `texture_id` field which is an index into MTEX
- **Location**: Within MCNK chunks in the MCLY subchunk
- **Structure**:
  ```rust
  pub struct McnkTextureLayer {
      pub texture_id: u32,        // Index into MTEX list
      pub flags: u32,
      pub alpha_map_offset: u32,  // Offset within MCAL chunk
      pub effect_id: u32,         // Reference to texture effect
  }
  ```

### 9. MCNK Object References
- **Doodad References**: 
  - Stored in MCRF subchunk
  - Contains indices into MMID (which map to MMDX filenames)
- **WMO References**:
  - Also stored in MCRF subchunk after doodad references
  - Contains indices into MWID (which map to MWMO filenames)

### 10. MTFX - Texture Effects (Cataclysm+)
- **Purpose**: Defines special effects for textures (e.g., lava, water shaders)
- **Reference Method**: Effect IDs that correspond to shader effects
- **Structure**: List of effect IDs corresponding to each texture in MTEX

## Reference Resolution Process

1. **Texture Resolution**:
   - MCNK chunk contains texture layers with `texture_id`
   - `texture_id` is used as an index into MTEX chunk
   - MTEX contains the actual BLP filename

2. **M2 Model Resolution**:
   - MDDF chunk contains doodad placements with `name_id`
   - `name_id` is used as an index into MMID chunk
   - MMID contains an offset into MMDX chunk
   - MMDX contains the actual M2 filename at that offset

3. **WMO Resolution**:
   - MODF chunk contains WMO placements with `name_id`
   - `name_id` is used as an index into MWID chunk
   - MWID contains an offset into MWMO chunk
   - MWMO contains the actual WMO filename at that offset

## Version-Specific Considerations

- **Vanilla/TBC**: Basic reference system as described above
- **WotLK**: Added MH2O for water (replaces MCLQ in MCNK chunks)
- **Cataclysm+**: 
  - Added MTFX for texture effects
  - Split ADT files may separate object references (_obj0, _obj1) from texture references (_tex0, _tex1)

## File Path Conventions

All file paths in ADT files follow these conventions:
- Use backslashes (`\`) as path separators
- Are relative to the game's data directory
- Do not include file extensions in some cases (game adds .blp for textures automatically)
- Are case-insensitive (game converts to lowercase internally)

## Usage in Code

To access referenced files from an ADT:

```rust
// Get texture filename
let texture_filename = &adt.mtex.as_ref().unwrap().filenames[texture_id as usize];

// Get M2 model filename
let mmid_offset = adt.mmid.as_ref().unwrap().offsets[name_id as usize];
let model_filename = &adt.mmdx.as_ref().unwrap().filenames
    .iter()
    .find(|f| /* find string at mmid_offset */)
    .unwrap();

// Get WMO filename
let mwid_offset = adt.mwid.as_ref().unwrap().offsets[name_id as usize];
let wmo_filename = &adt.mwmo.as_ref().unwrap().filenames
    .iter()
    .find(|f| /* find string at mwid_offset */)
    .unwrap();
```