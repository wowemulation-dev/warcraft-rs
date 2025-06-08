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

### From crates.io (Recommended)

Add warcraft-rs to your project:

```toml
[dependencies]
warcraft-rs = "0.1.0"
```

Or install individual crates:

```toml
[dependencies]
wow-mpq = "0.1.0"
wow-blp = "0.1.0"
wow-m2 = "0.1.0"
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

# Install CLI tools
cargo install --path crates/tools/warcraft-cli
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
warcraft-rs = "0.1.0"
```

Create `src/main.rs`:

```rust
use warcraft_rs::mpq::Archive;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("warcraft-rs installed successfully!");
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
