# Storm FFI (Foreign Function Interface)

## Overview

The `storm-ffi` crate provides a C-compatible FFI layer that implements the StormLib API, allowing C/C++ applications to use the `wow-mpq` Rust implementation. This enables drop-in replacement of StormLib with a Rust-based implementation.

**Key Features:**

- ✅ **StormLib API Compatibility** - Implements StormLib's C API
- ✅ **Thread-Safe** - Safe concurrent access from multiple threads
- ✅ **Archive Modification** - Full support for add/remove/rename operations
- ✅ **Cross-Platform** - Works on Windows, Linux, and macOS
- ✅ **Memory Safe** - Rust's safety guarantees with C compatibility

## API Reference

### Archive Management

#### Opening Archives

```c
// Open an archive for reading
HANDLE archive;
if (SFileOpenArchive("Data/patch.mpq", 0, MPQ_OPEN_READ_ONLY, &archive)) {
    // Archive opened successfully
    SFileCloseArchive(archive);
}

// Open an archive for modification
HANDLE mutable_archive;
if (SFileOpenArchive("my_archive.mpq", 0, MPQ_OPEN_READ_WRITE, &mutable_archive)) {
    // Archive opened for modification
    SFileCloseArchive(mutable_archive);
}
```

#### Creating Archives

```c
// Create a new MPQ archive (simple)
HANDLE new_archive;
if (SFileCreateArchive("new.mpq", MPQ_CREATE_ARCHIVE_V2, 0x1000, &new_archive)) {
    // Add files...
    SFileCloseArchive(new_archive);
}

// Create with extended options
SFILE_CREATE_MPQ create_info = {0};
create_info.cbSize = sizeof(SFILE_CREATE_MPQ);
create_info.mpq_version = 2;  // MPQ format v2
create_info.sector_size = 4;  // 4096 bytes
create_info.attr_flags = MPQ_ATTRIBUTE_CRC32 | MPQ_ATTRIBUTE_MD5;
create_info.max_file_count = 1000;

HANDLE advanced_archive;
if (SFileCreateArchive2("advanced.mpq", &create_info, &advanced_archive)) {
    // Archive created with custom settings
    SFileCloseArchive(advanced_archive);
}
```

### File Operations

#### Reading Files

```c
// Open and read a file
HANDLE file;
if (SFileOpenFileEx(archive, "Interface\\Icons\\INV_Misc_QuestionMark.blp", 0, &file)) {
    // Get file size
    DWORD file_size = SFileGetFileSize(file, NULL);
    
    // Read file data
    char* buffer = malloc(file_size);
    DWORD bytes_read;
    if (SFileReadFile(file, buffer, file_size, &bytes_read, NULL)) {
        // File read successfully
        printf("Read %u bytes\n", bytes_read);
    }
    
    free(buffer);
    SFileCloseFile(file);
}
```

#### Adding Files

```c
// Add a file from disk
if (SFileAddFile(archive, "local_file.txt", "archived_name.txt", MPQ_FILE_COMPRESS)) {
    printf("File added successfully\n");
}

// Add with extended options
DWORD flags = MPQ_FILE_ENCRYPTED | MPQ_FILE_REPLACEEXISTING;
DWORD compression = MPQ_COMPRESSION_ZLIB;
if (SFileAddFileEx(archive, "secret.dat", "Data\\Secret.dat", flags, compression, 0)) {
    printf("Encrypted file added\n");
}

// Add from memory
const char* data = "Hello, MPQ!";
if (SFileCreateFile(archive, "hello.txt", 0, strlen(data) + 1, 0, MPQ_FILE_COMPRESS, &file)) {
    DWORD written;
    SFileWriteFile(file, data, strlen(data) + 1, &written, MPQ_COMPRESSION_ZLIB);
    SFileFinishFile(file);
}
```

#### Modifying Archives

```c
// Remove a file
if (SFileRemoveFile(archive, "old_file.txt", 0)) {
    printf("File removed\n");
}

// Rename a file
if (SFileRenameFile(archive, "old_name.txt", "new_name.txt")) {
    printf("File renamed\n");
}

// Flush changes to disk
SFileFlushArchive(archive);

// Compact archive to reclaim space
if (SFileCompactArchive(archive, NULL, false)) {
    printf("Archive compacted\n");
}
```

### File Finding

```c
// Find files matching a pattern
SFILE_FIND_DATA find_data;
HANDLE find = SFileFindFirstFile(archive, "*.blp", &find_data);

if (find != INVALID_HANDLE_VALUE) {
    do {
        printf("Found: %s (%u bytes)\n", 
            find_data.c_file_name, 
            find_data.file_size);
    } while (SFileFindNextFile(find, &find_data));
    
    SFileFindClose(find);
}
```

### Archive Information

```c
// Get archive info
DWORD value;
DWORD size_needed;

// Get archive size
if (SFileGetFileInfo(archive, SFILE_INFO_ARCHIVE_SIZE, &value, sizeof(value), &size_needed)) {
    printf("Archive size: %u bytes\n", value);
}

// Get format version
if (SFileGetFileInfo(archive, SFILE_INFO_FORMAT_VERSION, &value, sizeof(value), &size_needed)) {
    printf("Format version: %u\n", value);
}

// Get archive name
char name_buffer[260];
if (SFileGetArchiveName(archive, name_buffer, sizeof(name_buffer))) {
    printf("Archive path: %s\n", name_buffer);
}
```

