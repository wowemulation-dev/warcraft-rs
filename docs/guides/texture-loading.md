# 🖼️ Texture Loading Guide

## Overview

BLP (Blizzard Picture) is the proprietary texture format used throughout World
of Warcraft for all textures including UI elements, models, terrain, and
effects. This guide covers loading, decoding, and using BLP textures with
`warcraft-rs`, including mipmap handling, format conversion, and GPU upload.

## Prerequisites

Before working with textures, ensure you have:

- Understanding of texture formats and graphics APIs
- Basic knowledge of image processing
- `warcraft-rs` installed with the `blp` feature enabled
- A graphics framework (wgpu, OpenGL, Vulkan, DirectX)
- Familiarity with texture compression formats

## Understanding BLP Format

### BLP Versions

- **BLP0**: Legacy format (Warcraft III)
- **BLP1**: Classic WoW through WotLK
- **BLP2**: Cataclysm and later

### Compression Types

- **Uncompressed**: Raw BGRA data
- **Palettized**: 256-color palette compression
- **DXT**: DirectX texture compression (DXT1, DXT3, DXT5)
- **JPEG**: JPEG compressed (BLP1 only)

### Key Features

- **Mipmaps**: Pre-generated levels for efficient rendering
- **Alpha Channel**: Transparency support
- **Power-of-two**: Dimensions are always powers of 2
- **Maximum Size**: Typically 2048x2048 or 4096x4096

## Step-by-Step Instructions

### 1. Loading BLP Files

```rust
use warcraft_rs::blp::{Blp, BlpImage, BlpFormat};
use std::path::Path;

fn load_blp_texture(file_path: &str) -> Result<Blp, Box<dyn std::error::Error>> {
    // Load from file
    let blp = Blp::from_file(file_path)?;

    println!("BLP Version: {}", blp.version());
    println!("Size: {}x{}", blp.width(), blp.height());
    println!("Format: {:?}", blp.format());
    println!("Mipmap Levels: {}", blp.mipmap_count());
    println!("Has Alpha: {}", blp.has_alpha());

    Ok(blp)
}

// Load from MPQ archive
fn load_blp_from_mpq(archive: &mut Archive, texture_path: &str) -> Result<Blp, Box<dyn std::error::Error>> {
    let data = archive.extract(texture_path)?;
    let blp = Blp::from_bytes(&data)?;
    Ok(blp)
}

// Batch loading textures
fn load_multiple_textures(paths: &[&str]) -> Vec<Result<Blp, Box<dyn std::error::Error>>> {
    paths.iter()
        .map(|path| load_blp_texture(path))
        .collect()
}
```

### 2. Converting BLP to Raw RGBA

```rust
use warcraft_rs::blp::{Blp, BlpFormat, MipmapLevel};

fn convert_blp_to_rgba(blp: &Blp, mipmap_level: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Get specific mipmap level
    let mipmap = blp.get_mipmap(mipmap_level)
        .ok_or("Invalid mipmap level")?;

    // Convert based on format
    let rgba_data = match blp.format() {
        BlpFormat::Jpeg => decode_jpeg_blp(blp, mipmap)?,
        BlpFormat::Palettized => decode_palettized_blp(blp, mipmap)?,
        BlpFormat::Dxt1 => decode_dxt1(mipmap.data(), mipmap.width(), mipmap.height())?,
        BlpFormat::Dxt3 => decode_dxt3(mipmap.data(), mipmap.width(), mipmap.height())?,
        BlpFormat::Dxt5 => decode_dxt5(mipmap.data(), mipmap.width(), mipmap.height())?,
        BlpFormat::Uncompressed => convert_bgra_to_rgba(mipmap.data()),
    };

    Ok(rgba_data)
}

fn decode_palettized_blp(blp: &Blp, mipmap: &MipmapLevel) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let palette = blp.palette()
        .ok_or("No palette found for palettized BLP")?;

    let mut rgba = Vec::with_capacity(mipmap.width() * mipmap.height() * 4);

    for &index in mipmap.data() {
        let color = &palette[index as usize];
        rgba.push(color.r);
        rgba.push(color.g);
        rgba.push(color.b);
        rgba.push(color.a);
    }

    Ok(rgba)
}

fn convert_bgra_to_rgba(bgra_data: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(bgra_data.len());

    for chunk in bgra_data.chunks_exact(4) {
        rgba.push(chunk[2]); // R
        rgba.push(chunk[1]); // G
        rgba.push(chunk[0]); // B
        rgba.push(chunk[3]); // A
    }

    rgba
}
```

