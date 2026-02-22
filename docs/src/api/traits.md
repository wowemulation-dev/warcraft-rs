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
    fn magic() -> &'static [u8; 4];
    fn expected_size() -> Option<usize> { None }
    fn read(reader: &mut impl Read, size: usize) -> Result<Self>;
    fn write(&self, writer: &mut impl Write) -> Result<()>;
    fn size(&self) -> usize;
    fn write_chunk(&self, writer: &mut impl Write) -> Result<()> { /* default impl */ }
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

// wow-m2: load returns M2Format (Legacy or Chunked variant)
let format = M2Model::load("model.m2")?;
let model = format.model();

// wow-adt: standalone function
let parsed = parse_adt(&mut reader)?;
```

### Writer Pattern

Crates with write support provide builder or writer types:

```rust
// wow-mpq: Archive with create/add methods
let mut archive = Archive::create("new.mpq")?;

// wow-wdt: WdtWriter wraps a writer
let writer = WdtWriter::new(&mut output);
writer.write(&wdt)?;

// wow-cdbc: DbcWriter wraps a writer
let writer = DbcWriter::new(&mut output);
```

## See Also

- [Core Types](core-types.md)
- [Error Handling](error-handling.md)
