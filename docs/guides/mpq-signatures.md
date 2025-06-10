# MPQ Digital Signatures Guide

This guide covers the digital signature functionality in MPQ archives, including
verification and generation of signatures for archive integrity protection.

## Overview

MPQ archives support digital signatures to ensure file integrity and authenticity.
There are two types of signatures:

### Weak Signatures (MPQ v1+)

- **Algorithm**: 512-bit RSA with MD5 hash
- **Storage**: Internal `(signature)` file within the archive
- **Size**: 72 bytes (8-byte header + 64-byte signature)
- **Support**: Full verification and generation support

### Strong Signatures (MPQ v2+)

- **Algorithm**: 2048-bit RSA with SHA-1 hash
- **Storage**: Appended after archive data with "NGIS" header
- **Size**: 260 bytes (4-byte header + 256-byte signature)
- **Support**: Verification only (generation requires private key)

## Signature Verification

### Checking Archive Signatures

```rust
use wow_mpq::Archive;

// Open an archive and check its signature
let archive = Archive::open("signed.mpq")?;

match archive.verify_signature()? {
    SignatureStatus::None => println!("No signature present"),
    SignatureStatus::Weak => println!("Valid weak signature (512-bit RSA)"),
    SignatureStatus::Strong => println!("Valid strong signature (2048-bit RSA)"),
    SignatureStatus::Invalid => println!("WARNING: Invalid signature detected!"),
}
```

### Manual Signature Verification

For more control over the verification process:

```rust
use wow_mpq::crypto::{verify_weak_signature_stormlib, parse_weak_signature, SignatureInfo};
use std::io::Cursor;

// Read the (signature) file from the archive
let signature_data = archive.read_file("(signature)")?;
let signature = parse_weak_signature(&signature_data)?;

// Create signature info
let sig_info = SignatureInfo::new_weak(
    0,                          // Archive start offset
    archive_size,               // Archive size (excluding signature)
    signature_file_pos,         // Position of (signature) file
    signature_data.len() as u64,// Size of (signature) file
    signature_data.clone(),
);

// Verify the signature
let archive_data = std::fs::read("signed.mpq")?;
let valid = verify_weak_signature_stormlib(
    Cursor::new(&archive_data),
    &signature,
    &sig_info
)?;

if valid {
    println!("Signature is valid!");
} else {
    println!("Signature verification failed!");
}
```

## Signature Generation

### Generating Weak Signatures

Weak signatures can be generated using the well-known Blizzard private key:

```rust
use wow_mpq::crypto::{generate_weak_signature, SignatureInfo, WEAK_SIGNATURE_FILE_SIZE};
use wow_mpq::ArchiveBuilder;
use std::io::Cursor;

// Create an archive
let mut builder = ArchiveBuilder::new();
builder
    .add_file("readme.txt", b"Hello, World!")?
    .add_file("data.bin", b"Binary data here")?;

// Build the archive to memory
let archive_data = builder.build_to_vec()?;
let archive_size = archive_data.len() as u64;

// Create signature info
let sig_info = SignatureInfo::new_weak(
    0,                               // Archive start
    archive_size,                    // Archive size
    archive_size,                    // Signature position (at end)
    WEAK_SIGNATURE_FILE_SIZE as u64, // Signature file size (72 bytes)
    vec![],
);

// Generate the signature
let signature_file = generate_weak_signature(
    Cursor::new(&archive_data),
    &sig_info
)?;

// Now you can append the signature to the archive
// or add it as a "(signature)" file when rebuilding
```

### Adding Signatures During Archive Creation

The recommended approach is to add signatures during archive creation:

```rust
use wow_mpq::{ArchiveBuilder, RebuildOptions, rebuild_archive};

// Method 1: Using rebuild with signature generation
let options = RebuildOptions::new()
    .add_weak_signature(true);  // Enable weak signature generation

rebuild_archive("unsigned.mpq", "signed.mpq", &options)?;

// Method 2: Manual signature addition
let mut builder = ArchiveBuilder::new();

// Add your files
builder.add_file("content.txt", b"Important data")?;

// The signature will be generated and added during build
// when the appropriate flag is set
```

## Technical Details

### Hash Calculation

