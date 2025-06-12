// normal_map.rs - Extract normal maps from ADT files

use crate::Adt;
use crate::error::Result;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Format for normal map export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalMapFormat {
    /// Raw data format (just the values)
    Raw,
    /// PNG format (requires image feature)
    PNG,
    // /// DirectDraw Surface format (requires dds_encoder feature)
    // DDS,
}

/// Options for normal map extraction
#[derive(Debug, Clone)]
pub struct NormalMapOptions {
    /// Output format
    pub format: NormalMapFormat,
    /// Whether to invert Y axis
    pub invert_y: bool,
    /// Whether to use tangent space normals (vs object space)
    pub tangent_space: bool,
    /// Whether to flip the Y axis in the image
    pub flip_y: bool,
    /// Whether to flip the X axis in the image
    pub flip_x: bool,
    /// Normal map channel encoding mode
    pub encoding: NormalChannelEncoding,
}

/// Normal map channel encoding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalChannelEncoding {
    /// Standard RGB encoding where R=X, G=Y, B=Z
    RGB,
    /// OpenGL normal map format (Y flipped)
    OpenGL,
    /// DirectX normal map format (Y and Z flipped)
    DirectX,
}

impl Default for NormalMapOptions {
    fn default() -> Self {
        Self {
            format: NormalMapFormat::PNG,
            invert_y: false,
            tangent_space: true,
            flip_y: false,
            flip_x: false,
            encoding: NormalChannelEncoding::RGB,
        }
    }
}

/// Extract a normal map from an ADT file
pub fn extract_normal_map<P: AsRef<Path>>(
    adt: &Adt,
    output_path: P,
    options: NormalMapOptions,
) -> Result<()> {
    match options.format {
        NormalMapFormat::Raw => extract_raw_normal_map(adt, output_path, &options),
        NormalMapFormat::PNG => {
            #[cfg(feature = "image")]
            {
                extract_png_normal_map(adt, output_path, &options)
            }
            #[cfg(not(feature = "image"))]
            {
                Err(crate::error::AdtError::NotImplemented(
                    "PNG export requires the 'image' feature to be enabled".to_string(),
                ))
            }
        } // NormalMapFormat::DDS => {
          //     #[cfg(feature = "dds_encoder")]
          //     {
          //         extract_dds_normal_map(adt, output_path, &options)
          //     }
          //     #[cfg(not(feature = "dds_encoder"))]
          //     {
          //         Err(crate::error::AdtError::NotImplemented(
          //             "DDS export requires the 'dds_encoder' feature to be enabled".to_string(),
          //         ))
          //     }
          // }
    }
}

