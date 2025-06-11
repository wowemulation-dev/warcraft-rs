// memory_opt.rs - Memory optimization for ADT parsing

use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::rc::Rc;

use crate::Adt;
use crate::chunk::*;
use crate::error::Result;
use crate::streaming::AdtStreamer;

/// Memory usage configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum memory buffer size for streaming operations
    pub max_buffer_size: usize,
    /// Whether to use zero-copy parsing when possible
    pub use_zero_copy: bool,
    /// Whether to use memory pooling for repeated structures
    pub use_memory_pool: bool,
    /// Whether to use compact data structures
    pub use_compact_structures: bool,
    /// Maximum number of chunks to keep in memory at once
    pub max_chunks_in_memory: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 1024 * 1024, // 1 MB
            use_zero_copy: true,
            use_memory_pool: true,
            use_compact_structures: true,
            max_chunks_in_memory: 64, // Keep at most 64 chunks in memory
        }
    }
}

/// Memory-optimized ADT parser
pub struct OptimizedAdtParser<R: Read + Seek> {
    /// The streamer for reading chunks
    streamer: AdtStreamer<R>,
    /// Memory configuration
    config: MemoryConfig,
    /// Pool of reusable byte buffers
    buffer_pool: Vec<Vec<u8>>,
    /// Loaded MCNK chunks
    loaded_chunks: Vec<(u32, u32, McnkChunk)>, // (x, y, chunk)
    /// Whether the parser has been initialized
    initialized: bool,
}

impl<R: Read + Seek> OptimizedAdtParser<R> {
    /// Create a new optimized ADT parser
    pub fn new(reader: R, config: MemoryConfig) -> Result<Self> {
        let streamer = AdtStreamer::new(reader)?;

        Ok(Self {
            streamer,
            config,
            buffer_pool: Vec::new(),
            loaded_chunks: Vec::new(),
            initialized: false,
        })
    }

    /// Initialize the parser (read headers and basic info)
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Read just the headers
        while let Some(chunk) = self.streamer.next_chunk()? {
            match chunk {
                crate::streaming::StreamedChunk::Mver(_)
                | crate::streaming::StreamedChunk::Mhdr(_)
                | crate::streaming::StreamedChunk::Mcin(_)
                | crate::streaming::StreamedChunk::Mtex(_)
                | crate::streaming::StreamedChunk::Mmdx(_)
                | crate::streaming::StreamedChunk::Mmid(_)
                | crate::streaming::StreamedChunk::Mwmo(_)
                | crate::streaming::StreamedChunk::Mwid(_)
                | crate::streaming::StreamedChunk::Mddf(_)
                | crate::streaming::StreamedChunk::Modf(_) => {
                    // Stop once we reach the first MCNK
                    // or one of the version-specific chunks that typically come after MODF
                    continue;
                }
                _ => break,
            }
        }

        // Reset to after the headers
        self.streamer.skip_to_mcnk()?;

