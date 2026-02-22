# Error Handling

Each crate in warcraft-rs defines its own error type using `thiserror`. There is
no shared error type across crates.

## Error Types by Crate

| Crate | Error Type | Module |
|-------|-----------|--------|
| wow-mpq | `Error` | `wow_mpq::error` |
| wow-blp | `LoadError`, `EncodeError`, `ConvertError` | `wow_blp::parser`, `wow_blp::encode`, `wow_blp::convert` |
| wow-m2 | `M2Error` | `wow_m2::error` |
| wow-wmo | `WmoError` | `wow_wmo::error` |
| wow-adt | `AdtError` | `wow_adt::error` |
| wow-wdl | `WdlError` | `wow_wdl::error` |
| wow-wdt | `Error` | `wow_wdt::error` |
| wow-cdbc | `Error` | `wow_cdbc` (re-exported at crate root) |

## Pattern

All error types follow the same pattern:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    // Format-specific variants...
}

pub type Result<T> = std::result::Result<T, Error>;
```

Each crate also provides a `Result<T>` type alias for convenience.

## Usage

```rust
use wow_mpq::Archive;

fn read_file() -> Result<Vec<u8>, wow_mpq::error::Error> {
    let mut archive = Archive::open("archive.mpq")?;
    let data = archive.read_file("Interface/FrameXML/UIParent.lua")?;
    Ok(data)
}
```

The CLI application (`warcraft-rs`) uses `anyhow` for error handling, converting
library errors via the `?` operator.

## See Also

- [Core Types](core-types.md)
- [Traits & Interfaces](traits.md)