/// Extract a raw normal map (just the values)
fn extract_raw_normal_map<P: AsRef<Path>>(
    adt: &Adt,
    output_path: P,
    options: &NormalMapOptions,
) -> Result<()> {
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    // The normal map is a grid of normal vectors for each vertex in the heightmap

    // For each MCNK in the grid
    for y in 0..16 {
        for x in 0..16 {
            let chunk_index = y * 16 + x;

            if chunk_index < adt.mcnk_chunks.len() {
                let chunk = &adt.mcnk_chunks[chunk_index];

                // Get the normal values from this chunk
                for normal in &chunk.normals {
                    // Convert normal from [u8; 3] format to normalized [-1, 1] floats
                    let nx = (normal[0] as f32) / 127.0;
                    let mut ny = (normal[1] as f32) / 127.0;
                    let mut nz = (normal[2] as f32) / 127.0;

                    // Apply options
                    if options.invert_y {
                        ny = -ny;
                    }

                    if options.encoding == NormalChannelEncoding::DirectX {
                        ny = -ny;
                        nz = -nz;
                    } else if options.encoding == NormalChannelEncoding::OpenGL {
                        ny = -ny;
                    }

                    // Write the normal components
                    writer.write_all(&nx.to_le_bytes())?;
                    writer.write_all(&ny.to_le_bytes())?;
                    writer.write_all(&nz.to_le_bytes())?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "image")]
fn extract_png_normal_map<P: AsRef<Path>>(
    adt: &Adt,
    output_path: P,
    options: &NormalMapOptions,
) -> Result<()> {
    use image::{ImageBuffer, Rgb};

    // Calculate final image dimensions
    // Each MCNK contains a 9x9 grid of vertices
    let width = 145; // 9*16 + 1 (with overlap)
    let height = 145; // 9*16 + 1 (with overlap)

    // Create a new RGB image
    let mut img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(width, height);

    // Collect all normals first
    let mut normals = vec![[0u8; 3]; width as usize * height as usize];

    // Process each MCNK to extract normals
    for y in 0..16 {
        for x in 0..16 {
            let chunk_index = y * 16 + x;

            if chunk_index < adt.mcnk_chunks.len() {
                let chunk = &adt.mcnk_chunks[chunk_index];

                // Map the chunk to the image grid
                let chunk_x = x * 9;
                let chunk_y = y * 9;

                // Place normals on the grid
                for i in 0..9 {
                    for j in 0..9 {
                        let normal_index = i * 9 + j;
                        let pos_x = chunk_x + j;
                        let pos_y = if options.flip_y {
                            height as usize - 1 - (chunk_y + i)
                        } else {
                            chunk_y + i
                        };

                        if normal_index < chunk.normals.len() {
                            let normal = chunk.normals[normal_index];

                            // Store normal
                            let combined_index = pos_y * width as usize + pos_x;
                            if combined_index < normals.len() {
                                normals[combined_index] = normal;
                            }
                        }
                    }
                }
            }
        }
    }

    // Convert normals to RGB image
    for (i, normal) in normals.iter().enumerate() {
        let x = (i % width as usize) as u32;
        let y = (i / width as usize) as u32;

        // Convert from [-127, 127] to [0, 255]
        // Note: ADT normals are actually stored as signed bytes (-127 to 127)

        // Get normal components - convert from u8 to i8 first
        let mut nx = normal[0] as i8;
        let mut ny = normal[1] as i8;
        let nz = normal[2] as i8;

        // Apply options
        if options.invert_y {
            ny = -ny;
        }

        if options.flip_x {
            nx = -nx;
        }

        // Convert to [0, 255] range (127 is 0, 255 is 1.0, 0 is -1.0)
        let r = ((nx + 127) as u8).clamp(0, 255);
        let g = ((ny + 127) as u8).clamp(0, 255);
        let b = ((nz + 127) as u8).clamp(0, 255);

        // Set the pixel
        img.put_pixel(x, y, Rgb([r, g, b]));
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

// #[cfg(feature = "dds_encoder")]
// fn extract_dds_normal_map<P: AsRef<Path>>(
//     _adt: &Adt,
//     _output_path: P,
//     _options: &NormalMapOptions,
// ) -> Result<()> {
//     // Implementation would use a DDS encoder library to save the normal map
//     // For now, return a not implemented error
//     Err(crate::error::AdtError::NotImplemented(
//         "DDS export is not yet implemented".to_string(),
//     ))
// }

/// Generate a normal map from a height map
#[allow(dead_code)]
pub fn generate_normal_map_from_heightmap(
    heightmap: &[f32],
    width: usize,
    height: usize,
    scale: f32,
) -> Vec<[i8; 3]> {
    let mut normals = vec![[0i8; 3]; width * height];

    // For each vertex in the height map
    for y in 0..height {
        for x in 0..width {
            // Get surrounding heights (with bounds checking)
            let h_center = heightmap[y * width + x];

            let h_left = if x > 0 {
                heightmap[y * width + (x - 1)]
            } else {
                h_center
            };

            let h_right = if x < width - 1 {
                heightmap[y * width + (x + 1)]
            } else {
                h_center
            };

            let h_up = if y > 0 {
                heightmap[(y - 1) * width + x]
            } else {
                h_center
            };

            let h_down = if y < height - 1 {
                heightmap[(y + 1) * width + x]
            } else {
                h_center
            };

            // Calculate derivatives
            let dx = (h_right - h_left) * scale;
            let dy = (h_down - h_up) * scale;

            // Calculate normal vector using cross product
            let nx = -dx;
            let ny = -dy;
            let nz = 2.0; // Fixed step size

            // Normalize
            let length = (nx * nx + ny * ny + nz * nz).sqrt();

            // Convert to -127 to 127 range for ADT normals
            let nx_scaled = ((nx / length) * 127.0) as i8;
            let ny_scaled = ((ny / length) * 127.0) as i8;
            let nz_scaled = ((nz / length) * 127.0) as i8;

            // Store normal
            normals[y * width + x] = [nx_scaled, ny_scaled, nz_scaled];
        }
    }

    normals
}