        self.initialized = true;
        Ok(())
    }

    /// Get information about the ADT
    pub fn get_info(&self) -> (crate::version::AdtVersion, Option<&MhdrChunk>) {
        (self.streamer.version(), self.streamer.mhdr())
    }

    /// Get a MCNK chunk by coordinates, loading it if necessary
    pub fn get_chunk(&mut self, x: u32, y: u32) -> Result<&McnkChunk> {
        // Check if the chunk is already loaded
        for (chunk_x, chunk_y, chunk) in &self.loaded_chunks {
            if *chunk_x == x && *chunk_y == y {
                return Ok(chunk);
            }
        }

        // Need to load the chunk
        self.load_chunk(x, y)?;

        // Find it in the loaded chunks
        for (chunk_x, chunk_y, chunk) in &self.loaded_chunks {
            if *chunk_x == x && *chunk_y == y {
                return Ok(chunk);
            }
        }

        Err(crate::error::AdtError::ParseError(format!(
            "Failed to load chunk at ({}, {})",
            x, y
        )))
    }

    /// Load a chunk by coordinates
    fn load_chunk(&mut self, x: u32, y: u32) -> Result<()> {
        // If we've reached the maximum number of chunks in memory, evict one
        if self.loaded_chunks.len() >= self.config.max_chunks_in_memory {
            self.loaded_chunks.remove(0); // Remove the oldest chunk
        }

        // Reset to the start of MCNK chunks
        self.streamer.skip_to_mcnk()?;

        // Scan through chunks until we find the one we want
        while let Some(chunk) = self.streamer.next_chunk()? {
            if let crate::streaming::StreamedChunk::Mcnk(mcnk) = chunk {
                if mcnk.ix == x && mcnk.iy == y {
                    // Found the chunk we're looking for
                    self.loaded_chunks.push((x, y, *mcnk));
                    return Ok(());
                }
            }
        }

        Err(crate::error::AdtError::ParseError(format!(
            "Could not find chunk at ({}, {})",
            x, y
        )))
    }

    /// Process each MCNK chunk with a callback
    pub fn process_chunks<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(u32, u32, &McnkChunk) -> Result<()>,
    {
        // Initialize if needed
        self.initialize()?;

        // Reset to the start of MCNK chunks
        self.streamer.skip_to_mcnk()?;

        // Process each chunk
        while let Some(chunk) = self.streamer.next_chunk()? {
            if let crate::streaming::StreamedChunk::Mcnk(mcnk) = chunk {
                callback(mcnk.ix, mcnk.iy, &*mcnk)?;
            }
        }

        Ok(())
    }

    /// Get a buffer from the pool or create a new one
    fn get_buffer(&mut self, min_size: usize) -> Vec<u8> {
        if self.config.use_memory_pool {
            // Try to find a buffer of appropriate size in the pool
            for i in 0..self.buffer_pool.len() {
                if self.buffer_pool[i].capacity() >= min_size {
                    let buf = self.buffer_pool.swap_remove(i);
                    return buf;
                }
            }
        }

        // Create a new buffer
        Vec::with_capacity(min_size)
    }

    /// Return a buffer to the pool
    fn return_buffer(&mut self, mut buf: Vec<u8>) {
        if self.config.use_memory_pool {
            // Clear the buffer but keep its capacity
            buf.clear();

            // Only keep buffers up to max_buffer_size
            if buf.capacity() <= self.config.max_buffer_size {
                self.buffer_pool.push(buf);
            }
        }
    }
}

/// Memory-efficient ADT structure using shared data
pub struct CompactAdt {
    /// Version of the ADT file
    version: crate::version::AdtVersion,
    /// MVER chunk - file version
    mver: MverChunk,
    /// MHDR chunk - header with offsets to other chunks
    mhdr: Option<MhdrChunk>,
    /// MCNK chunks - map chunk data (terrain height, texturing, etc.)
    mcnk_chunks: Vec<CompactMcnkChunk>,
    /// MCIN chunk - map chunk index
    mcin: Option<McinChunk>,
    /// Shared texture filenames
    textures: Rc<Vec<String>>,
    /// Shared model filenames
    models: Rc<Vec<String>>,
    /// Shared WMO filenames
    wmos: Rc<Vec<String>>,
    /// MDDF chunk - doodad placement information
    mddf: Option<MddfChunk>,
    /// MODF chunk - model placement information
    modf: Option<ModfChunk>,
    /// Version-specific data
    version_data: Option<VersionSpecificData>,
}

/// Version-specific data
pub struct VersionSpecificData {
    /// TBC and later - flight boundaries
    mfbo: Option<MfboChunk>,
    /// WotLK and later - water data
    mh2o: Option<Mh2oChunk>,
    /// Cataclysm and later - texture effects
    mtfx: Option<MtfxChunk>,
}

/// Memory-efficient version of McnkChunk
pub struct CompactMcnkChunk {
    /// Chunk coordinates and flags
    ix: u32,
    iy: u32,
    flags: u32,
    /// Number of layers
    n_layers: u32,
    /// Position
    position: [f32; 3],
    /// Area ID
    area_id: u32,
    /// Height map (stored as compact 16-bit values)
    height_data: Vec<u16>,
    /// Normal data (quantized)
    normal_data: Vec<u8>,
    /// Texture layers with shared texture references
    texture_layers: Vec<CompactTextureLayer>,
}