### 3. DXT Decompression

```rust
use squish::{Format, decompress_image};

fn decode_dxt1(data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    decompress_image(
        &mut rgba,
        width as i32,
        height as i32,
        data,
        Format::Bc1, // DXT1
    );

    Ok(rgba)
}

fn decode_dxt3(data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    decompress_image(
        &mut rgba,
        width as i32,
        height as i32,
        data,
        Format::Bc2, // DXT3
    );

    Ok(rgba)
}

fn decode_dxt5(data: &[u8], width: u32, height: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    decompress_image(
        &mut rgba,
        width as i32,
        height as i32,
        data,
        Format::Bc3, // DXT5
    );

    Ok(rgba)
}

// Alternative: Manual DXT decompression for learning purposes
fn decode_dxt1_block(block: &[u8; 8], output: &mut [u8], pitch: usize) {
    // Extract color endpoints
    let c0 = u16::from_le_bytes([block[0], block[1]]);
    let c1 = u16::from_le_bytes([block[2], block[3]]);

    // Decode colors
    let color0 = decode_565_color(c0);
    let color1 = decode_565_color(c1);

    // Generate color palette
    let mut colors = [color0, color1, [0, 0, 0, 255], [0, 0, 0, 255]];

    if c0 > c1 {
        // 4-color block
        colors[2] = interpolate_color(&color0, &color1, 1, 3);
        colors[3] = interpolate_color(&color0, &color1, 2, 3);
    } else {
        // 3-color block with transparency
        colors[2] = interpolate_color(&color0, &color1, 1, 2);
        colors[3] = [0, 0, 0, 0]; // Transparent
    }

    // Decode pixel indices
    let indices = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);

    for y in 0..4 {
        for x in 0..4 {
            let index = ((indices >> ((y * 4 + x) * 2)) & 0x3) as usize;
            let offset = (y * pitch + x * 4);
            output[offset..offset + 4].copy_from_slice(&colors[index]);
        }
    }
}

fn decode_565_color(color: u16) -> [u8; 4] {
    let r = ((color >> 11) & 0x1F) as u8;
    let g = ((color >> 5) & 0x3F) as u8;
    let b = (color & 0x1F) as u8;

    [
        (r << 3) | (r >> 2), // Expand 5-bit to 8-bit
        (g << 2) | (g >> 4), // Expand 6-bit to 8-bit
        (b << 3) | (b >> 2), // Expand 5-bit to 8-bit
        255,
    ]
}
```

### 4. GPU Texture Upload

