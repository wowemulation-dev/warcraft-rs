// writer.rs - Write ADT files to binary format

use std::io::{Seek, SeekFrom, Write};

use crate::Adt;
use crate::error::Result;
use crate::io_helpers::WriteLittleEndian;
use crate::mcnk_writer;
use crate::version::AdtVersion;

/// Write a chunk header to a writer
fn write_chunk_header<W: Write>(writer: &mut W, magic: &[u8; 4], size: u32) -> Result<()> {
    // WoW files store magic bytes in reverse order
    let mut reversed_magic = *magic;
    reversed_magic.reverse();
    writer.write_all(&reversed_magic)?;
    writer.write_u32_le(size)?;
    Ok(())
}

/// Structure to track offsets during writing
#[derive(Default, Debug)]
struct OffsetTracker {
    mver: Option<u32>,
    mhdr: Option<u32>,
    mcin: Option<u32>,
    mtex: Option<u32>,
    mmdx: Option<u32>,
    mmid: Option<u32>,
    mwmo: Option<u32>,
    mwid: Option<u32>,
    mddf: Option<u32>,
    modf: Option<u32>,
    mfbo: Option<u32>,
    mh2o: Option<u32>,
    mtfx: Option<u32>,
    mcnk: Vec<u32>,
}

impl Adt {
    /// Write the ADT to a writer
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<()> {
        let mut offset_tracker = OffsetTracker::default();

        // Write in a specific order according to the ADT format
        // MVER is always first
        self.write_mver(writer, &mut offset_tracker)?;

        // MHDR follows, but we'll need to come back to it after we know all the offsets
        // Write a placeholder for now
        let mhdr_pos = writer.stream_position()?;
        offset_tracker.mhdr = Some(mhdr_pos as u32);

        // Calculate the MHDR size based on version
        let mhdr_size = 36 // base size
            + if self.version >= AdtVersion::TBC { 4 } else { 0 }
            + if self.version >= AdtVersion::WotLK { 4 } else { 0 }
            + if self.version >= AdtVersion::Cataclysm { 4 } else { 0 };

        write_chunk_header(writer, b"MHDR", mhdr_size)?;
        writer.write_all(&vec![0; mhdr_size as usize])?; // Placeholder data

        // Write main chunks
        self.write_mcin(writer, &mut offset_tracker)?;
        self.write_mtex(writer, &mut offset_tracker)?;
        self.write_mmdx(writer, &mut offset_tracker)?;
        self.write_mmid(writer, &mut offset_tracker)?;
        self.write_mwmo(writer, &mut offset_tracker)?;
        self.write_mwid(writer, &mut offset_tracker)?;
        self.write_mddf(writer, &mut offset_tracker)?;
        self.write_modf(writer, &mut offset_tracker)?;

        // Write version-specific chunks
        if self.version >= AdtVersion::TBC {
            self.write_mfbo(writer, &mut offset_tracker)?;
        }

        if self.version >= AdtVersion::WotLK {
            self.write_mh2o(writer, &mut offset_tracker)?;
        }

        if self.version >= AdtVersion::Cataclysm {
            self.write_mtfx(writer, &mut offset_tracker)?;
        }

        // Write MCNK chunks (these come last)
        self.write_mcnks(writer, &mut offset_tracker)?;

        // Go back and write the actual MHDR with correct offsets
        writer.seek(SeekFrom::Start(mhdr_pos))?;
        self.write_mhdr(writer, &offset_tracker)?;

        Ok(())
    }

    /// Write MVER chunk
    fn write_mver<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        let pos = writer.stream_position()? as u32;
        offsets.mver = Some(pos);

        write_chunk_header(writer, b"MVER", 4)?;
        writer.write_u32_le(self.version.to_mver_value())?;