/// Memory-efficient texture layer information
pub struct CompactTextureLayer {
    /// Texture ID (index into shared textures)
    texture_id: u32,
    /// Flags
    flags: u32,
    /// Effect ID
    effect_id: u32,
    /// Shared alpha map data
    alpha_map: Option<Rc<Vec<u8>>>,
}

/// Open a memory-optimized ADT parser
pub fn open_optimized<P: AsRef<Path>>(
    path: P,
    config: MemoryConfig,
) -> Result<OptimizedAdtParser<File>> {
    let file = File::open(path)?;
    OptimizedAdtParser::new(file, config)
}

/// Load an ADT with memory optimizations
pub fn load_optimized<P: AsRef<Path>>(path: P, config: MemoryConfig) -> Result<Adt> {
    // Instead of loading the entire file at once, use the streaming parser
    let file = File::open(path)?;
    let mut streamer = AdtStreamer::new(file)?;

    // Build the ADT piece by piece
    let mut mver = None;
    let mut mhdr = None;
    let mut mcin = None;
    let mut mtex = None;
    let mut mmdx = None;
    let mut mmid = None;
    let mut mwmo = None;
    let mut mwid = None;
    let mut mddf = None;
    let mut modf = None;
    let mut mcnk_chunks = Vec::new();
    let mut mfbo = None;
    let mut mh2o = None;
    let mut mtfx = None;

    // Process each chunk
    while let Some(chunk) = streamer.next_chunk()? {
        match chunk {
            crate::streaming::StreamedChunk::Mver(chunk) => mver = Some(chunk),
            crate::streaming::StreamedChunk::Mhdr(chunk) => mhdr = Some(chunk),
            crate::streaming::StreamedChunk::Mcin(chunk) => mcin = Some(chunk),
            crate::streaming::StreamedChunk::Mtex(chunk) => mtex = Some(chunk),
            crate::streaming::StreamedChunk::Mmdx(chunk) => mmdx = Some(chunk),
            crate::streaming::StreamedChunk::Mmid(chunk) => mmid = Some(chunk),
            crate::streaming::StreamedChunk::Mwmo(chunk) => mwmo = Some(chunk),
            crate::streaming::StreamedChunk::Mwid(chunk) => mwid = Some(chunk),
            crate::streaming::StreamedChunk::Mddf(chunk) => mddf = Some(chunk),
            crate::streaming::StreamedChunk::Modf(chunk) => modf = Some(chunk),
            crate::streaming::StreamedChunk::Mcnk(chunk) => {
                // If we've reached the maximum chunks in memory and want to limit them
                if config.use_compact_structures && mcnk_chunks.len() >= config.max_chunks_in_memory
                {
                    // In a real implementation, we might compress or store on disk
                    // For now, just keep the latest chunks
                    mcnk_chunks.remove(0);
                }

                mcnk_chunks.push(*chunk);
            }
            crate::streaming::StreamedChunk::Mfbo(chunk) => mfbo = Some(chunk),
            crate::streaming::StreamedChunk::Mh2o(chunk) => mh2o = Some(chunk),
            crate::streaming::StreamedChunk::Mtfx(chunk) => mtfx = Some(chunk),
            _ => {}
        }
    }

    // Create the ADT
    Ok(Adt {
        version: streamer.version(),
        mver: mver.unwrap_or_else(|| MverChunk { version: 18 }),
        mhdr,
        mcnk_chunks,
        mcin,
        mtex,
        mmdx,
        mmid,
        mwmo,
        mwid,
        mddf,
        modf,
        mfbo,
        mh2o,
        mtfx,
    })
}

/// Convert normal array to a compact representation
pub fn compact_normals(normals: &[[i8; 3]]) -> Vec<u8> {
    // Encode each normal vector as 2 bytes using Lambert azimuthal equal-area projection
    // This provides a good balance between accuracy and memory usage
    // Full implementation would be more complex

    let mut result = Vec::with_capacity(normals.len() * 2);

    for normal in normals {
        // Simple encoding: just the X and Y components, Z can be derived
        result.push((normal[0] + 127) as u8);
        result.push((normal[1] + 127) as u8);
    }

    result
}