```rust
use wgpu::*;

struct GpuTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
    bind_group: BindGroup,
}

fn upload_blp_to_gpu(
    device: &Device,
    queue: &Queue,
    blp: &Blp,
    label: Option<&str>,
) -> Result<GpuTexture, Box<dyn std::error::Error>> {
    // Determine GPU format based on BLP format
    let format = match blp.format() {
        BlpFormat::Dxt1 if !blp.has_alpha() => TextureFormat::Bc1RgbaUnorm,
        BlpFormat::Dxt1 => TextureFormat::Bc1RgbaUnorm,
        BlpFormat::Dxt3 => TextureFormat::Bc2RgbaUnorm,
        BlpFormat::Dxt5 => TextureFormat::Bc3RgbaUnorm,
        _ => TextureFormat::Rgba8Unorm, // Decompress on CPU
    };

    // Create texture
    let texture = device.create_texture(&TextureDescriptor {
        label,
        size: Extent3d {
            width: blp.width(),
            height: blp.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: blp.mipmap_count() as u32,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Upload mipmap levels
    for level in 0..blp.mipmap_count() {
        let mipmap = blp.get_mipmap(level).unwrap();

        let data = if format.is_compressed() {
            // Use compressed data directly
            mipmap.data().to_vec()
        } else {
            // Convert to RGBA
            convert_blp_to_rgba(blp, level)?
        };

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: level as u32,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(calculate_bytes_per_row(format, mipmap.width())),
                rows_per_image: Some(mipmap.height()),
            },
            Extent3d {
                width: mipmap.width(),
                height: mipmap.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    // Create view and sampler
    let view = texture.create_view(&TextureViewDescriptor::default());

    let sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("BLP Sampler"),
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        ..Default::default()
    });

    // Create bind group
    let bind_group_layout = create_texture_bind_group_layout(device);
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("BLP Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&sampler),
            },
        ],
    });

    Ok(GpuTexture {
        texture,
        view,
        sampler,
        bind_group,
    })
}

fn calculate_bytes_per_row(format: TextureFormat, width: u32) -> u32 {
    match format {
        TextureFormat::Bc1RgbaUnorm => ((width + 3) / 4) * 8,
        TextureFormat::Bc2RgbaUnorm | TextureFormat::Bc3RgbaUnorm => ((width + 3) / 4) * 16,
        TextureFormat::Rgba8Unorm => width * 4,
        _ => panic!("Unsupported texture format"),
    }
}
```

### 5. Texture Caching System

```rust
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

pub struct TextureCache {
    cache: Arc<RwLock<HashMap<String, Arc<GpuTexture>>>>,
    lru_queue: Arc<RwLock<VecDeque<String>>>,
    max_size: usize,
    current_size: Arc<RwLock<usize>>,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl TextureCache {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, max_size_mb: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            lru_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_size: max_size_mb * 1024 * 1024,
            current_size: Arc::new(RwLock::new(0)),
            device,
            queue,
        }
    }

    pub async fn get_texture(&self, path: &str) -> Result<Arc<GpuTexture>, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(texture) = cache.get(path) {
                self.update_lru(path);
                return Ok(texture.clone());
            }
        }

        // Load texture
        let texture = self.load_texture(path).await?;
        let texture_arc = Arc::new(texture);

        // Add to cache
        self.add_to_cache(path.to_string(), texture_arc.clone())?;

        Ok(texture_arc)
    }

    async fn load_texture(&self, path: &str) -> Result<GpuTexture, Box<dyn std::error::Error>> {
        // Load BLP file asynchronously
        let blp_data = tokio::fs::read(path).await?;
        let blp = Blp::from_bytes(&blp_data)?;

        // Upload to GPU on main thread
        let device = self.device.clone();
        let queue = self.queue.clone();

        tokio::task::spawn_blocking(move || {
            upload_blp_to_gpu(&device, &queue, &blp, Some(path))
        })
        .await?
    }

    fn add_to_cache(&self, path: String, texture: Arc<GpuTexture>) -> Result<(), Box<dyn std::error::Error>> {
        let size = self.estimate_texture_size(&texture);

        // Evict old textures if needed
        while *self.current_size.read().unwrap() + size > self.max_size {
            self.evict_oldest()?;
        }

        // Add new texture
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(path.clone(), texture);
        }

        {
            let mut queue = self.lru_queue.write().unwrap();
            queue.push_back(path);
        }

        {
            let mut current = self.current_size.write().unwrap();
            *current += size;
        }

        Ok(())
    }

    fn evict_oldest(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = {
            let mut queue = self.lru_queue.write().unwrap();
            queue.pop_front()
        };

        if let Some(path) = path {
            let size = {
                let mut cache = self.cache.write().unwrap();
                if let Some(texture) = cache.remove(&path) {
                    self.estimate_texture_size(&texture)
                } else {
                    0
                }
            };

            let mut current = self.current_size.write().unwrap();
            *current = current.saturating_sub(size);
        }

        Ok(())
    }

    fn update_lru(&self, path: &str) {
        let mut queue = self.lru_queue.write().unwrap();
        queue.retain(|p| p != path);
        queue.push_back(path.to_string());
    }

    fn estimate_texture_size(&self, texture: &GpuTexture) -> usize {
        // Estimate based on texture dimensions and format
        // This is a rough estimate
        let desc = &texture.texture.size();
        let bytes_per_pixel = 4; // Assume RGBA8
        desc.width as usize * desc.height as usize * bytes_per_pixel
    }
}
```

