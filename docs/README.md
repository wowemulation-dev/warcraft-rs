# 📚 warcraft-rs Documentation

Welcome to the `warcraft-rs` documentation! This guide covers all supported
World of Warcraft file formats and provides examples for parsing and handling
them.

## 🚀 Getting Started

- [Quick Start Guide](getting-started/quick-start.md) - Get up and running quickly
- [Installation](getting-started/installation.md) - Detailed installation instructions
- [Basic Usage](getting-started/basic-usage.md) - Common usage patterns

## 📁 File Formats

### Archives

- [MPQ Format](formats/archives/mpq.md) - Blizzard's archive format

### World Data

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

### Database

- [DBC Format](formats/database/dbc.md) - Client database files

## 📖 Guides

- [Working with MPQ Archives](guides/mpq-archives.md)
- [StormLib vs wow-mpq Differences](guides/stormlib-differences.md)
- [Rendering ADT Terrain](guides/adt-rendering.md)
- [Loading M2 Models](guides/m2-models.md)
- [DBC Data Extraction](guides/dbc-extraction.md)

## 🔧 API Reference

- [Core Types](api/core-types.md)
- [Error Handling](api/error-handling.md)
- [Traits & Interfaces](api/traits.md)

## 📚 Resources

- [Glossary](resources/glossary.md) - Common terms and abbreviations
- [Version Support](resources/version-support.md) - WoW version compatibility
- [External Links](resources/links.md) - Helpful external resources

## 🤝 Contributing

See our [Contributing Guide](../CONTRIBUTING.md) for information on how to
contribute to this project.
