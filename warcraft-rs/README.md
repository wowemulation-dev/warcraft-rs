# warcraft-rs CLI

Unified command-line tool for working with World of Warcraft file formats.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/wowemulation-dev/warcraft-rs
cd warcraft-rs/warcraft-rs

# Build with all features
cargo build --release --features full

# Or build with specific features only
cargo build --release --features "mpq dbc"

# The binary will be at target/release/warcraft-rs
```

### Using Cargo

```bash
# Install with all formats
cargo install --path . --features full

# Or with specific formats only
cargo install --path . --features "mpq blp"
```

## Usage

The `warcraft-rs` CLI provides subcommands for each supported file format:

```bash
warcraft-rs <format> <command> [options]
```

### Available Formats

- `mpq` - MPQ archive operations (implemented)
- `dbc` - DBC database operations (planned)
- `blp` - BLP texture operations (planned)
- `m2` - M2 model operations (planned)
- `wmo` - WMO object operations (planned)
- `adt` - ADT terrain operations (planned)
- `wdt` - WDT map operations (planned)
- `wdl` - WDL world operations (implemented)

### MPQ Commands

```bash
# List files in an archive
warcraft-rs mpq list archive.mpq
warcraft-rs mpq list archive.mpq --long
warcraft-rs mpq list archive.mpq --filter "*.dbc"

# Extract files
warcraft-rs mpq extract archive.mpq
warcraft-rs mpq extract archive.mpq --output ./extracted
warcraft-rs mpq extract archive.mpq file1.txt file2.dat

# Create a new archive
warcraft-rs mpq create new.mpq --add file1.txt --add file2.dat
warcraft-rs mpq create new.mpq --add *.txt --version v2 --compression zlib

# Show archive information
warcraft-rs mpq info archive.mpq
warcraft-rs mpq info archive.mpq --show-hash-table

# Verify archive integrity
warcraft-rs mpq verify archive.mpq
```

### Global Options

- `-v, --verbose` - Increase verbosity (can be repeated)
- `-q, --quiet` - Suppress all output except errors
- `--help` - Show help for any command

### Shell Completions

Generate shell completions for your shell:

```bash
# Bash
warcraft-rs completions bash > ~/.local/share/bash-completion/completions/warcraft-rs

# Zsh
warcraft-rs completions zsh > ~/.zfunc/_warcraft-rs

# Fish
warcraft-rs completions fish > ~/.config/fish/completions/warcraft-rs.fish

# PowerShell
warcraft-rs completions powershell > _warcraft-rs.ps1
```

## Features

The CLI can be built with different feature flags to include only the formats you need:

- `default` - Includes MPQ support only
- `full` - Includes all format support
- `mpq` - MPQ archive support
- `dbc` - DBC database support
- `blp` - BLP texture support
- `m2` - M2 model support
- `wmo` - WMO object support
- `adt` - ADT terrain support
- `wdt` - WDT map support
- `wdl` - WDL world support

## Examples

### Working with MPQ Archives

```bash
# Extract all DBC files from an MPQ
warcraft-rs mpq list patch.mpq --filter "*.dbc" | \
  xargs warcraft-rs mpq extract patch.mpq --output ./dbc_files

# Create a new MPQ with compressed files
warcraft-rs mpq create my_mod.mpq \
  --add data/*.txt \
  --add scripts/*.lua \
  --compression zlib \
  --with-listfile

# Verify multiple archives
for mpq in *.mpq; do
  echo "Checking $mpq..."
  warcraft-rs mpq verify "$mpq"
done
```

### Future Format Support

Once implemented, other formats will follow similar patterns:

```bash
# Convert BLP textures to PNG
warcraft-rs blp convert texture.blp --to png

# Export DBC to JSON
warcraft-rs dbc export Items.dbc --format json --output items.json

# Get model information
warcraft-rs m2 info character.m2
```

## Development

See the main [warcraft-rs](https://github.com/wowemulation-dev/warcraft-rs) repository for development information.
