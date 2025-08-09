use std::fs::File;
use wow_data::prelude::*;

use custom_debug::Debug;
use wow_data::types::WowString;
use wow_utils::debug;

use std::path::Path;

use crate::chunks::M2Vertex;
use crate::chunks::animation::M2Animation;
use crate::chunks::bone::M2Bone;
use crate::chunks::material::M2Material;
use crate::chunks::texture::M2Texture;
use crate::error::Result;
use crate::header::M2Header;

/// Main M2 model structure
#[derive(Debug, Clone)]
pub struct M2Model {
    /// M2 header
    pub header: M2Header,
    /// Model name
    pub name: String,
    /// Global sequences
    pub global_sequences: Vec<u32>,
    /// Animations
    #[debug(with = debug::trimmed_collection_fmt)]
    pub animations: Vec<M2Animation>,
    /// Animation lookups
    #[debug(with = debug::trimmed_collection_fmt)]
    pub animation_lookup: Vec<u16>,
    /// Bones
    #[debug(with = debug::trimmed_collection_fmt)]
    pub bones: Vec<M2Bone>,
    /// Key bone lookups
    #[debug(with = debug::trimmed_collection_fmt)]
    pub key_bone_lookup: Vec<u16>,
    /// Vertices
    #[debug(with = debug::trimmed_collection_fmt)]
    pub vertices: Vec<M2Vertex>,
    /// Textures
    #[debug(with = debug::trimmed_collection_fmt)]
    pub textures: Vec<M2Texture>,
    /// Materials (render flags)
    #[debug(with = debug::trimmed_collection_fmt)]
    pub materials: Vec<M2Material>,
    /// Raw data for other sections
    /// This is used to preserve data that we don't fully parse yet
    pub raw_data: M2RawData,
}

/// Raw data for sections that are not fully parsed
#[derive(Debug, Clone, Default)]
pub struct M2RawData {
    /// Transparency data
    pub transparency: Vec<u8>,
    /// Texture animations
    pub texture_animations: Vec<u8>,
    /// Color animations
    pub color_animations: Vec<u8>,
    /// Render flags
    pub render_flags: Vec<u8>,
    /// Bone lookup table
    pub bone_lookup_table: Vec<u16>,
    /// Texture lookup table
    pub texture_lookup_table: Vec<u16>,
    /// Texture mapping lookup table
    pub texture_mapping_lookup_table: Vec<u16>,
    /// Texture units
    pub texture_units: Vec<u16>,
    /// Transparency lookup table
    pub transparency_lookup_table: Vec<u16>,
    /// Texture animation lookup
    pub texture_animation_lookup: Vec<u16>,
    /// Bounding triangles
    pub bounding_triangles: Vec<u8>,
    /// Bounding vertices
    pub bounding_vertices: Vec<u8>,
    /// Bounding normals
    pub bounding_normals: Vec<u8>,
    /// Attachments
    pub attachments: Vec<u8>,
    /// Attachment lookup table
    pub attachment_lookup_table: Vec<u16>,
    /// Events
    pub events: Vec<u8>,
    /// Lights
    pub lights: Vec<u8>,
    /// Cameras
    pub cameras: Vec<u8>,
    /// Camera lookup table
    pub camera_lookup_table: Vec<u16>,
    /// Ribbon emitters
    pub ribbon_emitters: Vec<u8>,
    /// Particle emitters
    pub particle_emitters: Vec<u8>,
    /// Texture combiner combos (added in Cataclysm)
    pub texture_combiner_combos: Option<Vec<u8>>,
}

impl M2Model {
    /// Parse an M2 model from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Parse the header first
        let header: M2Header = reader.wow_read()?;

        let name = String::from_wow_char_array(reader, header.name.clone())?;

        let global_sequences = header.global_sequences.wow_read_to_vec(reader)?;
        let animations = header.animations.wow_read_to_vec(reader, header.version)?;
        let animation_lookup = header.animation_lookup.wow_read_to_vec(reader)?;

        let bones = M2Bone::read_bone_array(reader, header.bones.clone(), header.version)?;