        Ok(())
    }

    /// Write MHDR chunk with correct offsets
    fn write_mhdr<W: Write + Seek>(&self, writer: &mut W, offsets: &OffsetTracker) -> Result<()> {
        // Calculate the size based on the version
        let base_size = 36; // 9 fields * 4 bytes
        let mut size = base_size;

        if self.version >= AdtVersion::TBC {
            size += 4; // MFBO offset
        }

        if self.version >= AdtVersion::WotLK {
            size += 4; // MH2O offset
        }

        if self.version >= AdtVersion::Cataclysm {
            size += 4; // MTFX offset
        }

        write_chunk_header(writer, b"MHDR", size)?;

        // Write flags and offsets
        let flags = self.mhdr.as_ref().map_or(0, |h| h.flags);
        writer.write_u32_le(flags)?;

        // Write offsets relative to the start of the file
        writer.write_u32_le(offsets.mcin.unwrap_or(0))?;
        writer.write_u32_le(offsets.mtex.unwrap_or(0))?;
        writer.write_u32_le(offsets.mmdx.unwrap_or(0))?;
        writer.write_u32_le(offsets.mmid.unwrap_or(0))?;
        writer.write_u32_le(offsets.mwmo.unwrap_or(0))?;
        writer.write_u32_le(offsets.mwid.unwrap_or(0))?;
        writer.write_u32_le(offsets.mddf.unwrap_or(0))?;
        writer.write_u32_le(offsets.modf.unwrap_or(0))?;

        // Version-specific offsets
        if self.version >= AdtVersion::TBC {
            writer.write_u32_le(offsets.mfbo.unwrap_or(0))?;
        }

        if self.version >= AdtVersion::WotLK {
            writer.write_u32_le(offsets.mh2o.unwrap_or(0))?;
        }

        if self.version >= AdtVersion::Cataclysm {
            writer.write_u32_le(offsets.mtfx.unwrap_or(0))?;
        }

        Ok(())
    }

    /// Write MCIN chunk
    fn write_mcin<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mcin) = self.mcin {
            let pos = writer.stream_position()? as u32;
            offsets.mcin = Some(pos);

            // MCIN is always 256 entries * 16 bytes = 4096 bytes
            write_chunk_header(writer, b"MCIN", 4096)?;

            for entry in &mcin.entries {
                writer.write_u32_le(entry.offset)?;
                writer.write_u32_le(entry.size)?;
                writer.write_u32_le(entry.flags)?;
                writer.write_u32_le(entry.layer_count)?;
            }
        }

        Ok(())
    }

    /// Write MTEX chunk
    fn write_mtex<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mtex) = self.mtex {
            let pos = writer.stream_position()? as u32;
            offsets.mtex = Some(pos);

            // Calculate size - all filenames plus null terminators
            let mut size = 0;
            for filename in &mtex.filenames {
                size += filename.len() + 1; // +1 for null terminator
            }

            write_chunk_header(writer, b"MTEX", size as u32)?;

            // Write filenames with null terminators
            for filename in &mtex.filenames {
                writer.write_all(filename.as_bytes())?;
                writer.write_u8(0)?; // Null terminator
            }
        }

        Ok(())
    }

    /// Write MMDX chunk
    fn write_mmdx<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mmdx) = self.mmdx {
            let pos = writer.stream_position()? as u32;
            offsets.mmdx = Some(pos);

            // Calculate size - all filenames plus null terminators
            let mut size = 0;
            for filename in &mmdx.filenames {
                size += filename.len() + 1; // +1 for null terminator
            }

            write_chunk_header(writer, b"MMDX", size as u32)?;

            // Write filenames with null terminators
            for filename in &mmdx.filenames {
                writer.write_all(filename.as_bytes())?;
                writer.write_u8(0)?; // Null terminator
            }
        }

        Ok(())
    }

    /// Write MMID chunk
    fn write_mmid<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mmid) = self.mmid {
            let pos = writer.stream_position()? as u32;
            offsets.mmid = Some(pos);

            // Size is number of offsets * 4 bytes
            let size = mmid.offsets.len() * 4;

            write_chunk_header(writer, b"MMID", size as u32)?;

            // Write offsets
            for offset in &mmid.offsets {
                writer.write_u32_le(*offset)?;
            }
        }

        Ok(())
    }

    /// Write MWMO chunk
    fn write_mwmo<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mwmo) = self.mwmo {
            let pos = writer.stream_position()? as u32;
            offsets.mwmo = Some(pos);

            // Calculate size - all filenames plus null terminators
            let mut size = 0;
            for filename in &mwmo.filenames {
                size += filename.len() + 1; // +1 for null terminator
            }

            write_chunk_header(writer, b"MWMO", size as u32)?;

            // Write filenames with null terminators
            for filename in &mwmo.filenames {
                writer.write_all(filename.as_bytes())?;
                writer.write_u8(0)?; // Null terminator
            }
        }

        Ok(())
    }

    /// Write MWID chunk
    fn write_mwid<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mwid) = self.mwid {
            let pos = writer.stream_position()? as u32;
            offsets.mwid = Some(pos);

            // Size is number of offsets * 4 bytes
            let size = mwid.offsets.len() * 4;

            write_chunk_header(writer, b"MWID", size as u32)?;

            // Write offsets
            for offset in &mwid.offsets {
                writer.write_u32_le(*offset)?;
            }
        }

        Ok(())
    }

    /// Write MDDF chunk
    fn write_mddf<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mddf) = self.mddf {
            let pos = writer.stream_position()? as u32;
            offsets.mddf = Some(pos);

            // Size is number of doodads * 36 bytes per doodad
            let size = mddf.doodads.len() * 36;

            write_chunk_header(writer, b"MDDF", size as u32)?;

            // Write doodad placements
            for doodad in &mddf.doodads {
                writer.write_u32_le(doodad.name_id)?;
                writer.write_u32_le(doodad.unique_id)?;

                for i in 0..3 {
                    writer.write_f32_le(doodad.position[i])?;
                }

                for i in 0..3 {
                    writer.write_f32_le(doodad.rotation[i])?;
                }

                writer.write_f32_le(doodad.scale)?;
                writer.write_u16_le(doodad.flags)?;

                // Write padding (2 bytes)
                writer.write_all(&[0, 0])?;
            }
        }

        Ok(())
    }

    /// Write MODF chunk
    fn write_modf<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref modf) = self.modf {
            let pos = writer.stream_position()? as u32;
            offsets.modf = Some(pos);

            // Size is number of models * 64 bytes per model
            let size = modf.models.len() * 64;

            write_chunk_header(writer, b"MODF", size as u32)?;

            // Write model placements
            for model in &modf.models {
                writer.write_u32_le(model.name_id)?;
                writer.write_u32_le(model.unique_id)?;

                for i in 0..3 {
                    writer.write_f32_le(model.position[i])?;
                }

                for i in 0..3 {
                    writer.write_f32_le(model.rotation[i])?;
                }

                for i in 0..3 {
                    writer.write_f32_le(model.bounds_min[i])?;
                }

                for i in 0..3 {
                    writer.write_f32_le(model.bounds_max[i])?;
                }

                writer.write_u16_le(model.flags)?;
                writer.write_u16_le(model.doodad_set)?;
                writer.write_u16_le(model.name_set)?;
                writer.write_u16_le(model.padding)?;
            }
        }

        Ok(())
    }

    /// Write MFBO chunk (TBC+)
    fn write_mfbo<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mfbo) = self.mfbo {
            let pos = writer.stream_position()? as u32;
            offsets.mfbo = Some(pos);

            // MFBO is always 8 bytes
            write_chunk_header(writer, b"MFBO", 8)?;

            // Write boundaries
            for i in 0..2 {
                writer.write_u16_le(mfbo.max[i])?;
            }

            for i in 0..2 {
                writer.write_u16_le(mfbo.min[i])?;
            }
        }

        Ok(())
    }

    /// Write MH2O chunk (WotLK+)
    fn write_mh2o<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref _mh2o) = self.mh2o {
            let pos = writer.stream_position()? as u32;
            offsets.mh2o = Some(pos);

            // MH2O headers are 256 entries * 24 bytes = 6144 bytes
            // Plus variable amount of data
            let header_size = 256 * 24;

            // For simplicity, we'll just write empty headers for now
            // A real implementation would need to calculate all the offsets
            write_chunk_header(writer, b"MH2O", header_size as u32)?;

            // Write empty headers
            for _ in 0..256 {
                writer.write_all(&[0; 24])?;
            }
        }

        Ok(())
    }

    /// Write MTFX chunk (Cataclysm+)
    fn write_mtfx<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        if let Some(ref mtfx) = self.mtfx {
            let pos = writer.stream_position()? as u32;
            offsets.mtfx = Some(pos);

            // Size is number of effects * 4 bytes per effect
            let size = mtfx.effects.len() * 4;

            write_chunk_header(writer, b"MTFX", size as u32)?;

            // Write effect IDs
            for effect in &mtfx.effects {
                writer.write_u32_le(effect.effect_id)?;
            }
        }

        Ok(())
    }

    /// Write MCNK chunks
    fn write_mcnks<W: Write + Seek>(
        &self,
        writer: &mut W,
        offsets: &mut OffsetTracker,
    ) -> Result<()> {
        offsets.mcnk.clear();

        // Store MCNK positions for MCIN
        let mut mcnk_entries = Vec::new();

        for mcnk in &self.mcnk_chunks {
            // Use the proper mcnk_writer to write the full MCNK chunk
            let (pos, size) = mcnk_writer::write_mcnk(writer, mcnk, self.version)?;

            offsets.mcnk.push(pos);
            mcnk_entries.push((pos, size));
        }

        // Update MCIN entries if needed
        if self.mcin.is_some() && offsets.mcin.is_some() {
            let mcin_pos = offsets.mcin.unwrap();
            let current_pos = writer.stream_position()?;

            // Go back to MCIN and update entries
            writer.seek(SeekFrom::Start((mcin_pos + 8) as u64))?;

            for (i, (pos, size)) in mcnk_entries.iter().enumerate() {
                if i < 256 {
                    // MCIN always has 256 entries
                    writer.write_u32_le(*pos)?;
                    writer.write_u32_le(*size)?;
                    writer.write_u32_le(0)?; // flags
                    writer.write_u32_le(0)?; // layer_count
                }
            }

            // Return to the end
            writer.seek(SeekFrom::Start(current_pos))?;
        }

        Ok(())
    }
}
