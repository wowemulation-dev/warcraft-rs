pub use crate::model_export::{ModelExportOptions, ModelFormat, export_to_3d};
pub use crate::normal_map::{
    NormalChannelEncoding, NormalMapFormat, NormalMapOptions, extract_normal_map,
}; // extract.rs - Extract data from ADT files

use crate::Adt;
use crate::error::Result;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Image formats for extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Raw data format (just the values)
    Raw,
    /// Portable GrayMap format (PGM)
    PGM,
    /// Portable Network Graphics (PNG)
    PNG,
    /// Tagged Image File Format (TIFF)
    TIFF,
}

/// Options for heightmap extraction
#[derive(Debug, Clone)]
pub struct HeightmapOptions {
    /// Output format
    pub format: ImageFormat,
    /// Minimum height (will be mapped to 0 in the output)
    pub min_height: Option<f32>,
    /// Maximum height (will be mapped to max value in the output)
    pub max_height: Option<f32>,
    /// Whether to interpolate missing values
    pub interpolate: bool,
    /// Whether to flip the Y axis
    pub flip_y: bool,
    /// Number of bits per pixel (8, 16, or 32)
    pub bits_per_pixel: u8,
}

impl Default for HeightmapOptions {
    fn default() -> Self {
        Self {
            format: ImageFormat::PGM,
            min_height: None,
            max_height: None,
            interpolate: true,
            flip_y: false,
            bits_per_pixel: 16,
        }
    }
}

/// Extract a heightmap from an ADT file
pub fn extract_heightmap<P: AsRef<Path>>(
    adt: &Adt,
    output_path: P,
    options: HeightmapOptions,
) -> Result<()> {
    match options.format {
        ImageFormat::Raw => extract_raw_heightmap(adt, output_path, &options),
        ImageFormat::PGM => extract_pgm_heightmap(adt, output_path, &options),
        ImageFormat::PNG => {
            #[cfg(feature = "image")]
            {
                extract_png_heightmap(adt, output_path, &options)
            }
            #[cfg(not(feature = "image"))]
            {
                Err(crate::error::AdtError::NotImplemented(
                    "PNG export requires the 'image' feature to be enabled".to_string(),
                ))
            }
        }
        ImageFormat::TIFF => {
            #[cfg(feature = "image")]
            {
                extract_tiff_heightmap(adt, output_path, &options)
            }
            #[cfg(not(feature = "image"))]
            {
                Err(crate::error::AdtError::NotImplemented(
                    "TIFF export requires the 'image' feature to be enabled".to_string(),
                ))
            }
        }
    }
}

