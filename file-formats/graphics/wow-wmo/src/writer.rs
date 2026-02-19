use std::io::{Seek, SeekFrom, Write};

use crate::chunk::ChunkHeader;
use crate::error::Result;
use crate::parser::chunks;
use crate::types::{Color, Vec3};
use crate::version::{WmoFeature, WmoVersion};
use crate::wmo_group_types::{TexCoord, WmoBatch, WmoBspNode, WmoGroup, WmoLiquid};
use crate::wmo_types::{
    WmoDoodadDef, WmoDoodadSet, WmoFlags, WmoGroupInfo, WmoLight, WmoMaterial, WmoPortal,
    WmoPortalReference, WmoRoot,
};

/// Helper trait for writing little-endian values
#[allow(dead_code)]
trait WriteLittleEndian: Write {
    fn write_u8(&mut self, val: u8) -> Result<()> {
        self.write_all(&[val])?;
        Ok(())
    }

    fn write_u16_le(&mut self, val: u16) -> Result<()> {
        self.write_all(&val.to_le_bytes())?;
        Ok(())
    }

    fn write_u32_le(&mut self, val: u32) -> Result<()> {
        self.write_all(&val.to_le_bytes())?;
        Ok(())
    }

    fn write_i16_le(&mut self, val: i16) -> Result<()> {
        self.write_all(&val.to_le_bytes())?;
        Ok(())
    }

    fn write_i32_le(&mut self, val: i32) -> Result<()> {
        self.write_all(&val.to_le_bytes())?;
        Ok(())
    }

    fn write_f32_le(&mut self, val: f32) -> Result<()> {
        self.write_all(&val.to_le_bytes())?;
        Ok(())
    }
}

impl<W: Write> WriteLittleEndian for W {}

/// Writer for WMO files
pub struct WmoWriter;

impl Default for WmoWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl WmoWriter {
    /// Create a new WMO writer
    pub fn new() -> Self {
        Self
    }

    /// Write a WMO root file
    pub fn write_root<W: Write + Seek>(
        &self,
        writer: &mut W,
        wmo: &WmoRoot,
        target_version: WmoVersion,
    ) -> Result<()> {
        // Write version chunk
        self.write_version(writer, target_version)?;

        // Write header chunk
        self.write_header(writer, wmo, target_version)?;

        // Write textures
        self.write_textures(writer, &wmo.textures)?;

        // Write materials
        self.write_materials(writer, &wmo.materials, target_version)?;

        // Write group names
        self.write_group_names(writer, &wmo.groups)?;

        // Write group info
        self.write_group_info(writer, &wmo.groups, target_version)?;

        // Write skybox if applicable
        if target_version.supports_feature(WmoFeature::SkyboxReferences) && wmo.skybox.is_some() {
            self.write_skybox(writer, wmo.skybox.as_deref())?;
        }

        // Write portals
        self.write_portals(writer, &wmo.portals)?;

        // Write portal references
        self.write_portal_references(writer, &wmo.portal_references)?;

        // Write visible block lists
        self.write_visible_block_lists(writer, &wmo.visible_block_lists)?;

        // Write lights
        self.write_lights(writer, &wmo.lights, target_version)?;

        // Write doodad definitions and sets
        self.write_doodad_definitions(writer, &wmo.doodad_defs, target_version)?;
        self.write_doodad_sets(writer, &wmo.doodad_sets)?;

        Ok(())
    }