/// Encode heights as 16-bit values
pub fn compact_heights(heights: &[f32], min_height: f32, max_height: f32) -> Vec<u16> {
    let range = max_height - min_height;
    if range <= 0.0 {
        return vec![0; heights.len()];
    }

    let mut result = Vec::with_capacity(heights.len());

    for &height in heights {
        // Normalize to 0-65535 range
        let normalized = ((height - min_height) / range * 65535.0) as u16;
        result.push(normalized);
    }

    result
}

/// Pool of reusable MCNK chunks to reduce memory allocations
pub struct McnkPool {
    /// Available MCNK chunks
    chunks: Vec<McnkChunk>,
    /// Maximum pool size
    max_size: usize,
}

impl McnkPool {
    /// Create a new MCNK pool
    pub fn new(max_size: usize) -> Self {
        Self {
            chunks: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Get a chunk from the pool or create a new one
    pub fn get_chunk(&mut self) -> McnkChunk {
        if let Some(chunk) = self.chunks.pop() {
            // Reuse an existing chunk
            chunk
        } else {
            // Create a new chunk
            McnkChunk {
                flags: 0,
                ix: 0,
                iy: 0,
                n_layers: 0,
                n_doodad_refs: 0,
                mcvt_offset: 0,
                mcnr_offset: 0,
                mcly_offset: 0,
                mcrf_offset: 0,
                mcal_offset: 0,
                mcal_size: 0,
                mcsh_offset: 0,
                mcsh_size: 0,
                area_id: 0,
                n_map_obj_refs: 0,
                holes: 0,
                s1: 0,
                s2: 0,
                d1: 0,
                d2: 0,
                d3: 0,
                pred_tex: 0,
                n_effect_doodad: 0,
                mcse_offset: 0,
                n_sound_emitters: 0,
                liquid_offset: 0,
                liquid_size: 0,
                position: [0.0, 0.0, 0.0],
                mccv_offset: 0,
                mclv_offset: 0,
                texture_id: 0,
                props: 0,
                effect_id: 0,
                height_map: Vec::new(),
                normals: Vec::new(),
                texture_layers: Vec::new(),
                doodad_refs: Vec::new(),
                map_obj_refs: Vec::new(),
                alpha_maps: Vec::new(),
                mclq: None,
            }
        }
    }

    /// Return a chunk to the pool
    pub fn return_chunk(&mut self, mut chunk: McnkChunk) {
        if self.chunks.len() < self.max_size {
            // Clear vectors to free memory but keep capacity
            chunk.height_map.clear();
            chunk.normals.clear();
            chunk.texture_layers.clear();
            chunk.doodad_refs.clear();
            chunk.map_obj_refs.clear();
            chunk.alpha_maps.clear();

            // Add to the pool
            self.chunks.push(chunk);
        }
    }
}

/// Use memory-mapped files for zero-copy parsing
#[cfg(feature = "mmap")]
pub mod mmap {
    use super::*;
    use memmap2::{Mmap, MmapOptions};
    use std::io::Cursor;

    /// Parse an ADT file using memory mapping for zero-copy parsing
    pub fn parse_mmap<P: AsRef<Path>>(path: P) -> Result<Adt> {
        let file = File::open(path)?;

        // Create a memory map
        let mmap = unsafe { Mmap::map(&file)? };

        // Create a cursor over the memory map
        let mut cursor = Cursor::new(&mmap[..]);

        // Parse the ADT
        Adt::from_reader(&mut cursor)
    }

    /// Process specific parts of an ADT file without loading the whole file
    pub fn process_mmap_regions<P: AsRef<Path>, F>(
        path: P,
        regions: &[(u64, u64)], // (offset, size) pairs
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(u64, &[u8]) -> Result<()>,
    {
        let file = File::open(path)?;

        // Create a memory map
        let mmap = unsafe { Mmap::map(&file)? };

        // Process each region
        for &(offset, size) in regions {
            if offset + size <= mmap.len() as u64 {
                let start = offset as usize;
                let end = (offset + size) as usize;
                callback(offset, &mmap[start..end])?;
            } else {
                return Err(crate::error::AdtError::ParseError(format!(
                    "Region out of bounds: offset={}, size={}",
                    offset, size
                )));
            }
        }

        Ok(())
    }
}