/// Extract a raw heightmap (just the values)
fn extract_raw_heightmap<P: AsRef<Path>>(
    adt: &Adt,
    output_path: P,
    options: &HeightmapOptions,
) -> Result<()> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    // The heightmap is a grid of 17x17 points for each MCNK (16x16 grid of MCNKs)
    // In total, that's 145 points per MCNK (9x9 grid with additional control points)

    // Determine global min/max heights if not provided
    let (min_height, max_height) = get_height_range(adt, options);

    // For each MCNK in the grid
    for y in 0..16 {
        for x in 0..16 {
            let chunk_index = y * 16 + x;

            if chunk_index < adt.mcnk_chunks.len() {
                let chunk = &adt.mcnk_chunks[chunk_index];

                // Get the height values from this chunk
                for height in &chunk.height_map {
                    // Normalize the height to the range
                    let normalized =
                        normalize_height(*height, min_height, max_height, options.bits_per_pixel);

                    // Write the normalized value based on bit depth
                    match options.bits_per_pixel {
                        8 => writer.write_all(&[normalized as u8])?,
                        16 => writer.write_all(&(normalized as u16).to_le_bytes())?,
                        32 => writer.write_all(&normalized.to_le_bytes())?,
                        _ => {
                            return Err(crate::error::AdtError::ParseError(format!(
                                "Unsupported bits per pixel: {}",
                                options.bits_per_pixel
                            )));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Extract a PGM heightmap (portable graymap format)
fn extract_pgm_heightmap<P: AsRef<Path>>(
    adt: &Adt,
    output_path: P,
    options: &HeightmapOptions,
) -> Result<()> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    // Determine global min/max heights if not provided
    let (min_height, max_height) = get_height_range(adt, options);

    // Calculate final image dimensions
    // Each MCNK contains a 9x9 grid of height values (including control points)
    // We need to combine them into a single image
    let width = 17 * 16; // 17 points per chunk (including overlapping edges) * 16 chunks
    let height = 17 * 16;

    // Write PGM header
    match options.bits_per_pixel {
        8 => {
            // 8-bit grayscale
            writeln!(&mut writer, "P5")?;
            writeln!(&mut writer, "{} {}", width, height)?;
            writeln!(&mut writer, "255")?;
        }
        16 => {
            // 16-bit grayscale
            writeln!(&mut writer, "P5")?;
            writeln!(&mut writer, "{} {}", width, height)?;
            writeln!(&mut writer, "65535")?;
        }
        _ => {
            return Err(crate::error::AdtError::ParseError(format!(
                "Unsupported bits per pixel for PGM: {}",
                options.bits_per_pixel
            )));
        }
    }

    // Create a combined heightmap
    let mut combined = vec![0.0; width * height];

    // Overlay the chunks to create the final heightmap
    // Each chunk is a 9x9 grid that needs to be mapped to the right position
    for y in 0..16 {
        for x in 0..16 {
            let chunk_index = y * 16 + x;

            if chunk_index < adt.mcnk_chunks.len() {
                let chunk = &adt.mcnk_chunks[chunk_index];

                // The MCNK grid is 9x9 vertices, but each chunk overlaps with neighboring chunks
                // at the edges. The full grid is actually 17x17 (including overlapping edges)
                let chunk_x = x * 17;
                let chunk_y = y * 17;

                // Map the 9x9 height grid to the combined image
                for i in 0..9 {
                    for j in 0..9 {
                        let height_index = i * 9 + j; // Index in the MCNK height map
                        let pos_x = chunk_x + j;
                        let pos_y = if options.flip_y {
                            height - 1 - (chunk_y + i)
                        } else {
                            chunk_y + i
                        };

                        let combined_index = pos_y * width + pos_x;

                        if height_index < chunk.height_map.len() && combined_index < combined.len()
                        {
                            combined[combined_index] = chunk.height_map[height_index];
                        }
                    }
                }

                // To properly handle the heightmap, we need to include the additional vertices
                // that are part of the MCVT data (8x8 grid of extra control points)
                // This is more complex and would need the full MCVT parsing
            }
        }
    }

    // Write the combined heightmap to the PGM file
    for &height in &combined {
        // Normalize the height to the output range
        let normalized = normalize_height(height, min_height, max_height, options.bits_per_pixel);

        // Write to PGM based on bit depth
        match options.bits_per_pixel {
            8 => {
                let value = normalized as u8;
                writer.write_all(&[value])?;
            }
            16 => {
                // PGM format expects big-endian for 16-bit
                let value = normalized as u16;
                writer.write_all(&[(value >> 8) as u8, value as u8])?;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

#[cfg(feature = "image")]
fn extract_png_heightmap<P: AsRef<Path>>(
    adt: &Adt,
    output_path: P,
    options: &HeightmapOptions,
) -> Result<()> {
    use image::{GrayImage, Luma};

    // Determine global min/max heights if not provided
    let (min_height, max_height) = get_height_range(adt, options);

    // Calculate final image dimensions
    let width = 17 * 16; // 17 points per chunk (including overlapping edges) * 16 chunks
    let height = 17 * 16;

    // Create a new grayscale image
    let mut img = GrayImage::new(width as u32, height as u32);

    // Process each MCNK to build the heightmap
    for y in 0..16 {
        for x in 0..16 {
            let chunk_index = y * 16 + x;

            if chunk_index < adt.mcnk_chunks.len() {
                let chunk = &adt.mcnk_chunks[chunk_index];

                // Map the chunk to the image grid
                let chunk_x = x * 17;
                let chunk_y = y * 17;

                // Map the 9x9 height grid to the combined image
                for i in 0..9 {
                    for j in 0..9 {
                        let height_index = i * 9 + j;
                        let pos_x = chunk_x + j;
                        let pos_y = if options.flip_y {
                            height - 1 - (chunk_y + i)
                        } else {
                            chunk_y + i
                        };

                        if height_index < chunk.height_map.len() {
                            let height = chunk.height_map[height_index];

                            // Normalize the height to 0-255 range for the image
                            let normalized =
                                normalize_height(height, min_height, max_height, 8) as u8;

                            // Set the pixel
                            img.put_pixel(pos_x as u32, pos_y as u32, Luma([normalized]));
                        }
                    }
                }
            }
        }
    }

    // Save the image
    img.save(output_path).map_err(|e| {
        crate::error::AdtError::Io(std::io::Error::other(format!(
            "Failed to save PNG image: {}",
            e
        )))
    })?;

    Ok(())
}

#[cfg(feature = "image")]
fn extract_tiff_heightmap<P: AsRef<Path>>(
    adt: &Adt,
    output_path: P,
    options: &HeightmapOptions,
) -> Result<()> {
    use image::{ImageBuffer, LumaA};

    // For TIFF, we can create 16-bit images
    let (min_height, max_height) = get_height_range(adt, options);

    // Calculate final image dimensions
    let width = 17 * 16;
    let height = 17 * 16;

    let mut img = ImageBuffer::<LumaA<u16>, Vec<u16>>::new(width as u32, height as u32);

    // Process each MCNK to build the heightmap
    for y in 0..16 {
        for x in 0..16 {
            let chunk_index = y * 16 + x;

            if chunk_index < adt.mcnk_chunks.len() {
                let chunk = &adt.mcnk_chunks[chunk_index];

                // Map the chunk to the image grid
                let chunk_x = x * 17;
                let chunk_y = y * 17;

                // Map the 9x9 height grid to the combined image
                for i in 0..9 {
                    for j in 0..9 {
                        let height_index = i * 9 + j;
                        let pos_x = chunk_x + j;
                        let pos_y = if options.flip_y {
                            height - 1 - (chunk_y + i)
                        } else {
                            chunk_y + i
                        };

                        if height_index < chunk.height_map.len() {
                            let height = chunk.height_map[height_index];

                            // Normalize to 16-bit range
                            let normalized =
                                normalize_height(height, min_height, max_height, 16) as u16;

                            // Set the pixel (TIFF uses LumaA<u16> with alpha)
                            img.put_pixel(pos_x as u32, pos_y as u32, LumaA([normalized, 65535]));
                        }
                    }
                }
            }
        }
    }

    // Save the image
    img.save(output_path).map_err(|e| {
        crate::error::AdtError::Io(std::io::Error::other(format!(
            "Failed to save TIFF image: {}",
            e
        )))
    })?;

    Ok(())
}

/// Determine the minimum and maximum heights in the ADT
fn get_height_range(adt: &Adt, options: &HeightmapOptions) -> (f32, f32) {
    // Use provided min/max if specified
    if let (Some(min), Some(max)) = (options.min_height, options.max_height) {
        return (min, max);
    }

    // Otherwise calculate from the ADT data
    let mut min_height = f32::MAX;
    let mut max_height = f32::MIN;

    for chunk in &adt.mcnk_chunks {
        for &height in &chunk.height_map {
            min_height = min_height.min(height);
            max_height = max_height.max(height);
        }
    }

    // If still using defaults, ensure a reasonable range
    if min_height == f32::MAX || max_height == f32::MIN {
        min_height = 0.0;
        max_height = 1000.0;
    }

    (
        options.min_height.unwrap_or(min_height),
        options.max_height.unwrap_or(max_height),
    )
}

/// Normalize a height value to the target bit depth
fn normalize_height(height: f32, min: f32, max: f32, bits_per_pixel: u8) -> u32 {
    let range = max - min;

    // Avoid division by zero
    if range <= 0.0 {
        return 0;
    }

    // Normalize to 0.0-1.0 range
    let normalized = (height - min) / range;

    // Scale to target bit depth
    let max_value = match bits_per_pixel {
        8 => 255u32,
        16 => 65535u32,
        32 => u32::MAX,
        _ => 255u32, // Default to 8-bit
    };

    (normalized * max_value as f32) as u32
}

/// Extract a normal map from an ADT file
pub fn extract_normalmap<P: AsRef<Path>>(
    _adt: &Adt,
    _output_path: P,
    _options: HeightmapOptions,
) -> Result<()> {
    #[cfg(feature = "image")]
    {
        // Implementation for normal map extraction would go here
        // Similar to heightmap but using the normals from MCNR
        Err(crate::error::AdtError::NotImplemented(
            "Normal map extraction is not yet implemented".to_string(),
        ))
    }

    #[cfg(not(feature = "image"))]
    {
        Err(crate::error::AdtError::NotImplemented(
            "Normal map export requires the 'image' feature to be enabled".to_string(),
        ))
    }
}

/// Extract texture information from an ADT file
pub fn extract_textures<P: AsRef<Path>>(adt: &Adt, output_dir: P) -> Result<()> {
    let output_dir = output_dir.as_ref();

    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    // Write texture list
    let texture_list_path = output_dir.join("textures.txt");
    let mut texture_list = File::create(texture_list_path)?;

    if let Some(ref mtex) = adt.mtex {
        for (i, filename) in mtex.filenames.iter().enumerate() {
            writeln!(&mut texture_list, "{}: {}", i, filename)?;
        }
    }

    // Write texture layer info for each MCNK
    let layer_info_path = output_dir.join("layers.txt");
    let mut layer_info = File::create(layer_info_path)?;

    // For each MCNK in order
    for y in 0..16 {
        for x in 0..16 {
            let chunk_index = y * 16 + x;

            if chunk_index < adt.mcnk_chunks.len() {
                let chunk = &adt.mcnk_chunks[chunk_index];

                writeln!(
                    &mut layer_info,
                    "Chunk ({}, {}) - {} layers:",
                    x,
                    y,
                    chunk.texture_layers.len()
                )?;

                for (layer_idx, layer) in chunk.texture_layers.iter().enumerate() {
                    writeln!(
                        &mut layer_info,
                        "  Layer {}: Texture ID {}, Flags {:08X}",
                        layer_idx, layer.texture_id, layer.flags
                    )?;
                }

                writeln!(&mut layer_info)?;
            }
        }
    }

    Ok(())
}

/// Extract model information from an ADT file
pub fn extract_models<P: AsRef<Path>>(adt: &Adt, output_dir: P) -> Result<()> {
    let output_dir = output_dir.as_ref();

    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    // Write doodad model list
    if let Some(ref mmdx) = adt.mmdx {
        let doodad_path = output_dir.join("doodads.txt");
        let mut doodad_file = File::create(doodad_path)?;

        for (i, filename) in mmdx.filenames.iter().enumerate() {
            writeln!(&mut doodad_file, "{}: {}", i, filename)?;
        }
    }

    // Write WMO model list
    if let Some(ref mwmo) = adt.mwmo {
        let wmo_path = output_dir.join("wmos.txt");
        let mut wmo_file = File::create(wmo_path)?;

        for (i, filename) in mwmo.filenames.iter().enumerate() {
            writeln!(&mut wmo_file, "{}: {}", i, filename)?;
        }
    }

    // Write doodad placements
    if let Some(ref mddf) = adt.mddf {
        let placements_path = output_dir.join("doodad_placements.txt");
        let mut placements_file = File::create(placements_path)?;

        for (i, doodad) in mddf.doodads.iter().enumerate() {
            writeln!(
                &mut placements_file,
                "Doodad {}: ID {}, Position [{:.2}, {:.2}, {:.2}], Rotation [{:.2}, {:.2}, {:.2}], Scale {:.2}",
                i,
                doodad.name_id,
                doodad.position[0],
                doodad.position[1],
                doodad.position[2],
                doodad.rotation[0],
                doodad.rotation[1],
                doodad.rotation[2],
                doodad.scale
            )?;
        }
    }

    // Write WMO placements
    if let Some(ref modf) = adt.modf {
        let placements_path = output_dir.join("wmo_placements.txt");
        let mut placements_file = File::create(placements_path)?;

        for (i, model) in modf.models.iter().enumerate() {
            writeln!(
                &mut placements_file,
                "WMO {}: ID {}, Position [{:.2}, {:.2}, {:.2}], Rotation [{:.2}, {:.2}, {:.2}]",
                i,
                model.name_id,
                model.position[0],
                model.position[1],
                model.position[2],
                model.rotation[0],
                model.rotation[1],
                model.rotation[2]
            )?;
        }
    }

    Ok(())
}
