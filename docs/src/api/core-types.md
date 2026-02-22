# Core Types

warcraft-rs does not have a shared core types library. Each crate defines its own
types independently. This page documents common patterns across crates.

## Math Types

The project uses [glam](https://docs.rs/glam) for vector and matrix math.
Individual crates define their own geometry structs for binary-compatible
parsing (e.g., `C3Vector`, `C4Quaternion` in wow-m2).

## Version Enums

Each crate that handles versioned formats defines its own version enum:

| Crate | Version Type | Variants |
|-------|-------------|----------|
| wow-mpq | `FormatVersion` | V1, V2, V3, V4 |
| wow-m2 | `M2Version` | Vanilla, TBC, WotLK, Cataclysm, MoP, WoD, Legion, BfA, Shadowlands, Dragonflight, TheWarWithin |
| wow-wmo | `WmoVersion` | Classic, Tbc, Wotlk, Cataclysm, Mop, Wod, Legion, Bfa, Shadowlands, Dragonflight, WarWithin |
| wow-wdt | `WowVersion` | Classic, TBC, WotLK, Cataclysm, MoP, WoD, Legion, BfA, Shadowlands, Dragonflight |
| wow-adt | `AdtVersion` | VanillaEarly, VanillaLate, TBC, WotLK, Cataclysm, MoP |
| wow-wdl | `WdlVersion` | Vanilla, Wotlk, Cataclysm, Mop, Wod, Legion, Bfa, Shadowlands, Dragonflight, Latest |

## Chunk-based Parsing

Several WoW file formats use a chunk-based structure with four-character codes
(FourCC). The project handles this differently per crate:

- **wow-wdt**: Defines a `Chunk` trait that chunk types implement
- **wow-adt, wow-wmo**: Uses binrw 0.14 with declarative chunk parsing
- **wow-wdl**: Custom parser that reads chunks sequentially
- **wow-m2**: Custom `ReadExt` trait for reading binary data

## Flag Types

The project uses the [bitflags](https://docs.rs/bitflags) crate for type-safe
flag handling in format-specific contexts.

## Serialization

Crates that support serialization use [serde](https://docs.rs/serde) behind
feature flags:

- wow-m2: `serde-support` feature
- wow-wdt: `serde` feature
- wow-cdbc: `serde` feature (also `csv_export`, `yaml`)

## API Reference

For detailed type documentation, see the per-crate API docs on docs.rs:

- [wow-mpq](https://docs.rs/wow-mpq)
- [wow-blp](https://docs.rs/wow-blp)
- [wow-m2](https://docs.rs/wow-m2)
- [wow-wmo](https://docs.rs/wow-wmo)
- [wow-adt](https://docs.rs/wow-adt)
- [wow-wdl](https://docs.rs/wow-wdl)
- [wow-wdt](https://docs.rs/wow-wdt)
- [wow-cdbc](https://docs.rs/wow-cdbc)

## See Also

- [Error Handling](error-handling.md)
- [Traits & Interfaces](traits.md)
