# wow-blp Implementation Status

This document tracks the implementation status of BLP (Blizzard Picture) texture format support.

## Overall Status: 🚧 In Development

### Core Functionality

| Feature | Status | Notes |
|---------|--------|-------|
| BLP2 Header Parsing | ❌ Not Started | Standard BLP2 header |
| Mipmap Support | ❌ Not Started | Multiple resolution levels |
| DXT Compression | ❌ Not Started | DXT1/3/5 formats |
| Uncompressed Formats | ❌ Not Started | ARGB formats |
| Palette Support | ❌ Not Started | 256-color indexed |
| JPEG Support | ❌ Not Started | Legacy JPEG compression |

### Compression Format Support

| Format | Status | Notes |
|--------|--------|-------|
| DXT1 | ❌ Not Started | 4 bpp, 1-bit alpha |
| DXT3 | ❌ Not Started | 8 bpp, explicit alpha |
| DXT5 | ❌ Not Started | 8 bpp, interpolated alpha |
| Uncompressed ARGB8888 | ❌ Not Started | 32-bit ARGB |
| Uncompressed RGB888 | ❌ Not Started | 24-bit RGB |
| Uncompressed Palette | ❌ Not Started | 8-bit indexed |
| JPEG | ❌ Not Started | Legacy format |

### Version Support

| Version | BLP Version | Status | Notes |
|---------|-------------|--------|-------|
| Classic - WotLK | BLP2 | ❌ Not Started | Standard format |
| Cataclysm+ | BLP2 | ❌ Not Started | Extended features |

### Features Planned

- [ ] BLP2 format parsing
- [ ] All compression formats
- [ ] Mipmap generation
- [ ] Format conversion
- [ ] Export to standard formats (PNG, DDS)
- [ ] Import from standard formats
- [ ] Alpha channel handling
- [ ] Palette optimization

### Known Limitations

1. No implementation started yet
2. BLP1 format (pre-release) not planned
3. JPEG decompression requires external decoder

### Testing Status

| Test Category | Status |
|---------------|--------|
| Unit Tests | ❌ Not Started |
| Integration Tests | ❌ Not Started |
| Compression Tests | ❌ Not Started |
| Round-trip Tests | ❌ Not Started |

### Mipmap Support

| Feature | Status | Notes |
|---------|--------|-------|
| Read Mipmaps | ❌ Not Started | Up to 16 levels |
| Generate Mipmaps | ❌ Not Started | Automatic generation |
| Selective Loading | ❌ Not Started | Load specific levels |

### Documentation Status

- [x] README.md - Basic structure
- [x] STATUS.md - This file
- [ ] API Documentation
- [ ] Format Specification
- [ ] Usage Examples
- [ ] Compression Guide

### References

- [WoWDev.wiki BLP Format](https://wowdev.wiki/BLP)
- [DXT Compression](https://docs.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression)

### TODO

1. Implement BLP2 header parsing
2. Add DXT decompression support
3. Implement uncompressed format support
4. Add mipmap handling
5. Create conversion utilities
6. Add export/import functionality