### 6. Texture Atlas Generation

```rust
use rectangle_pack::{RectanglePacker, PackedLocation};

pub struct TextureAtlas {
    texture: Texture,
    packer: RectanglePacker,
    regions: HashMap<String, AtlasRegion>,
}

#[derive(Clone, Debug)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

impl TextureAtlas {
    pub fn new(device: &Device, size: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture Atlas"),
            size: Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let packer = RectanglePacker::new(size, size);

        Self {
            texture,
            packer,
            regions: HashMap::new(),
        }
    }

    pub fn add_texture(
        &mut self,
        queue: &Queue,
        name: &str,
        blp: &Blp,
    ) -> Result<AtlasRegion, Box<dyn std::error::Error>> {
        // Convert BLP to RGBA
        let rgba_data = convert_blp_to_rgba(blp, 0)?;
        let width = blp.width();
        let height = blp.height();

        // Pack into atlas
        let packed = self.packer
            .pack(width as i32, height as i32, false)
            .ok_or("Failed to pack texture into atlas")?;

        // Upload to atlas
        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: packed.x as u32,
                    y: packed.y as u32,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            &rgba_data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Calculate UV coordinates
        let atlas_size = self.texture.size();
        let region = AtlasRegion {
            x: packed.x as u32,
            y: packed.y as u32,
            width,
            height,
            uv_min: [
                packed.x as f32 / atlas_size.width as f32,
                packed.y as f32 / atlas_size.height as f32,
            ],
            uv_max: [
                (packed.x + width as i32) as f32 / atlas_size.width as f32,
                (packed.y + height as i32) as f32 / atlas_size.height as f32,
            ],
        };

        self.regions.insert(name.to_string(), region.clone());
        Ok(region)
    }
}
```

## Code Examples

### Complete Texture Manager