    /// Write a WMO group file
    pub fn write_group<W: Write + Seek>(
        &self,
        writer: &mut W,
        group: &WmoGroup,
        target_version: WmoVersion,
    ) -> Result<()> {
        // Write version chunk
        self.write_version(writer, target_version)?;

        // Start MOGP chunk (we'll need to update its size at the end)
        let mogp_pos = writer.stream_position()?;
        let mogp_header = ChunkHeader {
            id: chunks::MOGP,
            size: 0, // Placeholder, will update later
        };
        mogp_header.write(writer)?;

        // Write group header fields
        writer.write_u32_le(group.header.name_offset)?;
        writer.write_u32_le(group.header.flags.bits())?;

        // Write bounding box
        writer.write_f32_le(group.header.bounding_box.min.x)?;
        writer.write_f32_le(group.header.bounding_box.min.y)?;
        writer.write_f32_le(group.header.bounding_box.min.z)?;

        writer.write_f32_le(group.header.bounding_box.max.x)?;
        writer.write_f32_le(group.header.bounding_box.max.y)?;
        writer.write_f32_le(group.header.bounding_box.max.z)?;

        // Write flags and index
        writer.write_u16_le(0)?; // Flags2, only used in later versions
        writer.write_u16_le(group.header.group_index as u16)?;

        // Mark the start of subchunks
        let _subchunks_start = writer.stream_position()?;

        // Write vertices
        if !group.vertices.is_empty() {
            self.write_vertices(writer, &group.vertices)?;
        }

        // Write indices
        if !group.indices.is_empty() {
            self.write_indices(writer, &group.indices)?;
        }

        // Write normals if available
        if !group.normals.is_empty() {
            self.write_normals(writer, &group.normals)?;
        }

        // Write texture coordinates
        if !group.tex_coords.is_empty() {
            self.write_texture_coords(writer, &group.tex_coords)?;
        }

        // Write vertex colors if available
        if let Some(colors) = &group.vertex_colors
            && !colors.is_empty()
        {
            self.write_vertex_colors(writer, colors)?;
        }

        // Write batches
        if !group.batches.is_empty() {
            self.write_batches(writer, &group.batches)?;
        }

        // Write BSP nodes if available
        if let Some(nodes) = &group.bsp_nodes
            && !nodes.is_empty()
        {
            self.write_bsp_nodes(writer, nodes)?;
        }

        // Write liquid data if available
        if let Some(liquid) = &group.liquid {
            self.write_liquid(writer, liquid, target_version)?;
        }

        // Write doodad references if available
        if let Some(refs) = &group.doodad_refs
            && !refs.is_empty()
        {
            self.write_doodad_refs(writer, refs)?;
        }

        // Update MOGP chunk size
        let end_pos = writer.stream_position()?;
        let mogp_size = end_pos - mogp_pos - 8; // Subtract header size

        writer.seek(SeekFrom::Start(mogp_pos + 4))?; // Position at size field
        writer.write_u32_le(mogp_size as u32)?;

        // Return to end
        writer.seek(SeekFrom::Start(end_pos))?;

        Ok(())
    }

    /// Write version chunk
    fn write_version<W: Write>(&self, writer: &mut W, version: WmoVersion) -> Result<()> {
        let header = ChunkHeader {
            id: chunks::MVER,
            size: 4,
        };

        header.write(writer)?;
        writer.write_u32_le(version.to_raw())?;

        Ok(())
    }

    /// Write header chunk
    fn write_header<W: Write>(
        &self,
        writer: &mut W,
        wmo: &WmoRoot,
        target_version: WmoVersion,
    ) -> Result<()> {
        let header = ChunkHeader {
            id: chunks::MOHD,
            size: 60, // Fixed size for header (without padding)
        };

        header.write(writer)?;

        // Basic counts
        writer.write_u32_le(wmo.materials.len() as u32)?;
        writer.write_u32_le(wmo.groups.len() as u32)?;
        writer.write_u32_le(wmo.portals.len() as u32)?;
        writer.write_u32_le(wmo.lights.len() as u32)?;
        writer.write_u32_le(wmo.doodad_defs.len() as u32)?;
        writer.write_u32_le(wmo.doodad_defs.len() as u32)?; // n_doodad_names is same as defs
        writer.write_u32_le(wmo.doodad_sets.len() as u32)?;

        // Ambient color
        let color_bytes = (wmo.header.ambient_color.r as u32) << 16
            | (wmo.header.ambient_color.g as u32) << 8
            | (wmo.header.ambient_color.b as u32)
            | (wmo.header.ambient_color.a as u32) << 24;

        writer.write_u32_le(color_bytes)?;

        // Flags - adjust for version differences
        let mut flags = wmo.header.flags;

        // Add/remove flags based on version requirements
        if target_version.supports_feature(WmoFeature::SkyboxReferences) && wmo.skybox.is_some() {
            flags |= WmoFlags::HAS_SKYBOX;
        } else {
            flags &= !WmoFlags::HAS_SKYBOX;
        }

        writer.write_u32_le(flags.bits())?;

        // Bounding box
        writer.write_f32_le(wmo.bounding_box.min.x)?;
        writer.write_f32_le(wmo.bounding_box.min.y)?;
        writer.write_f32_le(wmo.bounding_box.min.z)?;

        writer.write_f32_le(wmo.bounding_box.max.x)?;
        writer.write_f32_le(wmo.bounding_box.max.y)?;
        writer.write_f32_le(wmo.bounding_box.max.z)?;

        Ok(())
    }

