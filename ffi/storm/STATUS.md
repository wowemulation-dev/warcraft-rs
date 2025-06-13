# storm Implementation Status

This document tracks the implementation status of the StormLib-compatible C API.

## Overall Status: ✅ Implemented

### Core Functionality

| Feature | Status | Notes |
|---------|--------|-------|
| Archive Operations | ✅ Implemented | Open, create, close |
| File Operations | ✅ Implemented | Read, extract, seek |
| Information Queries | ✅ Implemented | File info, archive info |
| Error Handling | ✅ Implemented | Thread-local error storage |
| Handle Management | ✅ Implemented | Thread-safe handle storage |
| Header Generation | ✅ Implemented | Automatic via cbindgen |

### Archive Operations

| Function | Status | Notes |
|----------|--------|-------|
| `SFileOpenArchive` | ✅ Implemented | Full compatibility |
| `SFileCreateArchive` | ✅ Implemented | All format versions |
| `SFileCloseArchive` | ✅ Implemented | Handle cleanup |
| `SFileFlushArchive` | ❌ Not Implemented | Not needed |
| `SFileCompactArchive` | ❌ Not Implemented | Use rebuild instead |
| `SFileSetMaxFileCount` | ❌ Not Implemented | Not applicable |

### File Operations

| Function | Status | Notes |
|----------|--------|-------|
| `SFileOpenFileEx` | ✅ Implemented | With locale support |
| `SFileCloseFile` | ✅ Implemented | Handle cleanup |
| `SFileReadFile` | ✅ Implemented | Full decompression |
| `SFileGetFileSize` | ✅ Implemented | Both compressed/uncompressed |
| `SFileSetFilePointer` | ✅ Implemented | All seek modes |
| `SFileGetFilePointer` | ✅ Implemented | Current position |
| `SFileHasFile` | ✅ Implemented | File existence check |
| `SFileExtractFile` | ✅ Implemented | Direct to disk |
| `SFileGetFileName` | ✅ Implemented | Current file name |
| `SFileAddFile` | ❌ Not Implemented | Use MutableArchive |
| `SFileRemoveFile` | ❌ Not Implemented | Use MutableArchive |
| `SFileRenameFile` | ❌ Not Implemented | Use MutableArchive |

### Information Functions

| Function | Status | Notes |
|----------|--------|-------|
| `SFileGetFileInfo` | ✅ Implemented | Archive and file info |
| `SFileGetArchiveName` | ✅ Implemented | Full path retrieval |
| `SFileEnumFiles` | ✅ Implemented | With callback support |
| `SFileGetAttributes` | ⚠️ Partial | Stub implementation |
| `SFileSetAttributes` | ❌ Not Implemented | Not needed |
| `SFileUpdateFileAttributes` | ❌ Not Implemented | Auto-managed |

### Utility Functions

| Function | Status | Notes |
|----------|--------|-------|
| `SFileSetLocale` | ✅ Implemented | Global locale |
| `SFileGetLocale` | ✅ Implemented | Current locale |
| `SFileGetLastError` | ✅ Implemented | Thread-local storage |
| `SFileSetLastError` | ✅ Implemented | Error setting |

### Verification Functions

| Function | Status | Notes |
|----------|--------|-------|
| `SFileVerifyFile` | ✅ Implemented | CRC/MD5 checks |
| `SFileVerifyArchive` | ✅ Implemented | Signature verification |
| `SFileSignArchive` | ⚠️ Partial | Stub only |
| `SFileGetFileChecksums` | ❌ Not Implemented | Use file info |

### Listfile Functions

| Function | Status | Notes |
|----------|--------|-------|
| `SFileAddListFile` | ❌ Not Implemented | Auto-managed |
| `SFileSetAddFileCallback` | ❌ Not Implemented | Not needed |
| `SFileOpenPatchArchive` | ❌ Not Implemented | Use PatchChain |

### Version Support

| WoW Version | Status | Notes |
|-------------|--------|-------|
| Classic (1.12.1) | ✅ Supported | Full compatibility |
| TBC (2.4.3) | ✅ Supported | Full compatibility |
| WotLK (3.3.5a) | ✅ Supported | Full compatibility |
| Cataclysm (4.3.4) | ✅ Supported | Full compatibility |
| MoP (5.4.8) | ✅ Supported | Full compatibility |

### Error Code Mapping

| wow-mpq Error | Windows Error Code | Status |
|---------------|-------------------|--------|
| `FileNotFound` | `ERROR_FILE_NOT_FOUND` | ✅ Mapped |
| `InvalidFormat` | `ERROR_FILE_CORRUPT` | ✅ Mapped |
| `Io` | `ERROR_ACCESS_DENIED` | ✅ Mapped |
| `ChecksumMismatch` | `ERROR_CRC` | ✅ Mapped |
| Other errors | `ERROR_INVALID_PARAMETER` | ✅ Default |

### Testing Status

| Test Category | Status |
|---------------|--------|
| Unit Tests | ⚠️ Minimal |
| Integration Tests | ❌ Not Implemented |
| C Example Programs | ✅ Implemented |
| StormLib Compatibility | ⚠️ Partial |

### Dependencies

- `wow-mpq` - Core MPQ functionality
- `libc` - C types and functions
- `log` - Logging support
- `md-5` - MD5 hashing
- `crc32fast` - CRC32 checksums
- `cbindgen` - Header generation

### Known Limitations

1. **No Archive Modification**: File add/remove/rename operations not implemented
   - Workaround: Use wow-mpq's `MutableArchive` directly from Rust
2. **No Streaming API**: Large file streaming not supported
3. **No Protected MPQ Support**: Copy-protected archives not supported
4. **Stub Functions**: Some functions return success without operation
5. **No Async Support**: All operations are synchronous

### Implementation Notes

1. **Handle Management**: Uses global mutex-protected HashMap for thread safety
2. **Error Storage**: Thread-local storage for Windows error codes
3. **Path Conversion**: Automatic forward slash to backslash conversion
4. **Memory Safety**: All pointers validated before use
5. **Locale Support**: Global locale setting affects file operations

### Future Improvements

- [ ] Add archive modification functions
- [ ] Implement missing information queries
- [ ] Add comprehensive test suite
- [ ] Performance optimizations
- [ ] Extended error mapping

### References

- [StormLib Documentation](http://www.zezula.net/en/mpq/stormlib.html)
- [wow-mpq Documentation](https://docs.rs/wow-mpq)
