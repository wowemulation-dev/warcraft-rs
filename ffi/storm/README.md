# storm

[![Crates.io](https://img.shields.io/crates/v/storm.svg)](https://crates.io/crates/storm)
[![Documentation](https://docs.rs/storm/badge.svg)](https://docs.rs/storm)
[![License](https://img.shields.io/crates/l/storm.svg)](https://github.com/wowemulation-dev/warcraft-rs#license)

StormLib-compatible C API for the World of Warcraft MPQ archive library.

## Status

✅ **Implemented** - Drop-in StormLib replacement using wow-mpq.

## Overview

The storm crate provides a C-compatible Foreign Function Interface (FFI) that emulates the StormLib API, wrapping the native Rust `wow-mpq` implementation. This allows C/C++ applications that expect StormLib's interface to use the memory-safe Rust implementation instead.

## Features

- StormLib-compatible function signatures
- Thread-safe handle management
- Windows error code compatibility
- Support for MPQ archives from World of Warcraft versions 1.12.1 through 5.4.8
- Both static and dynamic library output

## Supported Versions

- ✅ **Classic** (1.12.1) - Full support
- ✅ **The Burning Crusade** (2.4.3) - Full support
- ✅ **Wrath of the Lich King** (3.3.5a) - Full support
- ✅ **Cataclysm** (4.3.4) - Full support
- ✅ **Mists of Pandaria** (5.4.8) - Full support

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
storm = "0.1"
```

### Building the Library

```bash
# Build the library
cargo build --release -p storm

# Output libraries will be in target/release:
# - libstorm.so (Linux)
# - libstorm.dylib (macOS)
# - storm.dll (Windows)
```

### C/C++ Usage

Include the generated header and link against the storm library:

```c
#include "StormLib.h"

int main() {
    HANDLE hArchive = NULL;

    // Open an archive
    if (SFileOpenArchive("Data/patch.MPQ", 0, 0, &hArchive)) {
        // Check if a file exists
        if (SFileHasFile(hArchive, "Interface\\FrameXML\\GlobalStrings.lua")) {
            // Extract the file
            SFileExtractFile(hArchive,
                "Interface\\FrameXML\\GlobalStrings.lua",
                "GlobalStrings.lua",
                0);
        }

        // Close the archive
        SFileCloseArchive(hArchive);
    }

    return 0;
}
```

### Supported Functions

#### Archive Operations

- `SFileOpenArchive` - Open an existing MPQ archive
- `SFileCreateArchive` - Create a new MPQ archive
- `SFileCloseArchive` - Close an open archive

#### File Operations

- `SFileOpenFileEx` - Open a file within an archive
- `SFileCloseFile` - Close an open file
- `SFileReadFile` - Read data from a file
- `SFileGetFileSize` - Get the size of a file
- `SFileSetFilePointer` - Seek within a file
- `SFileHasFile` - Check if a file exists
- `SFileExtractFile` - Extract a file to disk

#### Archive Information

- `SFileGetArchiveName` - Get the archive file path
- `SFileGetFileName` - Get the current file name
- `SFileGetFileInfo` - Query archive/file information
- `SFileEnumFiles` - Enumerate files in the archive

#### Utility Functions

- `SFileSetLocale` / `SFileGetLocale` - Locale management
- `SFileGetLastError` / `SFileSetLastError` - Error handling
- `SFileVerifyFile` - Verify file integrity
- `SFileSignArchive` - Sign an archive (stub)
- `SFileVerifyArchive` - Verify archive signatures

### Error Handling

The library uses Windows-compatible error codes:

```c
DWORD error = SFileGetLastError();
switch (error) {
    case ERROR_SUCCESS:           // Operation succeeded
        break;
    case ERROR_FILE_NOT_FOUND:    // File not found in archive
        break;
    case ERROR_ACCESS_DENIED:     // I/O error
        break;
    case ERROR_INVALID_PARAMETER: // Invalid parameter
        break;
    case ERROR_FILE_CORRUPT:      // Archive or file corrupt
        break;
}
```

## Examples

See the [examples](examples/) directory for C examples:

- `basic.c` - Simple archive open/close example
- `storm_example.c` - Comprehensive usage example

## Building from Source

### Prerequisites

- Rust 1.86 or later
- C compiler (for examples)
- cbindgen (automatically installed as build dependency)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/wowemulation-dev/warcraft-rs
cd warcraft-rs

# Build the storm crate
cargo build --release -p storm

# Generate C header
cargo build -p storm  # cbindgen runs automatically

# Header will be at: ffi/storm/include/StormLib.h
```

## Compatibility Notes

### StormLib Differences

While the storm crate aims for StormLib compatibility, there are some differences:

- Thread-safe by default (uses Rust's safety guarantees)
- Error codes are translated from wow-mpq's error types
- Some functions are stubs (e.g., `SFileSignArchive`)
- No support for protected MPQs

### Memory Management

- All handles are opaque pointers managed by Rust
- No manual memory management required from C/C++
- Thread-local error storage for compatibility

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