### Verification

```c
// Verify a single file
DWORD verify_flags = SFILE_VERIFY_SECTOR_CRC | SFILE_VERIFY_FILE_CRC;
if (SFileVerifyFile(archive, "important.dat", verify_flags)) {
    printf("File verification passed\n");
} else {
    printf("File verification failed: %u\n", SFileGetLastError());
}

// Verify entire archive
DWORD archive_flags = SFILE_VERIFY_SIGNATURE | SFILE_VERIFY_ALL_FILES;
if (SFileVerifyArchive(archive, archive_flags)) {
    printf("Archive verification passed\n");
}
```

### Attributes

```c
// Get file attributes
DWORD crc32, md5[4];
char file_time[8];

if (SFileGetFileAttributes(archive, 10, &crc32, file_time, md5)) {
    printf("CRC32: 0x%08X\n", crc32);
}

// Update file attributes
DWORD new_crc = 0x12345678;
SFileUpdateFileAttributes(archive, "updated_file.txt");

// Work with archive attributes
if (SFileFlushAttributes(archive)) {
    printf("Attributes flushed\n");
}
```

## Error Handling

The FFI layer uses Windows-compatible error codes:

```c
DWORD error = SFileGetLastError();
switch (error) {
    case ERROR_SUCCESS:
        printf("Operation successful\n");
        break;
    case ERROR_FILE_NOT_FOUND:
        printf("File not found\n");
        break;
    case ERROR_ACCESS_DENIED:
        printf("Access denied\n");
        break;
    case ERROR_NOT_SUPPORTED:
        printf("Operation not supported\n");
        break;
    case ERROR_INVALID_PARAMETER:
        printf("Invalid parameter\n");
        break;
    case ERROR_ALREADY_EXISTS:
        printf("File already exists\n");
        break;
    case ERROR_DISK_FULL:
        printf("Disk full\n");
        break;
    case ERROR_FILE_CORRUPT:
        printf("File corrupt\n");
        break;
    default:
        printf("Unknown error: %u\n", error);
}
```

## Building and Linking

### Rust Side

Add to your `Cargo.toml`:

```toml
[dependencies]
storm-ffi = { path = "../ffi/storm-ffi" }
```

### C/C++ Side

Include the header and link the library:

```c
#include "StormLib.h"

// Link with libstorm.so (Linux), storm.dll (Windows), or libstorm.dylib (macOS)
```

CMake example:

```cmake
find_library(STORM_LIB storm PATHS ${CMAKE_SOURCE_DIR}/lib)
target_link_libraries(your_app ${STORM_LIB})
```

## Thread Safety

All functions are thread-safe with the following guarantees:

- Multiple threads can read from the same archive concurrently
- Archive modification operations are serialized internally
- Each thread maintains its own error state
- File handles are not thread-safe (use one handle per thread)

## Differences from StormLib

While the API is compatible, there are some implementation differences:

1. **Memory Management**: The Rust implementation manages memory internally - no manual cleanup required except for handles
2. **Error Handling**: Thread-local error storage instead of global error state
3. **Performance**: Generally faster due to Rust optimizations
4. **Safety**: Memory-safe implementation prevents buffer overflows and use-after-free

## Example: Complete Program

```c
#include <stdio.h>
#include <stdlib.h>
#include "StormLib.h"

int main() {
    HANDLE archive;
    
    // Open archive
    if (!SFileOpenArchive("Data/patch.mpq", 0, MPQ_OPEN_READ_WRITE, &archive)) {
        printf("Failed to open archive: %u\n", SFileGetLastError());
        return 1;
    }
    
    // Add a new file
    if (SFileAddFile(archive, "readme.txt", "README.txt", MPQ_FILE_COMPRESS)) {
        printf("File added successfully\n");
    }
    
    // Find all DBC files
    SFILE_FIND_DATA find_data;
    HANDLE find = SFileFindFirstFile(archive, "*.dbc", &find_data);
    
    if (find != INVALID_HANDLE_VALUE) {
        do {
            printf("DBC: %s\n", find_data.c_file_name);
        } while (SFileFindNextFile(find, &find_data));
        
        SFileFindClose(find);
    }
    
    // Extract a file
    HANDLE file;
    if (SFileOpenFileEx(archive, "Interface\\FrameXML\\Fonts.xml", 0, &file)) {
        DWORD size = SFileGetFileSize(file, NULL);
        char* buffer = malloc(size);
        DWORD read;
        
        if (SFileReadFile(file, buffer, size, &read, NULL)) {
            FILE* out = fopen("Fonts.xml", "wb");
            fwrite(buffer, 1, read, out);
            fclose(out);
            printf("Extracted Fonts.xml\n");
        }
        
        free(buffer);
        SFileCloseFile(file);
    }
    
    // Close archive
    SFileCloseArchive(archive);
    
    return 0;
}
```

## Testing

The storm-ffi crate includes the following tests:

```bash
# Run FFI tests
cargo test -p storm-ffi

# Run with StormLib comparison tests
cargo test -p storm-ffi --features stormlib-compare
```

## See Also

- [MPQ Archives Guide](../guides/mpq-archives.md) - Working with MPQ archives in Rust
- [StormLib Differences](../guides/stormlib-differences.md) - Detailed comparison with StormLib
- [MPQ Format](../formats/archives/mpq.md) - MPQ format specification