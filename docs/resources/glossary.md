# 📖 Glossary

Common terms and abbreviations used in World of Warcraft file formats.

## File Format Terms

### ADT

**Azeroth Data Terrain** - Map tile files containing terrain mesh, textures, and
object placement data.

### BLP

**Blizzard Picture** - Proprietary texture format supporting DXT compression and
mipmaps.

### DBC

**DataBase Client** - Client-side database files storing game data in a tabular
format.

### M2

**Model Version 2** - 3D model format for characters, creatures, and doodads.
Includes animations, bones, and particle effects.

### MPQ

**Mo'PaQ** (Mike O'Brien Pack) - Archive format for storing game assets with
compression and encryption. The `wow-mpq` implementation has 98.75% compatibility
with StormLib (the reference C++ implementation).

### WDL

**World Data Low-resolution** - Low-detail terrain for distant rendering and minimaps.

### WDT

**World Data Table** - Map definition files that reference ADT tiles and define
map properties.

### WMO

**World Map Object** - Large static models like buildings, dungeons, and cities.

## Technical Terms

### Chunk

A data block in a file, usually identified by a 4-character ID (e.g., MVER, MHDR).

### Doodad

Small decorative objects like trees, rocks, and furniture (stored as M2 models).

### FOURCC

Four-Character Code - A 32-bit identifier used for chunk types (e.g., 'MCNK').

### Heightmap

2D array of elevation values defining terrain height.

### Listfile

Text file mapping file hashes to filenames in MPQ archives.

### Mipmap

Progressively smaller versions of a texture for efficient rendering at different
distances.

### Skinning

Process of binding mesh vertices to bones for animation.

### UV Mapping

Texture coordinate system mapping 2D textures to 3D surfaces.

### Vertex

A point in 3D space with position, normal, texture coordinates, and other attributes.

## Compression Algorithms

### ZLIB

Standard compression library used in MPQ archives.

### PKWARE

PKZip compression algorithm, also used in MPQ.

### BZip2

Block-sorting compression algorithm for better ratios.

### LZMA

Lempel-Ziv-Markov chain algorithm for high compression.

## Game-Specific Terms

### Expansion IDs

- 0: Classic (Vanilla)
- 1: The Burning Crusade (TBC)
- 2: Wrath of the Lich King (WotLK)
- 3: Cataclysm
- 4: Mists of Pandaria (MoP)

### Map IDs

- 0: Eastern Kingdoms
- 1: Kalimdor
- 530: Outland
- 571: Northrend

### Coordinate System

- WoW uses a Y-up coordinate system
- Maps are divided into 64x64 ADT grid
- Each ADT is 533.33 yards square

## Common Patterns

### Magic Numbers

File signatures used to identify format:

- MPQ: `MPQ\x1A` (0x1A51504D)
- BLP: `BLP2` (0x32504C42)
- M2: `MD20` (0x3032444D)
- WMO: `MVER` chunk at start

### Byte Order

Most WoW files use little-endian byte order.

### String Encoding

- Filenames: Usually ASCII or UTF-8
- Localized text: UTF-8 with locale-specific DBCs
