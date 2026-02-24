# Archive Formats

Archive formats are used to store and compress World of Warcraft game assets.

## Supported Formats

### [MPQ Format](mpq.md)

The primary archive format used by Blizzard games. MPQ archives can contain any
type of game asset including models, textures, sounds, and data files.

**Key Features:**

- Multiple compression algorithms (ZLIB, PKWare, BZip2, LZMA)
- File encryption support
- Patching capability
- Efficient hash-based file lookup

## Common Use Cases

### Extracting Game Assets

```rust
use wow_mpq::Archive;

let mut archive = Archive::open("Data/art.mpq")?;
let entries = archive.list()?;
for entry in entries {
    println!("{}: {} bytes", entry.name, entry.size);
}
```

### Working with Patches

WoW uses a patch chain system where newer MPQs override files in older ones:

- `base-*.mpq` - Base game files
- `patch-*.mpq` - Incremental patches
- `locale-*.mpq` - Localization files

## Tools

- `warcraft-rs mpq` - CLI commands for MPQ operations
- Ladik's MPQ Editor - Popular Windows MPQ editor

## Performance Tips

1. **Caching**: Cache frequently accessed files in memory
2. **Streaming**: Read files on demand rather than extracting all at once
3. **Parallel Extraction**: Use `--threads` flag in CLI for concurrent extraction
4. **Compression**: Choose appropriate compression for your use case

## See Also

- [Working with MPQ Archives Guide](../../guides/mpq-archives.md)
- [MPQ Format Specification](mpq.md)