```rust
use warcraft_rs::blp::*;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TextureManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    cache: Arc<TextureCache>,
    atlases: Arc<RwLock<Vec<TextureAtlas>>>,
    placeholder: Arc<GpuTexture>,
}

impl TextureManager {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let cache = Arc::new(TextureCache::new(
            device.clone(),
            queue.clone(),
            512, // 512 MB cache
        ));

        // Create placeholder texture
        let placeholder = create_placeholder_texture(&device, &queue);

        Self {
            device,
            queue,
            cache,
            atlases: Arc::new(RwLock::new(Vec::new())),
            placeholder: Arc::new(placeholder),
        }
    }

    pub async fn load_texture(&self, path: &str) -> Arc<GpuTexture> {
        match self.cache.get_texture(path).await {
            Ok(texture) => texture,
            Err(e) => {
                eprintln!("Failed to load texture {}: {}", path, e);
                self.placeholder.clone()
            }
        }
    }

    pub async fn load_texture_array(
        &self,
        paths: &[&str],
    ) -> Result<Texture, Box<dyn std::error::Error>> {
        let mut blps = Vec::new();
        let mut max_width = 0;
        let mut max_height = 0;

        // Load all BLPs
        for path in paths {
            let data = tokio::fs::read(path).await?;
            let blp = Blp::from_bytes(&data)?;
            max_width = max_width.max(blp.width());
            max_height = max_height.max(blp.height());
            blps.push(blp);
        }

        // Create texture array
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Texture Array"),
            size: Extent3d {
                width: max_width,
                height: max_height,
                depth_or_array_layers: blps.len() as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload each layer
        for (i, blp) in blps.iter().enumerate() {
            let rgba = convert_blp_to_rgba(blp, 0)?;

            self.queue.write_texture(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: i as u32,
                    },
                    aspect: TextureAspect::All,
                },
                &rgba,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(blp.width() * 4),
                    rows_per_image: Some(blp.height()),
                },
                Extent3d {
                    width: blp.width(),
                    height: blp.height(),
                    depth_or_array_layers: 1,
                },
            );
        }

        Ok(texture)
    }

    pub async fn create_atlas_for_textures(
        &self,
        textures: &[(String, String)], // (name, path) pairs
        atlas_size: u32,
    ) -> Result<Arc<TextureAtlas>, Box<dyn std::error::Error>> {
        let mut atlas = TextureAtlas::new(&self.device, atlas_size);

        for (name, path) in textures {
            let data = tokio::fs::read(path).await?;
            let blp = Blp::from_bytes(&data)?;

            atlas.add_texture(&self.queue, name, &blp)?;
        }

        let atlas_arc = Arc::new(atlas);
        self.atlases.write().await.push(atlas_arc.clone());

        Ok(atlas_arc)
    }
}

fn create_placeholder_texture(device: &Device, queue: &Queue) -> GpuTexture {
    // Create a simple checkerboard pattern
    let size = 64;
    let mut data = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let color = if (x / 8 + y / 8) % 2 == 0 { 255 } else { 128 };
            data[idx] = color;     // R
            data[idx + 1] = 0;     // G
            data[idx + 2] = color; // B
            data[idx + 3] = 255;   // A
        }
    }

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("Placeholder Texture"),
        size: Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(size as u32 * 4),
            rows_per_image: Some(size as u32),
        },
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
    );

    // Create view and sampler
    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor::default());

    GpuTexture {
        texture,
        view,
        sampler,
        bind_group: todo!(), // Create appropriate bind group
    }
}
```

### BLP Conversion Utilities

```rust
use image::{DynamicImage, ImageBuffer, Rgba};

pub fn blp_to_image(blp: &Blp) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let rgba_data = convert_blp_to_rgba(blp, 0)?;
    let width = blp.width();
    let height = blp.height();

    let image_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
        width,
        height,
        rgba_data,
    ).ok_or("Failed to create image buffer")?;

    Ok(DynamicImage::ImageRgba8(image_buffer))
}

pub fn image_to_blp(
    image: &DynamicImage,
    format: BlpFormat,
    generate_mipmaps: bool,
) -> Result<Blp, Box<dyn std::error::Error>> {
    let rgba = image.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());

    // Ensure power-of-two dimensions
    if !width.is_power_of_two() || !height.is_power_of_two() {
        return Err("BLP textures must have power-of-two dimensions".into());
    }

    let mut blp = Blp::new(width, height, format);

    // Add base mipmap
    match format {
        BlpFormat::Dxt1 | BlpFormat::Dxt3 | BlpFormat::Dxt5 => {
            let compressed = compress_to_dxt(&rgba, format)?;
            blp.add_mipmap(0, compressed);
        }
        BlpFormat::Uncompressed => {
            let bgra = convert_rgba_to_bgra(&rgba);
            blp.add_mipmap(0, bgra);
        }
        _ => return Err("Unsupported BLP format for conversion".into()),
    }

    // Generate mipmaps if requested
    if generate_mipmaps {
        let mut current = rgba.clone();
        let mut level = 1;

        while current.width() > 1 && current.height() > 1 {
            current = image::imageops::resize(
                &current,
                current.width() / 2,
                current.height() / 2,
                image::imageops::FilterType::Lanczos3,
            );

            let mipmap_data = match format {
                BlpFormat::Dxt1 | BlpFormat::Dxt3 | BlpFormat::Dxt5 => {
                    compress_to_dxt(&current, format)?
                }
                BlpFormat::Uncompressed => {
                    convert_rgba_to_bgra(&current)
                }
                _ => unreachable!(),
            };

            blp.add_mipmap(level, mipmap_data);
            level += 1;
        }
    }

    Ok(blp)
}

fn compress_to_dxt(
    image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    format: BlpFormat,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use squish::{Format, CompressImage};

    let squish_format = match format {
        BlpFormat::Dxt1 => Format::Bc1,
        BlpFormat::Dxt3 => Format::Bc2,
        BlpFormat::Dxt5 => Format::Bc3,
        _ => return Err("Invalid DXT format".into()),
    };

    let compressed = compress_image(
        image.as_raw(),
        image.width() as i32,
        image.height() as i32,
        squish_format,
    );

    Ok(compressed)
}
```

