# 🚀 Quick Start Guide

Get started with `warcraft-rs` in just a few minutes!

## Prerequisites

- Rust 1.86 or later
- Basic familiarity with Rust
- WoW game files to parse

## Installation

Add `warcraft-rs` to your `Cargo.toml`:

```toml
[dependencies]
warcraft-rs = "0.1.0"
```

## Basic Example

Here's a simple example of reading an MPQ archive:

```rust
use warcraft_rs::mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an MPQ archive
    let archive = Archive::open("path/to/file.mpq")?;

    // List files in the archive
    for file in archive.files() {
        println!("Found file: {}", file.name());
    }

    // Extract a specific file
    let data = archive.read_file("Interface\\Icons\\INV_Misc_Bag_07.blp")?;

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
- Read the [troubleshooting guide](troubleshooting.md)