        let key_bone_lookup = header.key_bone_lookup.wow_read_to_vec(reader)?;
        let vertices = header.vertices.wow_read_to_vec(reader, header.version)?;

        let mut iter = header.textures.new_iterator(reader).unwrap();
        let mut textures = Vec::new();
        loop {
            match iter.next(|reader, item_header| {
                let item_header = match item_header {
                    Some(item) => item,
                    None => reader.wow_read()?,
                };
                textures.push(M2Texture {
                    data: reader.new_from_header(&item_header)?,
                    header: item_header,
                });
                Ok(())
            }) {
                Ok(is_active) => {
                    if !is_active {
                        break;
                    }
                }
                Err(err) => return Err(err.into()),
            }
        }

        let materials = header.materials.wow_read_to_vec(reader)?;

        // Parse raw data for other sections
        // These are sections we won't fully parse yet but want to preserve
        let raw_data = M2RawData::default();

        Ok(Self {
            header,
            name,
            global_sequences,
            animations,
            animation_lookup,
            bones,
            key_bone_lookup,
            vertices,
            textures,
            materials,
            raw_data,
        })
    }

    /// Load an M2 model from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        Self::parse(&mut file)
    }

    /// Save an M2 model to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path)?;
        self.write(&mut file)
    }

    /// Write an M2 model to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        todo!()
        //     // We need to recalculate all offsets and build the file in memory
        //     let mut data_section = Vec::new();
        //     let mut header = self.header.clone();
        //
        //     // Start with header size (will be written last)
        //     let header_size = self.calculate_header_size();
        //     let mut current_offset = header_size as u32;
        //
        //     // Write name
        //     if let Some(ref name) = self.name {
        //         let name_bytes = name.as_bytes();
        //         let name_len = name_bytes.len() as u32 + 1; // +1 for null terminator
        //         header.name = M2Array::new(name_len, current_offset);
        //
        //         data_section.extend_from_slice(name_bytes);
        //         data_section.push(0); // Null terminator
        //         current_offset += name_len;
        //     } else {
        //         header.name = M2Array::new(0, 0);
        //     }
        //
        //     // Write global sequences
        //     if !self.global_sequences.is_empty() {
        //         header.global_sequences =
        //             M2Array::new(self.global_sequences.len() as u32, current_offset);
        //
        //         for &seq in &self.global_sequences {
        //             data_section.extend_from_slice(&seq.to_le_bytes());
        //         }
        //
        //         current_offset += (self.global_sequences.len() * std::mem::size_of::<u32>()) as u32;
        //     } else {
        //         header.global_sequences = M2Array::new(0, 0);
        //     }
        //
        //     // Write animations
        //     if !self.animations.is_empty() {
        //         header.animations = M2Array::new(self.animations.len() as u32, current_offset);
        //
        //         for anim in &self.animations {
        //             // For each animation, write its data
        //             let mut anim_data = Vec::new();
        //             anim.write(&mut anim_data, header.version)?;
        //             data_section.extend_from_slice(&anim_data);
        //         }
        //
        //         // Animation size depends on version: 24 bytes for Classic, 52 bytes for BC+
        //         let anim_size = if header.version <= 256 { 24 } else { 52 };
        //         current_offset += (self.animations.len() * anim_size) as u32;
        //     } else {
        //         header.animations = M2Array::new(0, 0);
        //     }
        //
        //     // Write animation lookups
        //     if !self.animation_lookup.is_empty() {
        //         header.animation_lookup =
        //             M2Array::new(self.animation_lookup.len() as u32, current_offset);
        //
        //         for &lookup in &self.animation_lookup {
        //             data_section.extend_from_slice(&lookup.to_le_bytes());
        //         }
        //
        //         current_offset += (self.animation_lookup.len() * std::mem::size_of::<u16>()) as u32;
        //     } else {
        //         header.animation_lookup = M2Array::new(0, 0);
        //     }
        //
        //     // Write bones
        //     if !self.bones.is_empty() {
        //         header.bones = M2Array::new(self.bones.len() as u32, current_offset);
        //
        //         for bone in &self.bones {
        //             let mut bone_data = Vec::new();
        //             bone.write(&mut bone_data, self.header.version)?;
        //             data_section.extend_from_slice(&bone_data);
        //         }
        //
        //         // Bone size is 92 bytes for all versions we support
        //         let bone_size = 92;
        //         current_offset += (self.bones.len() * bone_size) as u32;
        //     } else {
        //         header.bones = M2Array::new(0, 0);
        //     }
        //
        //     // Write key bone lookups
        //     if !self.key_bone_lookup.is_empty() {
        //         header.key_bone_lookup =
        //             M2Array::new(self.key_bone_lookup.len() as u32, current_offset);
        //
        //         for &lookup in &self.key_bone_lookup {
        //             data_section.extend_from_slice(&lookup.to_le_bytes());
        //         }
        //
        //         current_offset += (self.key_bone_lookup.len() * std::mem::size_of::<u16>()) as u32;
        //     } else {
        //         header.key_bone_lookup = M2Array::new(0, 0);
        //     }
        //
        //     // Write vertices
        //     if !self.vertices.is_empty() {
        //         header.vertices = M2Array::new(self.vertices.len() as u32, current_offset);
        //
        //         let vertex_size =
        //             if self.header.version().unwrap_or(M2Version::Classic) >= M2Version::Cataclysm {
        //                 // Cataclysm and later have secondary texture coordinates
        //                 44
        //             } else {
        //                 // Pre-Cataclysm don't have secondary texture coordinates
        //                 36
        //             };
        //
        //         for vertex in &self.vertices {
        //             let mut vertex_data = Vec::new();
        //             vertex.write(&mut vertex_data, self.header.version)?;
        //             data_section.extend_from_slice(&vertex_data);
        //         }
        //
        //         current_offset += (self.vertices.len() * vertex_size) as u32;
        //     } else {
        //         header.vertices = M2Array::new(0, 0);
        //     }
        //
        //     // Write textures
        //     if !self.textures.is_empty() {
        //         header.textures = M2Array::new(self.textures.len() as u32, current_offset);
        //
        //         // First, we need to write the texture definitions
        //         let mut texture_name_offsets = Vec::new();
        //         let texture_def_size = 16; // Each texture definition is 16 bytes
        //
        //         for texture in &self.textures {
        //             // Save the current offset for this texture's filename
        //             texture_name_offsets
        //                 .push(current_offset + (self.textures.len() * texture_def_size) as u32);
        //
        //             // Write the texture definition (without the actual filename)
        //             let mut texture_def = Vec::new();
        //
        //             // Write texture type
        //             texture_def.extend_from_slice(&(texture.texture_type as u32).to_le_bytes());
        //
        //             // Write flags
        //             texture_def.extend_from_slice(&texture.flags.bits().to_le_bytes());
        //
        //             // Write filename offset and length (will be filled in later)
        //             texture_def.extend_from_slice(&0u32.to_le_bytes()); // Count
        //             texture_def.extend_from_slice(&0u32.to_le_bytes()); // Offset
        //
        //             data_section.extend_from_slice(&texture_def);
        //         }
        //
        //         // Now write the filenames
        //         current_offset += (self.textures.len() * texture_def_size) as u32;
        //
        //         // For each texture, update the offset in the definition and write the filename
        //         for (i, texture) in self.textures.iter().enumerate() {
        //             // Get the filename
        //             let filename_offset = texture.filename.array.offset as usize;
        //             let filename_len = texture.filename.array.count as usize;
        //             // Not every texture has a filename (some are hardcoded)
        //             if filename_offset == 0 || filename_len == 0 {
        //                 continue;
        //             }
        //
        //             // Calculate the offset in the data section where this texture's definition was written
        //             // The texture definitions start at (header.textures.offset - base_data_offset)
        //             let base_data_offset = std::mem::size_of::<M2Header>();
        //             let def_offset_in_data = (header.textures.offset as usize - base_data_offset)
        //                 + (i * texture_def_size)
        //                 + 8;
        //
        //             // Update the count and offset for the filename
        //             data_section[def_offset_in_data..def_offset_in_data + 4]
        //                 .copy_from_slice(&(filename_len as u32).to_le_bytes());
        //             data_section[def_offset_in_data + 4..def_offset_in_data + 8]
        //                 .copy_from_slice(&current_offset.to_le_bytes());
        //
        //             // Write the filename
        //             data_section.extend_from_slice(&texture.filename.string.data);
        //             data_section.push(0); // Null terminator
        //
        //             current_offset += filename_len as u32;
        //         }
        //     } else {
        //         header.textures = M2Array::new(0, 0);
        //     }
        //
        //     // Write materials (render flags)
        //     if !self.materials.is_empty() {
        //         header.materials = M2Array::new(self.materials.len() as u32, current_offset);
        //
        //         for material in &self.materials {
        //             let mut material_data = Vec::new();
        //             material.write(&mut material_data, self.header.version)?;
        //             data_section.extend_from_slice(&material_data);
        //         }
        //
        //         let material_size = match self.header.version().unwrap_or(M2Version::Classic) {
        //             v if v >= M2Version::WoD => 18, // WoD and later have color animation lookup
        //             v if v >= M2Version::Cataclysm => 16, // Cataclysm and later have shader ID and secondary texture unit
        //             _ => 12,                              // Classic to WotLK
        //         };
        //
        //         current_offset += (self.materials.len() * material_size) as u32;
        //     } else {
        //         header.materials = M2Array::new(0, 0);
        //     }
        //
        //     // Write bone lookup table
        //     if !self.raw_data.bone_lookup_table.is_empty() {
        //         header.bone_lookup_table =
        //             M2Array::new(self.raw_data.bone_lookup_table.len() as u32, current_offset);
        //
        //         for &lookup in &self.raw_data.bone_lookup_table {
        //             data_section.extend_from_slice(&lookup.to_le_bytes());
        //         }
        //
        //         current_offset +=
        //             (self.raw_data.bone_lookup_table.len() * std::mem::size_of::<u16>()) as u32;
        //     } else {
        //         header.bone_lookup_table = M2Array::new(0, 0);
        //     }
        //
        //     // Write texture lookup table
        //     if !self.raw_data.texture_mapping_lookup_table.is_empty() {
        //         header.texture_mapping_lookup_table = M2Array::new(
        //             self.raw_data.texture_mapping_lookup_table.len() as u32,
        //             current_offset,
        //         );
        //
        //         for &lookup in &self.raw_data.texture_mapping_lookup_table {
        //             data_section.extend_from_slice(&lookup.to_le_bytes());
        //         }
        //
        //         current_offset += (self.raw_data.texture_mapping_lookup_table.len()
        //             * std::mem::size_of::<u16>()) as u32;
        //     } else {
        //         header.texture_mapping_lookup_table = M2Array::new(0, 0);
        //     }
        //
        //     // Write texture units
        //     if !self.raw_data.texture_units.is_empty() {
        //         header.texture_units =
        //             M2Array::new(self.raw_data.texture_units.len() as u32, current_offset);
        //
        //         for &unit in &self.raw_data.texture_units {
        //             data_section.extend_from_slice(&unit.to_le_bytes());
        //         }
        //
        //         current_offset +=
        //             (self.raw_data.texture_units.len() * std::mem::size_of::<u16>()) as u32;
        //     } else {
        //         header.texture_units = M2Array::new(0, 0);
        //     }
        //
        //     // Write transparency lookup table
        //     if !self.raw_data.transparency_lookup_table.is_empty() {
        //         header.transparency_lookup_table = M2Array::new(
        //             self.raw_data.transparency_lookup_table.len() as u32,
        //             current_offset,
        //         );
        //
        //         for &lookup in &self.raw_data.transparency_lookup_table {
        //             data_section.extend_from_slice(&lookup.to_le_bytes());
        //         }
        //
        //         current_offset +=
        //             (self.raw_data.transparency_lookup_table.len() * std::mem::size_of::<u16>()) as u32;
        //     } else {
        //         header.transparency_lookup_table = M2Array::new(0, 0);
        //     }
        //
        //     // Write texture animation lookup
        //     if !self.raw_data.texture_animation_lookup.is_empty() {
        //         header.texture_animation_lookup = M2Array::new(
        //             self.raw_data.texture_animation_lookup.len() as u32,
        //             current_offset,
        //         );
        //
        //         for &lookup in &self.raw_data.texture_animation_lookup {
        //             data_section.extend_from_slice(&lookup.to_le_bytes());
        //         }
        //
        //         // current_offset +=
        //         //     (self.raw_data.texture_animation_lookup.len() * std::mem::size_of::<u16>()) as u32;
        //     } else {
        //         header.texture_animation_lookup = M2Array::new(0, 0);
        //     }
        //
        //     // Finally, write the header followed by the data section
        //     header.write(writer)?;
        //     writer.write_all(&data_section)?;
        //
        //     Ok(())
    }

    // /// Convert this model to a different version
    // pub fn convert(&self, target_version: M2Version) -> Result<Self> {
    //     let source_version = self.header.version().ok_or(M2Error::ConversionError {
    //         from: self.header.version,
    //         to: target_version.to_header_version(),
    //         reason: "Unknown source version".to_string(),
    //     })?;
    //
    //     if source_version == target_version {
    //         return Ok(self.clone());
    //     }
    //
    //     // Convert the header
    //     let header = self.header.convert(target_version)?;
    //
    //     // Convert vertices
    //     let vertices = self
    //         .vertices
    //         .iter()
    //         .map(|v| v.convert(target_version))
    //         .collect();
    //
    //     // Convert textures
    //     let textures = self
    //         .textures
    //         .iter()
    //         .map(|t| t.convert(target_version))
    //         .collect();
    //
    //     // Convert bones
    //     let bones = self
    //         .bones
    //         .iter()
    //         .map(|b| b.convert(target_version))
    //         .collect();
    //
    //     // Convert materials
    //     let materials = self
    //         .materials
    //         .iter()
    //         .map(|m| m.convert(target_version))
    //         .collect();
    //
    //     // Create the new model
    //     let mut new_model = self.clone();
    //     new_model.header = header;
    //     new_model.vertices = vertices;
    //     new_model.textures = textures;
    //     new_model.bones = bones;
    //     new_model.materials = materials;
    //
    //     Ok(new_model)
    // }

    // /// Validate the model structure
    // pub fn validate(&self) -> Result<()> {
    //     // Check if the version is supported
    //     if self.header.version().is_none() {
    //         return Err(M2Error::UnsupportedVersion(self.header.version.to_string()));
    //     }
    //
    //     // Validate vertices
    //     if self.vertices.is_empty() {
    //         return Err(M2Error::ValidationError(
    //             "Model has no vertices".to_string(),
    //         ));
    //     }
    //
    //     // Validate bones
    //     for (i, bone) in self.bones.iter().enumerate() {
    //         // Check if parent bone is valid
    //         if bone.parent_bone >= 0 && bone.parent_bone as usize >= self.bones.len() {
    //             return Err(M2Error::ValidationError(format!(
    //                 "Bone {} has invalid parent bone {}",
    //                 i, bone.parent_bone
    //             )));
    //         }
    //     }
    //
    //     // Validate textures
    //     for (i, texture) in self.textures.iter().enumerate() {
    //         // Check if the texture has a valid filename
    //         if texture.filename.array.count > 0 && texture.filename.array.offset == 0 {
    //             return Err(M2Error::ValidationError(format!(
    //                 "Texture {i} has invalid filename offset"
    //             )));
    //         }
    //     }
    //
    //     // Validate materials (simplified - just check basic structure)
    //     for (i, _material) in self.materials.iter().enumerate() {
    //         // Materials now only contain render flags and blend modes
    //         // No direct texture references to validate here
    //         let _material_index = i; // Just to acknowledge we're iterating
    //     }
    //
    //     Ok(())
    // }
}