## Best Practices

### 1. Texture Streaming

```rust
pub struct TextureStreamer {
    loader_thread: Option<std::thread::JoinHandle<()>>,
    request_sender: mpsc::Sender<TextureRequest>,
    result_receiver: mpsc::Receiver<TextureResult>,
}

struct TextureRequest {
    path: String,
    priority: u32,
}

struct TextureResult {
    path: String,
    texture: Result<Blp, Box<dyn std::error::Error>>,
}

impl TextureStreamer {
    pub fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel();
        let (result_tx, result_rx) = mpsc::channel();

        let loader_thread = std::thread::spawn(move || {
            texture_loader_thread(request_rx, result_tx);
        });

        Self {
            loader_thread: Some(loader_thread),
            request_sender: request_tx,
            result_receiver: result_rx,
        }
    }

    pub fn request_texture(&self, path: String, priority: u32) {
        let _ = self.request_sender.send(TextureRequest { path, priority });
    }

    pub fn poll_results(&self) -> Vec<TextureResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.result_receiver.try_recv() {
            results.push(result);
        }
        results
    }
}

fn texture_loader_thread(
    requests: mpsc::Receiver<TextureRequest>,
    results: mpsc::Sender<TextureResult>,
) {
    let mut queue = BinaryHeap::new();

    loop {
        // Collect requests
        while let Ok(request) = requests.try_recv() {
            queue.push(request);
        }

        // Process highest priority request
        if let Some(request) = queue.pop() {
            let texture = load_blp_texture(&request.path);
            let _ = results.send(TextureResult {
                path: request.path,
                texture,
            });
        } else {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
```

### 2. Texture Quality Settings

```rust
pub struct TextureQualitySettings {
    pub max_texture_size: u32,
    pub force_dxt_compression: bool,
    pub generate_mipmaps: bool,
    pub anisotropic_filtering: u8,
}

impl TextureQualitySettings {
    pub fn high() -> Self {
        Self {
            max_texture_size: 4096,
            force_dxt_compression: false,
            generate_mipmaps: true,
            anisotropic_filtering: 16,
        }
    }

    pub fn medium() -> Self {
        Self {
            max_texture_size: 2048,
            force_dxt_compression: true,
            generate_mipmaps: true,
            anisotropic_filtering: 8,
        }
    }

    pub fn low() -> Self {
        Self {
            max_texture_size: 1024,
            force_dxt_compression: true,
            generate_mipmaps: false,
            anisotropic_filtering: 2,
        }
    }

    pub fn apply_to_blp(&self, blp: &mut Blp) {
        // Downscale if needed
        if blp.width() > self.max_texture_size || blp.height() > self.max_texture_size {
            let scale = (self.max_texture_size as f32 / blp.width().max(blp.height()) as f32).min(1.0);
            let new_width = (blp.width() as f32 * scale) as u32;
            let new_height = (blp.height() as f32 * scale) as u32;

            blp.resize(new_width, new_height);
        }

        // Force compression if needed
        if self.force_dxt_compression && !blp.is_compressed() {
            blp.compress_to_dxt();
        }
    }
}
```