    /// Write textures
    fn write_textures<W: Write>(&self, writer: &mut W, textures: &[String]) -> Result<()> {
        if textures.is_empty() {
            return Ok(());
        }

        // Calculate the size of the MOTX chunk
        let mut total_size = 0;
        for texture in textures {
            total_size += texture.len() + 1; // +1 for null terminator
        }

        let header = ChunkHeader {
            id: chunks::MOTX,
            size: total_size as u32,
        };

        header.write(writer)?;

        // Write null-terminated strings
        for texture in textures {
            writer.write_all(texture.as_bytes())?;
            writer.write_u8(0)?; // Null terminator
        }

        Ok(())
    }

    /// Write materials
    fn write_materials<W: Write>(
        &self,
        writer: &mut W,
        materials: &[WmoMaterial],
        target_version: WmoVersion,
    ) -> Result<()> {
        if materials.is_empty() {
            return Ok(());
        }

        // Determine material size based on version
        let material_size = if target_version >= WmoVersion::Mop {
            64
        } else {
            40
        };

        let header = ChunkHeader {
            id: chunks::MOMT,
            size: (materials.len() * material_size) as u32,
        };

        header.write(writer)?;

        for material in materials {
            writer.write_u32_le(material.flags.bits())?;
            writer.write_u32_le(material.shader)?;
            writer.write_u32_le(material.blend_mode)?;
            writer.write_u32_le(material.texture1)?;

            writer.write_u8(material.emissive_color.r)?;
            writer.write_u8(material.emissive_color.g)?;
            writer.write_u8(material.emissive_color.b)?;
            writer.write_u8(material.emissive_color.a)?;

            writer.write_u8(material.sidn_color.r)?;
            writer.write_u8(material.sidn_color.g)?;
            writer.write_u8(material.sidn_color.b)?;
            writer.write_u8(material.sidn_color.a)?;

            writer.write_u32_le(material.texture2)?;

            writer.write_u8(material.diffuse_color.r)?;
            writer.write_u8(material.diffuse_color.g)?;
            writer.write_u8(material.diffuse_color.b)?;
            writer.write_u8(material.diffuse_color.a)?;

            writer.write_u32_le(material.ground_type)?;

            // Write 28 bytes of padding
            for _ in 0..28 {
                writer.write_u8(0)?;
            }
        }

        Ok(())
    }

    /// Write group names
    fn write_group_names<W: Write>(&self, writer: &mut W, groups: &[WmoGroupInfo]) -> Result<()> {
        if groups.is_empty() {
            return Ok(());
        }

        // Calculate the size of the MOGN chunk
        let mut total_size = 0;
        for group in groups {
            total_size += group.name.len() + 1; // +1 for null terminator
        }

        let header = ChunkHeader {
            id: chunks::MOGN,
            size: total_size as u32,
        };

        header.write(writer)?;

        // Write null-terminated strings
        for group in groups {
            writer.write_all(group.name.as_bytes())?;
            writer.write_u8(0)?; // Null terminator
        }

        Ok(())
    }

    /// Write group info
    fn write_group_info<W: Write>(
        &self,
        writer: &mut W,
        groups: &[WmoGroupInfo],
        _target_version: WmoVersion,
    ) -> Result<()> {
        if groups.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOGI,
            size: (groups.len() * 32) as u32, // 32 bytes per group
        };

        header.write(writer)?;

        for group in groups {
            writer.write_u32_le(group.flags.bits())?;

            writer.write_f32_le(group.bounding_box.min.x)?;
            writer.write_f32_le(group.bounding_box.min.y)?;
            writer.write_f32_le(group.bounding_box.min.z)?;

            writer.write_f32_le(group.bounding_box.max.x)?;
            writer.write_f32_le(group.bounding_box.max.y)?;
            writer.write_f32_le(group.bounding_box.max.z)?;

            // Write name offset in MOGN chunk
            // This is a simplification - in a real implementation, you'd need to calculate actual offsets
            writer.write_u32_le(0)?; // Placeholder
        }

