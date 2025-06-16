# 📦 Installation Guide

This guide will walk you through installing `warcraft-rs` and its dependencies.

## Prerequisites

### Required

- **Rust**: Version 1.86 or later
  - Install from [rustup.rs](https://rustup.rs/)
  - Verify: `rustc --version`
- **Git**: For cloning the repository
  - [Download Git](https://git-scm.com/downloads)

### Optional

- **Cross**: For cross-platform builds

  ```bash
  cargo install cross
  ```

- **cargo-workspaces**: For workspace management

  ```bash
  cargo install cargo-workspaces
  ```

## Installation Methods

### From crates.io

#### Install the CLI Tool

```bash
cargo install warcraft-rs
```

#### Add Individual Crates

```toml
[dependencies]
wow-mpq = "0.1.0"    # MPQ archive support
wow-blp = "0.1.0"    # BLP texture support
wow-adt = "0.1.0"    # ADT terrain support
wow-wdl = "0.1.0"    # WDL low-resolution terrain support
wow-wdt = "0.1.0"    # WDT map definition support
wow-wmo = "0.1.0"    # WMO world map object support
wow-m2 = "0.1.0"     # M2 model support
wow-cdbc = "0.1.0"   # DBC database support
```

Or use cargo add:

```bash
cargo add wow-mpq wow-blp wow-adt
```

### From Source

Clone and build the repository:

```bash
# Clone the repository
git clone https://github.com/wowemulation-dev/warcraft-rs.git
cd warcraft-rs

# Build all crates
cargo build --release

# Run tests
cargo test

# Install CLI tool with all features
cargo install --path warcraft-rs --features full

# Or install with specific features only
cargo install --path warcraft-rs --features "mpq wdl wdt"
```

### Development Setup

For contributing to warcraft-rs:

```bash
# Clone with full history
git clone https://github.com/wowemulation-dev/warcraft-rs.git
cd warcraft-rs

# Setup pre-commit hooks
cp .githooks/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit

# Verify setup
cargo fmt --all -- --check
cargo check --all-features --all-targets
cargo clippy --all-targets --all-features
cargo test
```

## Feature Flags

The `warcraft-rs` CLI supports feature flags to include only the formats you need:

```bash
# Default build (all formats included)
cargo build --release

# Build with all features including extras
cargo build --release --features full

# Build with specific features only
cargo build --release --no-default-features --features "mpq wdl wdt"
cargo run --features wdt -- wdt info map.wdt
```

Available features:

- `mpq` - MPQ archive support (always available)
- `dbc` - DBC database support (enabled by default)
- `blp` - BLP texture support (enabled by default)
- `m2` - M2 model support (enabled by default)
- `wmo` - WMO object support (enabled by default)
- `adt` - ADT terrain support (enabled by default)
- `wdt` - WDT map definition support (enabled by default)
- `wdl` - WDL low-resolution terrain support (enabled by default)
- `serde` - JSON/YAML serialization support
- `extract` - ADT data extraction features
- `parallel` - Parallel processing support
- `yaml` - YAML support for DBC schemas
- `full` - All features including extras

## Platform-Specific Notes

### Windows

- Visual Studio Build Tools or full Visual Studio required
- Use PowerShell or cmd for commands
- Paths use backslashes: `World\Maps\Azeroth`

### macOS

- Xcode Command Line Tools required:

  ```bash
  xcode-select --install
  ```

- Case-sensitive filesystem recommended

### Linux

- Development packages may be needed:

  ```bash
  # Ubuntu/Debian
  sudo apt-get install build-essential

  # Fedora
  sudo dnf install gcc
  ```

## Verifying Installation

Create a test project:

```bash
cargo new wow-test
cd wow-test
```

Add to `Cargo.toml`:

```toml
[dependencies]
wow-mpq = "0.1.0"
```

Create `src/main.rs`:

```rust
use wow_mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("wow-mpq installed successfully!");

    // Test opening an MPQ if you have one
    if let Ok(mut archive) = Archive::open("path/to/archive.mpq") {
        println!("Successfully opened MPQ archive!");
    }

    Ok(())
}
```

Build and run:

```bash
cargo run
```

## Troubleshooting

### Common Issues

#### Rust Version Error

```text
error: package `warcraft-rs v0.1.0` cannot be built because it requires rustc 1.86 or newer
```

**Solution**: Update Rust with `rustup update`

#### Missing Dependencies

```text
error: linker `cc` not found
```

**Solution**: Install platform build tools (see platform notes above)

#### Out of Memory

```text
error: could not compile `warcraft-rs` (bin "warcraft-rs") due to previous error
```

**Solution**: Close other applications or add swap space

### Getting Help

- Check [Troubleshooting Guide](troubleshooting.md)
- Open an [issue on GitHub](https://github.com/wowemulation-dev/warcraft-rs/issues)
- Join our [Discord community](https://discord.gg/warcraft-rs)

## Next Steps

- Read the [Quick Start Guide](quick-start.md)
- Explore [Basic Usage](basic-usage.md)
- Browse [Example Projects](https://github.com/wowemulation-dev/warcraft-rs/tree/main/examples)