### 3. Texture Preloading

```rust
pub struct TexturePreloader {
    preload_list: Vec<String>,
    loaded: Arc<RwLock<HashMap<String, Arc<GpuTexture>>>>,
}

impl TexturePreloader {
    pub fn new() -> Self {
        Self {
            preload_list: Vec::new(),
            loaded: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_preload_list(&mut self, paths: Vec<String>) {
        self.preload_list.extend(paths);
    }

    pub async fn preload_all(&self, texture_manager: &TextureManager) {
        let futures: Vec<_> = self.preload_list
            .iter()
            .map(|path| {
                let path = path.clone();
                let manager = texture_manager.clone();
                async move {
                    let texture = manager.load_texture(&path).await;
                    (path, texture)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut loaded = self.loaded.write().await;
        for (path, texture) in results {
            loaded.insert(path, texture);
        }
    }
}
```

## Common Issues and Solutions

### Issue: Out of Memory

**Problem**: Loading too many large textures causes GPU memory exhaustion.

**Solution**:

```rust
pub struct MemoryBudget {
    max_memory: usize,
    current_usage: AtomicUsize,
}

impl MemoryBudget {
    pub fn can_allocate(&self, size: usize) -> bool {
        self.current_usage.load(Ordering::Relaxed) + size <= self.max_memory
    }

    pub fn allocate(&self, size: usize) -> bool {
        let mut current = self.current_usage.load(Ordering::Relaxed);
        loop {
            if current + size > self.max_memory {
                return false;
            }

            match self.current_usage.compare_exchange(
                current,
                current + size,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }
}
```

### Issue: Texture Corruption

**Problem**: Textures appear corrupted or have wrong colors.

**Solution**:

```rust
fn validate_blp(blp: &Blp) -> Result<(), String> {
    // Check magic number
    if !blp.is_valid_signature() {
        return Err("Invalid BLP signature".to_string());
    }

    // Validate dimensions
    if !blp.width().is_power_of_two() || !blp.height().is_power_of_two() {
        return Err("BLP dimensions must be power of two".to_string());
    }

    // Validate mipmap chain
    let expected_levels = (blp.width().max(blp.height()) as f32).log2() as usize + 1;
    if blp.mipmap_count() > expected_levels {
        return Err("Invalid mipmap count".to_string());
    }

    Ok(())
}
```

### Issue: Performance with Many Small Textures

**Problem**: Rendering performance drops with many texture switches.

**Solution**:

```rust
// Use texture arrays for similar textures
pub fn batch_similar_textures(
    textures: &[Blp],
    device: &Device,
    queue: &Queue,
) -> Result<Texture, Box<dyn std::error::Error>> {
    // Group by size
    let mut groups: HashMap<(u32, u32), Vec<&Blp>> = HashMap::new();

    for texture in textures {
        let key = (texture.width(), texture.height());
        groups.entry(key).or_insert_with(Vec::new).push(texture);
    }

    // Create texture arrays for each size group
    for ((width, height), group) in groups {
        if group.len() > 1 {
            create_texture_array(device, queue, &group, width, height)?;
        }
    }

    Ok(())
}
```

## Performance Tips

### 1. GPU Format Selection