Signatures are calculated over the entire archive data, excluding the signature
area itself:

1. **Weak Signatures**: Use MD5 hash over 64KB chunks
2. **Strong Signatures**: Use SHA-1 hash over 64KB chunks
3. **Exclusion Area**: The signature file area is zeroed during hash calculation

### StormLib Compatibility

The implementation maintains full compatibility with StormLib's signature format:

```rust
// Hash calculation uses 64KB chunks (DIGEST_UNIT_SIZE)
const DIGEST_UNIT_SIZE: usize = 0x10000;

// Signature areas are zeroed, not skipped
// Multi-byte values use little-endian byte order
// RSA signatures are stored in little-endian format
```

### Signature File Format

#### Weak Signature File (72 bytes)

```text
Offset  Size  Description
------  ----  -----------
0x00    8     Unknown header (usually zeros)
0x08    64    RSA signature (little-endian)
```

#### Strong Signature Block (260 bytes)

```text
Offset  Size  Description
------  ----  -----------
0x00    4     "NGIS" header (0x5349474E reversed)
0x04    256   RSA signature (little-endian)
```

## Common Use Cases

### Verifying Downloaded Archives

```rust
use wow_mpq::Archive;

fn verify_download(path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let archive = Archive::open(path)?;

    match archive.verify_signature()? {
        SignatureStatus::None => {
            println!("Warning: Archive has no signature");
            Ok(false)
        }
        SignatureStatus::Invalid => {
            println!("ERROR: Archive signature is invalid!");
            Ok(false)
        }
        _ => {
            println!("Archive signature verified successfully");
            Ok(true)
        }
    }
}
```

### Creating Signed Patches

```rust
use wow_mpq::{ArchiveBuilder, RebuildOptions};

fn create_signed_patch(files: Vec<(&str, Vec<u8>)>) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ArchiveBuilder::new();

    // Add patch files
    for (name, data) in files {
        builder.add_file(name, data)?;
    }

    // Build with signature
    let options = RebuildOptions::new()
        .add_weak_signature(true);

    builder.build_with_options("patch.mpq", &options)?;
    Ok(())
}
```

### Batch Signature Verification

```rust
use wow_mpq::Archive;
use std::path::Path;

fn verify_all_archives(directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension() == Some("mpq".as_ref()) {
            print!("Checking {:?}... ", path.file_name().unwrap());

            match Archive::open(&path) {
                Ok(archive) => {
                    match archive.verify_signature()? {
                        SignatureStatus::None => println!("no signature"),
                        SignatureStatus::Weak => println!("weak signature OK"),
                        SignatureStatus::Strong => println!("strong signature OK"),
                        SignatureStatus::Invalid => println!("INVALID SIGNATURE!"),
                    }
                }
                Err(e) => println!("failed to open: {}", e),
            }
        }
    }
    Ok(())
}
```

## Best Practices

1. **Always Verify Signatures**: Check signatures on downloaded or received archives
2. **Sign Distribution Archives**: Add signatures to archives you distribute
3. **Use Appropriate Signature Type**:
   - Weak signatures for compatibility with older tools
   - Strong signatures for maximum security (when possible)
4. **Handle Missing Signatures Gracefully**: Not all archives have signatures
5. **Log Verification Results**: Keep audit trails of signature checks

## Limitations

1. **Strong Signature Generation**: Requires Blizzard's private key (not publicly available)
2. **Signature Modification**: Cannot modify signatures on existing archives without rebuilding
3. **Performance**: Signature verification requires reading the entire archive

## CLI Usage

The warcraft-rs CLI supports signature operations:

```bash
# Validate archive signature
warcraft-rs mpq validate archive.mpq --check-checksums

# Rebuild with signature
warcraft-rs mpq rebuild input.mpq output.mpq --add-signature

# Show signature information
warcraft-rs mpq info archive.mpq --show-signature
```

## References

- [MPQ Format Documentation](http://www.zezula.net/en/mpq/mpqformat.html)
- [StormLib Source Code](https://github.com/ladislav-zezula/StormLib)
- [RSA Cryptography](https://en.wikipedia.org/wiki/RSA_(cryptosystem))
- [PKCS#1 Standard](https://www.rfc-editor.org/rfc/rfc8017)
