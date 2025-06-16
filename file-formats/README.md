# World of Warcraft File Formats

This directory contains all file format parsing and handling crates for World of
Warcraft, organized by category.

## 📂 Directory Structure

```text
file-formats/
├── archives/      # Archive formats
│   └── wow-mpq    # MPQ (Mike O'Brien Pack) archives
├── world-data/    # World and terrain data
│   ├── wow-adt    # ADT (Terrain) files
│   ├── wow-wdl    # WDL (Low-resolution world maps)
│   └── wow-wdt    # WDT (World map definitions)
├── graphics/      # Graphics and model formats
│   ├── wow-blp    # BLP (Texture) files
│   ├── wow-m2     # M2 (Model) files
│   └── wow-wmo    # WMO (World Map Object) files
└── database/      # Game data storage
    └── wow-cdbc   # cDBC (Database Client) files
```

## 🎯 Format Categories

### Archives

- **MPQ** - The primary archive format used by World of Warcraft for storing game
  assets

### World Data

- **ADT** - Terrain data files containing height maps, textures, and objects
- **WDL** - Low-resolution world maps used for distant terrain rendering
- **WDT** - World definition tables that define which ADT tiles exist

### Graphics

- **BLP** - Blizzard's proprietary texture format
- **M2** - 3D models for characters, creatures, and objects
- **WMO** - Large world objects like buildings and dungeons

### Database

- **cDBC** - Client-side database files containing game data

## 🔧 Usage

Each crate can be used independently:

```toml
[dependencies]
wow-mpq = { path = "file-formats/archives/wow-mpq" }
wow-cdbc = { path = "file-formats/database/wow-cdbc" }
wow-blp = { path = "file-formats/graphics/wow-blp" }
```

## 📖 Documentation

See the individual README files in each crate for format-specific documentation
and usage examples.
