# Quick Start Guide

Get started with `warcraft-rs` in just a few minutes!

## Prerequisites

- Rust 1.92 or later
- Basic familiarity with Rust
- WoW game files to parse

## Installation

Add the specific crates you need to your `Cargo.toml`:

```toml
[dependencies]
wow-mpq = "0.6"    # For MPQ archive support
wow-wdt = "0.6"    # For WDT world table files
wow-wdl = "0.6"    # For WDL low-resolution heightmaps
wow-cdbc = "0.6"   # For DBC database files
wow-blp = "0.6"    # For BLP textures
# Add other crates as needed
```

Or install the CLI tool:

```bash
# From crates.io
cargo install warcraft-rs

# From source
git clone https://github.com/wowemulation-dev/warcraft-rs
cd warcraft-rs
cargo install --path warcraft-rs
```

## Basic Example

Here's a simple example of reading an MPQ archive:

```rust
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an MPQ archive
    let mut archive = Archive::open("path/to/file.mpq")?;

    // List files in the archive (requires listfile)
    if let Ok(entries) = archive.list() {
        for entry in entries {
            println!("Found file: {} ({} bytes)", entry.name, entry.size);
        }
    }

    // Extract a specific file
    // Note: Both forward and backslashes work - they're automatically converted
    let data = archive.read_file("Interface/Icons/INV_Misc_Bag_07.blp")?;
    // or: archive.read_file("Interface\\Icons\\INV_Misc_Bag_07.blp")?;

    println!("Extracted {} bytes", data.len());

    Ok(())
}
```

## Next Steps

- [Installation Guide](installation.md) - Detailed setup instructions
- [Basic Usage](basic-usage.md) - More examples and patterns
- [File Format Reference](../formats/README.md) - Detailed format documentation

## Getting Help

- Check the [API documentation](https://docs.rs/warcraft-rs)
- Visit our [GitHub repository](https://github.com/wowemulation-dev/warcraft-rs)
- Read troubleshooting documentation
