# Traits & Interfaces

warcraft-rs does not define shared traits across crates. Each crate has its own
parsing and I/O approach. This page documents the common patterns.

## Parsing Patterns

Three parsing approaches are used across the workspace:

### binrw Declarative (wow-adt, wow-wmo, wow-cdbc)

Uses derive macros for binary parsing:

```rust
use binrw::BinRead;

#[derive(BinRead, Debug)]
#[br(little)]
pub struct ChunkHeader {
    pub magic: [u8; 4],
    pub size: u32,
}
```

wow-adt and wow-wmo use binrw 0.14. wow-cdbc uses binrw 0.15.

### Chunk Trait (wow-wdt)

Defines a `Chunk` trait that format-specific chunk types implement:

```rust
pub trait Chunk: Sized {
    fn id() -> &'static [u8; 4];
    fn read<R: Read + Seek>(reader: &mut R, size: u32) -> Result<Self>;
}
```

### Hand-Written Readers (wow-mpq, wow-blp, wow-wdl, wow-m2)

Custom byte-level parsing using `Read` + `Seek` traits:

```rust
// wow-m2 uses a ReadExt trait
pub trait ReadExt: Read {
    fn read_u32_le(&mut self) -> io::Result<u32>;
    fn read_f32_le(&mut self) -> io::Result<f32>;
    // ...
}
```

## Common API Patterns

### Open/Parse Pattern

Most crates provide a way to parse from a reader or file:

```rust
// wow-mpq: static open method
let archive = Archive::open("archive.mpq")?;

// wow-wdt: reader struct
let reader = WdtReader::new(BufReader::new(file), WowVersion::WotLK);
let wdt = reader.read()?;

// wow-m2: parse from reader
let model = M2Model::parse(&mut cursor)?;

// wow-adt: standalone function
let parsed = parse_adt(&mut reader)?;
```

### Writer Pattern

Crates with write support provide builder or writer types:

```rust
// wow-mpq: Archive with create/add methods
let mut archive = Archive::create("new.mpq")?;

// wow-wdt: WdtWriter
let writer = WdtWriter::new(WowVersion::WotLK);
writer.write(&wdt, &mut output)?;

// wow-cdbc: DbcWriter
let writer = DbcWriter::new();
```

## See Also

- [Core Types](core-types.md)
- [Error Handling](error-handling.md)