        Ok(())
    }

    /// Write skybox
    fn write_skybox<W: Write>(&self, writer: &mut W, skybox: Option<&str>) -> Result<()> {
        if let Some(skybox) = skybox {
            let size = skybox.len() + 1; // +1 for null terminator

            let header = ChunkHeader {
                id: chunks::MOSB,
                size: size as u32,
            };

            header.write(writer)?;

            writer.write_all(skybox.as_bytes())?;
            writer.write_u8(0)?; // Null terminator
        }

        Ok(())
    }

    /// Write portals
    fn write_portals<W: Write>(&self, writer: &mut W, portals: &[WmoPortal]) -> Result<()> {
        if portals.is_empty() {
            return Ok(());
        }

        // First write portal vertices (MOPV)
        let mut all_vertices = Vec::new();

        for portal in portals {
            all_vertices.extend_from_slice(&portal.vertices);
        }

        let mopv_header = ChunkHeader {
            id: chunks::MOPV,
            size: (all_vertices.len() * 12) as u32, // 12 bytes per vertex (3 floats)
        };

        mopv_header.write(writer)?;

        for vertex in &all_vertices {
            writer.write_f32_le(vertex.x)?;
            writer.write_f32_le(vertex.y)?;
            writer.write_f32_le(vertex.z)?;
        }

        // Now write portal info (MOPT)
        let mopt_header = ChunkHeader {
            id: chunks::MOPT,
            size: (portals.len() * 20) as u32, // 20 bytes per portal
        };

        mopt_header.write(writer)?;

        let mut vertex_index = 0;

        for portal in portals {
            writer.write_u16_le(vertex_index as u16)?;
            writer.write_u16_le(portal.vertices.len() as u16)?;

            writer.write_f32_le(portal.normal.x)?;
            writer.write_f32_le(portal.normal.y)?;
            writer.write_f32_le(portal.normal.z)?;

            // Plane distance (dot product of normal and any vertex on the plane)
            let distance = if !portal.vertices.is_empty() {
                portal.normal.x * portal.vertices[0].x
                    + portal.normal.y * portal.vertices[0].y
                    + portal.normal.z * portal.vertices[0].z
            } else {
                0.0
            };

            writer.write_f32_le(distance)?;

            vertex_index += portal.vertices.len();
        }

        Ok(())
    }

    /// Write portal references
    fn write_portal_references<W: Write>(
        &self,
        writer: &mut W,
        refs: &[WmoPortalReference],
    ) -> Result<()> {
        if refs.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOPR,
            size: (refs.len() * 8) as u32, // 8 bytes per reference
        };

        header.write(writer)?;

        for r in refs {
            writer.write_u16_le(r.portal_index)?;
            writer.write_u16_le(r.group_index)?;
            writer.write_u16_le(r.side)?;
            writer.write_u16_le(0)?; // Padding
        }

        Ok(())
    }

    /// Write visible block lists
    fn write_visible_block_lists<W: Write>(
        &self,
        writer: &mut W,
        lists: &[Vec<u16>],
    ) -> Result<()> {
        if lists.is_empty() {
            return Ok(());
        }

        // First calculate the offset table (MOVV)
        let mut offsets = Vec::with_capacity(lists.len());
        let mut current_offset = 0;

        for list in lists {
            offsets.push(current_offset);
            current_offset += (list.len() + 1) * 2; // +1 for the 0xFFFF terminator, *2 for u16 size
        }

        // Write offset table (MOVV)
        let movv_header = ChunkHeader {
            id: chunks::MOVV,
            size: (offsets.len() * 4) as u32, // 4 bytes per offset (u32)
        };

        movv_header.write(writer)?;

        for offset in &offsets {
            writer.write_u32_le(*offset as u32)?;
        }

        // Calculate the total size of visible blocks data
        let total_block_size = lists.iter().map(|list| (list.len() + 1) * 2).sum::<usize>();

        // Write visible blocks (MOVB)
        let movb_header = ChunkHeader {
            id: chunks::MOVB,
            size: total_block_size as u32,
        };

        movb_header.write(writer)?;

        for list in lists {
            for &index in list {
                writer.write_u16_le(index)?;
            }

            writer.write_u16_le(0xFFFF)?; // End of list marker
        }

        Ok(())
    }

    /// Write lights
    fn write_lights<W: Write>(
        &self,
        writer: &mut W,
        lights: &[WmoLight],
        _target_version: WmoVersion,
    ) -> Result<()> {
        if lights.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOLT,
            size: (lights.len() * 48) as u32, // 48 bytes per light
        };

        header.write(writer)?;

        for light in lights {
            // +0x00: type (u8)
            writer.write_u8(light.light_type as u8)?;
            // +0x01: use_attenuation (u8)
            writer.write_u8(if light.use_attenuation { 1 } else { 0 })?;
            // +0x02: padding (u8[2])
            writer.write_u8(0)?;
            writer.write_u8(0)?;

            // +0x04: color (CImVector, BGRA)
            let color_bytes = (light.color.b as u32)
                | (light.color.g as u32) << 8
                | (light.color.r as u32) << 16
                | (light.color.a as u32) << 24;
            writer.write_u32_le(color_bytes)?;

            // +0x08: position (C3Vector)
            writer.write_f32_le(light.position.x)?;
            writer.write_f32_le(light.position.y)?;
            writer.write_f32_le(light.position.z)?;

            // +0x14: intensity
            writer.write_f32_le(light.intensity)?;

            // +0x18: rotation (C4Quaternion)
            for &r in &light.rotation {
                writer.write_f32_le(r)?;
            }

            // +0x28: attenuation start
            writer.write_f32_le(light.attenuation_start)?;
            // +0x2C: attenuation end
            writer.write_f32_le(light.attenuation_end)?;
        }

        Ok(())
    }

    /// Write doodad definitions
    fn write_doodad_definitions<W: Write>(
        &self,
        writer: &mut W,
        doodads: &[WmoDoodadDef],
        _target_version: WmoVersion,
    ) -> Result<()> {
        if doodads.is_empty() {
            return Ok(());
        }

        // Calculate the size of the MODN chunk (doodad names)
        let mut doodad_names = Vec::new();
        let mut name_offsets = Vec::new();
        let mut current_offset = 0;

        for doodad in doodads {
            // This is a simplification - in a real implementation, you'd need to extract names from doodad_names
            let name = format!("doodad_{}", doodad.name_offset);
            name_offsets.push(current_offset);

            current_offset += name.len() + 1; // +1 for null terminator
            doodad_names.push(name);
        }

        // Write doodad names (MODN)
        let modn_header = ChunkHeader {
            id: chunks::MODN,
            size: current_offset as u32,
        };

        modn_header.write(writer)?;

        for name in &doodad_names {
            writer.write_all(name.as_bytes())?;
            writer.write_u8(0)?; // Null terminator
        }

        // All versions use 40 bytes for doodad definitions
        let doodad_size = 40;

        // Write doodad definitions (MODD)
        let modd_header = ChunkHeader {
            id: chunks::MODD,
            size: (doodads.len() * doodad_size) as u32,
        };

        modd_header.write(writer)?;

        for (i, doodad) in doodads.iter().enumerate() {
            writer.write_u32_le(name_offsets[i] as u32)?;

            writer.write_f32_le(doodad.position.x)?;
            writer.write_f32_le(doodad.position.y)?;
            writer.write_f32_le(doodad.position.z)?;

            writer.write_f32_le(doodad.orientation[0])?;
            writer.write_f32_le(doodad.orientation[1])?;
            writer.write_f32_le(doodad.orientation[2])?;
            writer.write_f32_le(doodad.orientation[3])?;

            writer.write_f32_le(doodad.scale)?;

            // Color
            let color_bytes = (doodad.color.r as u32) << 16
                | (doodad.color.g as u32) << 8
                | (doodad.color.b as u32)
                | (doodad.color.a as u32) << 24;

            writer.write_u32_le(color_bytes)?;
        }

        Ok(())
    }

    /// Write doodad sets
    fn write_doodad_sets<W: Write>(&self, writer: &mut W, sets: &[WmoDoodadSet]) -> Result<()> {
        if sets.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MODS,
            size: (sets.len() * 32) as u32, // 32 bytes per set
        };

        header.write(writer)?;

        for set in sets {
            // Write name (20 bytes, null padded)
            let mut name_bytes = [0u8; 20];
            for (i, &b) in set.name.as_bytes().iter().enumerate() {
                if i < 19 {
                    // Ensure space for null terminator
                    name_bytes[i] = b;
                }
            }

            writer.write_all(&name_bytes)?;

            writer.write_u32_le(set.start_doodad)?;
            writer.write_u32_le(set.n_doodads)?;
            writer.write_u32_le(0)?; // Unused
        }

        Ok(())
    }

    // Group file writing methods

    /// Write vertices
    fn write_vertices<W: Write>(&self, writer: &mut W, vertices: &[Vec3]) -> Result<()> {
        if vertices.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOVT,
            size: (vertices.len() * 12) as u32, // 12 bytes per vertex (3 floats)
        };

        header.write(writer)?;

        for vertex in vertices {
            writer.write_f32_le(vertex.x)?;
            writer.write_f32_le(vertex.y)?;
            writer.write_f32_le(vertex.z)?;
        }

        Ok(())
    }

    /// Write indices
    fn write_indices<W: Write>(&self, writer: &mut W, indices: &[u16]) -> Result<()> {
        if indices.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOVI,
            size: (indices.len() * 2) as u32, // 2 bytes per index (u16)
        };

        header.write(writer)?;

        for &index in indices {
            writer.write_u16_le(index)?;
        }

        Ok(())
    }

    /// Write normals
    fn write_normals<W: Write>(&self, writer: &mut W, normals: &[Vec3]) -> Result<()> {
        if normals.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MONR,
            size: (normals.len() * 12) as u32, // 12 bytes per normal (3 floats)
        };

        header.write(writer)?;

        for normal in normals {
            writer.write_f32_le(normal.x)?;
            writer.write_f32_le(normal.y)?;
            writer.write_f32_le(normal.z)?;
        }

        Ok(())
    }

    /// Write texture coordinates
    fn write_texture_coords<W: Write>(
        &self,
        writer: &mut W,
        tex_coords: &[TexCoord],
    ) -> Result<()> {
        if tex_coords.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOTV,
            size: (tex_coords.len() * 8) as u32, // 8 bytes per tex coord (2 floats)
        };

        header.write(writer)?;

        for tex_coord in tex_coords {
            writer.write_f32_le(tex_coord.u)?;
            writer.write_f32_le(tex_coord.v)?;
        }

        Ok(())
    }

    /// Write vertex colors
    fn write_vertex_colors<W: Write>(&self, writer: &mut W, colors: &[Color]) -> Result<()> {
        if colors.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOCV,
            size: (colors.len() * 4) as u32, // 4 bytes per color (BGRA)
        };

        header.write(writer)?;

        for color in colors {
            writer.write_u8(color.b)?;
            writer.write_u8(color.g)?;
            writer.write_u8(color.r)?;
            writer.write_u8(color.a)?;
        }

        Ok(())
    }

    /// Write batches
    fn write_batches<W: Write>(&self, writer: &mut W, batches: &[WmoBatch]) -> Result<()> {
        if batches.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOBA,
            size: (batches.len() * 24) as u32, // 24 bytes per batch
        };

        header.write(writer)?;

        for batch in batches {
            writer.write_all(&batch.flags)?;

            writer.write_u16_le(batch.material_id)?;
            writer.write_u32_le(batch.start_index)?;
            writer.write_u16_le(batch.count)?;

            writer.write_u16_le(batch.start_vertex)?;
            writer.write_u16_le(batch.end_vertex)?;

            writer.write_u8(batch.use_large_material_id as u8)?;
            writer.write_u8(batch.material_id as u8)?;
        }

        Ok(())
    }

    /// Write BSP nodes
    fn write_bsp_nodes<W: Write>(&self, writer: &mut W, nodes: &[WmoBspNode]) -> Result<()> {
        if nodes.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MOBN,
            size: (nodes.len() * 16) as u32, // 16 bytes per node
        };

        header.write(writer)?;

        for node in nodes {
            // Write plane normal and flags packed into first float
            let plane_flags;
            let plane_normal_x;

            // Encode the normal into the first float and flags
            if node.plane.normal.x.abs() > 0.999 {
                plane_flags = 0; // X axis
                plane_normal_x = f32::from_bits(plane_flags);
            } else if node.plane.normal.y.abs() > 0.999 {
                plane_flags = 1; // Y axis
                plane_normal_x = f32::from_bits(plane_flags);
            } else if node.plane.normal.z.abs() > 0.999 {
                plane_flags = 2; // Z axis
                plane_normal_x = f32::from_bits(plane_flags);
            } else {
                plane_flags = 3; // Custom normal

                // Encode x component into the upper 30 bits
                let x_encoded = (node.plane.normal.x * 32767.0) as i32;
                plane_normal_x = f32::from_bits((x_encoded << 2 | plane_flags as i32) as u32);
            }

            if plane_flags < 3 {
                writer.write_u32_le(plane_flags)?;
            } else {
                writer.write_f32_le(plane_normal_x)?;
            }

            writer.write_f32_le(node.plane.distance)?;

            writer.write_i16_le(node.children[0])?;
            writer.write_i16_le(node.children[1])?;

            writer.write_u16_le(node.first_face)?;
            writer.write_u16_le(node.num_faces)?;
        }

        Ok(())
    }

    /// Write liquid data
    fn write_liquid<W: Write>(
        &self,
        writer: &mut W,
        liquid: &WmoLiquid,
        target_version: WmoVersion,
    ) -> Result<()> {
        // Calculate size based on version and content
        let vertex_size = if target_version >= WmoVersion::Wod {
            16
        } else {
            4
        }; // 16 bytes for new format, 4 for old
        let vertices_size = (liquid.width * liquid.height) as usize * vertex_size;

        let has_tile_flags = liquid.tile_flags.is_some();
        let tile_flags_size = if has_tile_flags {
            ((liquid.width - 1) * (liquid.height - 1)) as usize
        } else {
            0
        };

        let total_size = 32 + vertices_size + tile_flags_size; // 32 bytes for header

        let header = ChunkHeader {
            id: chunks::MLIQ,
            size: total_size as u32,
        };

        header.write(writer)?;

        // Write liquid header
        writer.write_u32_le(liquid.liquid_type)?;
        writer.write_u32_le(liquid.flags)?;

        // Width/height are stored as width-1 and height-1
        writer.write_u32_le(liquid.width - 1)?;
        writer.write_u32_le(liquid.height - 1)?;

        // Write bounding box (calculated from vertices)
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;

        for vertex in &liquid.vertices {
            min_x = min_x.min(vertex.position.x);
            min_y = min_y.min(vertex.position.y);
            min_z = min_z.min(vertex.position.z);

            max_x = max_x.max(vertex.position.x);
            max_y = max_y.max(vertex.position.y);
            max_z = max_z.max(vertex.position.z + vertex.height);
        }

        writer.write_f32_le(min_x)?;
        writer.write_f32_le(min_y)?;
        writer.write_f32_le(min_z)?;

        writer.write_f32_le(max_x)?;
        writer.write_f32_le(max_y)?;
        writer.write_f32_le(max_z)?;

        // Write vertices based on format
        if target_version >= WmoVersion::Wod {
            // New format in WoD+ with base position and height
            for vertex in &liquid.vertices {
                writer.write_f32_le(vertex.position.x)?;
                writer.write_f32_le(vertex.position.y)?;
                writer.write_f32_le(vertex.position.z)?;
                writer.write_f32_le(vertex.height)?;
            }
        } else {
            // Old format with just heights
            for vertex in &liquid.vertices {
                writer.write_f32_le(vertex.height)?;
            }
        }

        // Write tile flags if present
        if let Some(flags) = &liquid.tile_flags {
            for &flag in flags {
                writer.write_u8(flag)?;
            }
        }

        Ok(())
    }

    /// Write doodad references
    fn write_doodad_refs<W: Write>(&self, writer: &mut W, doodad_refs: &[u16]) -> Result<()> {
        if doodad_refs.is_empty() {
            return Ok(());
        }

        let header = ChunkHeader {
            id: chunks::MODR,
            size: (doodad_refs.len() * 2) as u32, // 2 bytes per reference (u16)
        };

        header.write(writer)?;

        for &doodad_ref in doodad_refs {
            writer.write_u16_le(doodad_ref)?;
        }

        Ok(())
    }
}