```rust
fn select_optimal_gpu_format(blp: &Blp, device: &Device) -> TextureFormat {
    let features = device.features();

    match blp.format() {
        BlpFormat::Dxt1 => {
            if features.contains(Features::TEXTURE_COMPRESSION_BC) {
                if blp.has_alpha() {
                    TextureFormat::Bc1RgbaUnorm
                } else {
                    TextureFormat::Bc1RgbaUnorm // No separate RGB format in wgpu
                }
            } else {
                TextureFormat::Rgba8Unorm
            }
        }
        BlpFormat::Dxt3 => {
            if features.contains(Features::TEXTURE_COMPRESSION_BC) {
                TextureFormat::Bc2RgbaUnorm
            } else {
                TextureFormat::Rgba8Unorm
            }
        }
        BlpFormat::Dxt5 => {
            if features.contains(Features::TEXTURE_COMPRESSION_BC) {
                TextureFormat::Bc3RgbaUnorm
            } else {
                TextureFormat::Rgba8Unorm
            }
        }
        _ => TextureFormat::Rgba8Unorm,
    }
}
```

### 2. Async Texture Loading

```rust
use futures::stream::{FuturesUnordered, StreamExt};

pub async fn load_textures_parallel(
    paths: Vec<String>,
    max_concurrent: usize,
) -> Vec<Result<Blp, Box<dyn std::error::Error>>> {
    let mut futures = FuturesUnordered::new();
    let mut results = Vec::with_capacity(paths.len());

    for (i, path) in paths.into_iter().enumerate() {
        if futures.len() >= max_concurrent {
            if let Some(result) = futures.next().await {
                results.push(result);
            }
        }

        futures.push(async move {
            let data = tokio::fs::read(&path).await?;
            Blp::from_bytes(&data)
        });
    }

    // Collect remaining futures
    while let Some(result) = futures.next().await {
        results.push(result);
    }

    results
}
```

### 3. Texture Compression Cache

```rust
use std::path::PathBuf;

pub struct CompressionCache {
    cache_dir: PathBuf,
}

impl CompressionCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).unwrap();
        Self { cache_dir }
    }

    pub fn get_compressed_path(&self, original_path: &str, format: BlpFormat) -> PathBuf {
        let hash = calculate_file_hash(original_path);
        let filename = format!("{}_{}_{:?}.cache",
            Path::new(original_path).file_stem().unwrap().to_str().unwrap(),
            hash,
            format
        );
        self.cache_dir.join(filename)
    }

    pub fn load_or_compress(
        &self,
        blp: &Blp,
        original_path: &str,
        target_format: BlpFormat,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let cache_path = self.get_compressed_path(original_path, target_format);

        // Check cache
        if cache_path.exists() {
            return Ok(std::fs::read(cache_path)?);
        }

        // Compress
        let compressed = match target_format {
            BlpFormat::Dxt1 => compress_to_dxt1(blp)?,
            BlpFormat::Dxt3 => compress_to_dxt3(blp)?,
            BlpFormat::Dxt5 => compress_to_dxt5(blp)?,
            _ => return Err("Unsupported compression format".into()),
        };

        // Save to cache
        std::fs::write(cache_path, &compressed)?;

        Ok(compressed)
    }
}
```

## Related Guides

- [📦 Working with MPQ Archives](./mpq-archives.md) - Extract BLP textures from archives
- [🎭 Loading M2 Models](./m2-models.md) - Apply textures to models
- [🌍 Rendering ADT Terrain](./adt-rendering.md) - Texture terrain with BLPs
- [🏛️ WMO Rendering Guide](./wmo-rendering.md) - Texture world objects
- [🎨 Model Rendering Guide](./model-rendering.md) - Advanced texture techniques

## References

- [BLP Format Documentation](https://wowdev.wiki/BLP) - Complete BLP format specification
- [DXT Compression](https://docs.microsoft.com/en-us/windows/win32/direct3d11/texture-block-compression) - Understanding DXT formats
- [Texture Best Practices](https://developer.nvidia.com/content/understanding-compressed-texture-formats) - GPU texture optimization
- [WoW Model Viewer](https://github.com/Marlamin/WoWModelViewer) - Reference implementation
