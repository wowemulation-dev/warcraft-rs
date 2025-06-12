# wow-blp Implementation Status

This document tracks the implementation status of BLP (Blizzard Picture) texture format support.

## Overall Status: ✅ Implemented

### Core Functionality

| Feature | Status | Notes |
|---------|--------|-------|
| BLP0 Support | ✅ Implemented | Warcraft III Beta format |
| BLP1 Support | ✅ Implemented | Warcraft III format |
| BLP2 Header Parsing | ✅ Implemented | Standard BLP2 header |
| Mipmap Support | ✅ Implemented | Internal and external mipmaps |
| DXT Compression | ✅ Implemented | DXT1/3/5 formats using texpresso |
| Uncompressed Formats | ✅ Implemented | RAW3 format |
| Palette Support | ✅ Implemented | RAW1 256-color indexed |
| JPEG Support | ✅ Implemented | JPEG compression with alpha |

### Compression Format Support

| Format | Status | Notes |
|--------|--------|-------|
| DXT1 | ✅ Implemented | 4 bpp, 1-bit alpha |
| DXT3 | ✅ Implemented | 8 bpp, explicit alpha |
| DXT5 | ✅ Implemented | 8 bpp, interpolated alpha |
| RAW3 | ✅ Implemented | Uncompressed BGRA |
| RAW1 | ✅ Implemented | 8-bit indexed palette |
| JPEG | ✅ Implemented | JPEG with BGRA color space |

### Version Support

| Version | BLP Version | Status | Notes |
|---------|-------------|--------|-------|
| Warcraft III Beta | BLP0 | ✅ Implemented | External mipmaps |
| Warcraft III | BLP1 | ✅ Implemented | Internal mipmaps |
| WoW 1.12.1 - 5.4.8 | BLP2 | ✅ Implemented | All compression types |

### Features Implemented

- [x] BLP0/1/2 format parsing
- [x] All compression formats
- [x] Mipmap generation
- [x] Format conversion
- [x] Export to standard formats (via image crate)
- [x] Import from standard formats
- [x] Alpha channel handling (0, 1, 4, 8 bits)
- [x] Palette optimization

### Testing Status

| Test Category | Status |
|---------------|--------|
| Unit Tests | ✅ Implemented |
| Integration Tests | ✅ Implemented |
| Round-trip Tests | ✅ Implemented |

### Mipmap Support

| Feature | Status | Notes |
|---------|--------|-------|
| Read Mipmaps | ✅ Implemented | Up to 16 levels |
| Generate Mipmaps | ✅ Implemented | Automatic generation |
| External Mipmaps | ✅ Implemented | BLP0 .b00-.b15 files |
| Internal Mipmaps | ✅ Implemented | BLP1/2 embedded |

### Documentation Status

- [x] README.md - Complete documentation
- [x] STATUS.md - This file
- [x] API Documentation - Inline docs
- [x] Usage Examples - In examples/ directory

### Technical Details

- **Parser**: Native Rust implementation for binary parsing (no external parser dependencies)
- **Error Handling**: Comprehensive error types with context information
- **Performance**: Zero-copy parsing where possible

### Dependencies

- `image` - Image processing and conversion
- `color_quant` - Palette generation for RAW1
- `texpresso` - DXT compression/decompression

### Known Limitations

1. JPEG header size limited to 624 bytes for compatibility
2. Maximum texture size: 65535x65535 pixels
3. Test assets not included (proprietary Blizzard files)

### References

- [WoWDev.wiki BLP Format](https://wowdev.wiki/BLP)
- Original image-blp crate by zloy_tulen
