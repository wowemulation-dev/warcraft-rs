# warcraft-rs Documentation

This documentation covers all supported World of Warcraft file formats and
provides examples for parsing and handling them.

## Getting Started

- [Quick Start Guide](getting-started/quick-start.md) - Get up and running quickly
- [Installation](getting-started/installation.md) - Detailed installation
  instructions
- [Basic Usage](getting-started/basic-usage.md) - Common usage patterns

## File Formats

### Archives

- [MPQ Format](formats/archives/mpq.md) - Blizzard's archive format
  (100% StormLib compatible)

### Terrain and World Data

- [ADT Format](formats/world-data/adt.md) - Terrain tiles
- [WDL Format](formats/world-data/wdl.md) - Low-resolution world maps
- [WDT Format](formats/world-data/wdt.md) - World tables

### Graphics & Models

- [BLP Format](formats/graphics/blp.md) - Texture format
- [M2 Format](formats/graphics/m2.md) - 3D models
  - [.anim Files](formats/graphics/m2-anim.md) - Animation data
  - [.skin Files](formats/graphics/m2-skin.md) - Mesh data
  - [.phys Files](formats/graphics/m2-phys.md) - Physics data
- [WMO Format](formats/graphics/wmo.md) - World map objects

### Client Database

- [DBC Format](formats/database/dbc.md) - Client database files

## Guides

### MPQ Archives

- [Working with MPQ Archives](guides/mpq-archives.md)
- [MPQ CLI Usage](guides/mpq-cli-usage.md)
- [MPQ Digital Signatures](guides/mpq-signatures.md)
- [StormLib vs wow-mpq Differences](guides/stormlib-differences.md)
- [WoW Patch Chain Summary](guides/wow-patch-chain-summary.md)

### Terrain and World Tools

- [ADT CLI Usage](guides/adt-cli-usage.md)
- [WDT CLI Usage](guides/wdt-cli-usage.md)

### Graphics & Rendering

- [Rendering ADT Terrain](guides/adt-rendering.md)
- [Loading M2 Models](guides/m2-models.md)
- [Model Rendering Guide](guides/model-rendering.md)
- [WMO Rendering Guide](guides/wmo-rendering.md)
- [Texture Loading](guides/texture-loading.md)
- [Animation System](guides/animation-system.md)
- [LOD System](guides/lod-system.md)

### Client Database Tools

- [DBC Data Extraction](guides/dbc-extraction.md)

## API Reference

- [Core Types](api/core-types.md)
- [Error Handling](api/error-handling.md)
- [Traits & Interfaces](api/traits.md)

## Resources

- [Glossary](resources/glossary.md) - Common terms and abbreviations
- [Version Support](resources/version-support.md) - WoW version compatibility
- [External Links](resources/links.md) - Helpful external resources

## Contributing

See our [Contributing Guide](../CONTRIBUTING.md) for information on how to
contribute to this project.
