# MPQ Archive Modification API Design

## Overview

This document outlines the design for adding archive modification capabilities to wow-mpq. The API should be:
- Safe and idiomatic Rust
- Efficient for both small and large operations
- Compatible with existing read-only operations
- Similar to StormLib where appropriate, but leveraging Rust's strengths

## Design Approach

### Option 1: Separate Mutable Archive Type (Recommended)

Create a new `MutableArchive` type that wraps an `Archive` and adds modification capabilities:

```rust
use std::fs::OpenOptions;
use std::path::Path;

pub struct MutableArchive {
    /// The underlying read-only archive
    archive: Archive,
    /// File handle opened for read/write
    file: File,
    /// Track pending modifications
    pending_changes: Vec<PendingChange>,
    /// Whether changes need to be flushed
    dirty: bool,
}

impl MutableArchive {
    /// Open an archive for modification
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self>;
    
    /// Add a file from disk
    pub fn add_file<P: AsRef<Path>>(
        &mut self,
        source_path: P,
        archive_name: &str,
        options: AddFileOptions,
    ) -> Result<()>;
    
    /// Add a file from memory
    pub fn add_file_data(
        &mut self,
        data: &[u8],
        archive_name: &str,
        options: AddFileOptions,
    ) -> Result<()>;
    
    /// Remove a file
    pub fn remove_file(&mut self, archive_name: &str) -> Result<()>;
    
    /// Rename a file
    pub fn rename_file(&mut self, old_name: &str, new_name: &str) -> Result<()>;
    
    /// Compact the archive (remove deleted file space)
    pub fn compact(&mut self) -> Result<()>;
    
    /// Flush pending changes to disk
    pub fn flush(&mut self) -> Result<()>;
    
    /// Get immutable access to the underlying archive
    pub fn archive(&self) -> &Archive;
}
```

### Option 2: Add Modification Methods to Archive

Extend the existing `Archive` type with modification methods that require mutable access:

```rust
impl Archive {
    /// Reopen the archive for writing
    pub fn make_writable(&mut self) -> Result<()>;
    
    /// Add/remove/rename methods as above...
}
```

**Recommendation**: Option 1 is preferred because:
- Clear separation between read-only and mutable operations
- Prevents accidental modifications
- Can optimize for different access patterns
- Follows Rust patterns (like `Vec` vs `&[T]`)

## API Details

### AddFileOptions

```rust
#[derive(Debug, Clone, Default)]
pub struct AddFileOptions {
    /// Compression method (default: Zlib)
    pub compression: CompressionMethod,
    /// Whether to encrypt the file
    pub encrypt: bool,
    /// Whether to use FIX_KEY encryption
    pub fix_key: bool,
    /// Whether to replace existing file (default: true)
    pub replace_existing: bool,
    /// Locale code (default: 0 = neutral)
    pub locale: u16,
    /// Platform code (default: 0 = all)
    pub platform: u8,
}

impl AddFileOptions {
    pub fn new() -> Self { Default::default() }
    
    pub fn compression(mut self, method: CompressionMethod) -> Self {
        self.compression = method;
        self
    }
    
    pub fn encrypt(mut self) -> Self {
        self.encrypt = true;
        self
    }
    
    // ... other builder methods
}
```

### Progress Callbacks

For long operations like compacting:

```rust
pub trait ProgressCallback: FnMut(Progress) {
    fn on_progress(&mut self, progress: Progress);
}

pub struct Progress {
    pub operation: Operation,
    pub current: u64,
    pub total: u64,
    pub message: Option<String>,
}

pub enum Operation {
    AddingFile,
    RemovingFile,
    Compacting,
    WritingTables,
}
```

## Implementation Strategy

### Phase 1: Basic Modification Support

1. **File Addition Algorithm**:
   - Find free space or append to end
   - Update hash table entry
   - Update block table entry
   - Write file data with compression/encryption
   - Update (listfile) if present

2. **File Removal Algorithm**:
   - Mark hash table entry as deleted (FFFFFFFFh)
   - Keep block table entry (for compacting)
   - Update (listfile) if present

3. **File Renaming Algorithm**:
   - Remove old hash table entry
   - Add new hash table entry
   - Update (listfile) if present

### Phase 2: Advanced Features

1. **Archive Compacting**:
   - Create new temporary file
   - Copy all non-deleted files
   - Rebuild tables
   - Replace original file

2. **In-Place Modifications**:
   - Optimize small changes without full rewrite
   - Reuse deleted file space
   - Extend archive when needed

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ModificationError {
    #[error("Archive is read-only")]
    ReadOnly,
    
    #[error("File already exists: {0}")]
    FileExists(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("No space available in hash table")]
    HashTableFull,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Usage Examples

```rust
use wow_mpq::{MutableArchive, AddFileOptions, CompressionMethod};

// Open archive for modification
let mut archive = MutableArchive::open("data.mpq")?;

// Add a file with default options
archive.add_file("texture.blp", "Textures\\new_texture.blp", Default::default())?;

// Add with custom options
let options = AddFileOptions::new()
    .compression(CompressionMethod::Lzma)
    .encrypt();
archive.add_file("model.m2", "Models\\character.m2", options)?;

// Remove a file
archive.remove_file("old_file.txt")?;

// Rename a file
archive.rename_file("Textures\\old.blp", "Textures\\new.blp")?;

// Compact to reclaim space
archive.compact()?;

// Changes are automatically flushed on drop, or manually:
archive.flush()?;
```

## Testing Strategy

1. **Unit Tests**:
   - Test each operation in isolation
   - Test error conditions
   - Test table updates

2. **Integration Tests**:
   - Create archive, modify, verify with StormLib
   - Test modification of StormLib-created archives
   - Round-trip testing

3. **Comparison Tests**:
   - Compare behavior with StormLib
   - Ensure file format compatibility
   - Test edge cases

## Performance Considerations

1. **Lazy Updates**:
   - Batch table updates
   - Defer writes until flush/drop

2. **Space Management**:
   - Track free blocks for reuse
   - Minimize file moves

3. **Memory Usage**:
   - Stream large files
   - Limit in-memory buffers

## Next Steps

1. Implement `MutableArchive` struct
2. Add basic file addition support
3. Implement removal and renaming
4. Add compacting functionality
5. Create comprehensive tests
6. Optimize performance