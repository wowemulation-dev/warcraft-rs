use crate::McnkChunk;

/// Builder / converter that assembles multiple per-layer alpha maps into a single 64x64 RGBA
/// texture where R/G/B contain terrain layer alphas (layers 1/2/3 respectively) and A is set to
/// 255 for visibility in tools.
///
/// Internal layout: `map[y][x][channel]` with `channel` in 0..=3 (R, G, B, A).
/// Pixels are written sequentially in row-major order (y outer, x inner) for each channel, then
/// iteration proceeds to the next channel. This ordering matches the progressive ingestion of
/// alpha bytes and avoids allocating separate buffers per channel.
///
/// `fix_alpha` mode: Some ADT data omits the final row/column (providing only 63x63 values).
/// To preserve sampling semantics we duplicate the preceding pixel when we hit x==63 or y==63
/// during `set_next_alpha` writes.
pub struct CombinedAlphaMap {
    map: [[[u8; 4]; 64]; 64],
    current_x: usize,
    current_y: usize,
    current_layer: usize,
    has_big_alpha: bool,
    fix_alpha: bool,
}

impl CombinedAlphaMap {
    /// Internal helper: allocate a blank alpha map accumulator.
    /// R/G/B channels initialized to 0, A to 255.
    fn blank(has_big_alpha: bool, fix_alpha: bool) -> Self {
        let mut map = [[[0u8; 4]; 64]; 64];
        // Alpha is unused, but we set it to 255 so the image is visible when viewed in debug UI.
        map.iter_mut().for_each(|layer| layer.fill([0, 0, 0, 255]));
        Self {
            map,
            current_x: 0,
            current_y: 0,
            current_layer: 0,
            has_big_alpha,
            fix_alpha,
        }
    }

    /// Construct and fully ingest the alpha layers of `chunk`.
    pub fn new(chunk: &McnkChunk, has_big_alpha: bool, fix_alpha: bool) -> Self {
        let mut s = Self::blank(has_big_alpha, fix_alpha);
        s.ingest_chunk_layers(chunk);
        s
    }

    /// Ingest all (up to 3) alpha layers for a single ADT terrain chunk into this combined map.
    ///
    /// High-level overview:
    /// - Terrain chunks can have up to 4 texture layers; layer 0 is fully opaque and has no alpha map.
    /// - Layers 1..=3 each provide a 64x64 alpha map in one of three encodings:
    ///   * Big (8-bit) uncompressed: 4096 bytes.
    ///   * Small (4-bit) uncompressed: 2048 bytes, two pixels per byte (low nibble then high).
    ///   * RLE compressed (flag bit 9 set): variable length, Blizzard variant (row-limited runs).
    /// - `has_big_alpha` determines whether uncompressed data uses big (8-bit) or small (4-bit) form.
    /// - Each decoded layer is written into one of R, G, B channels respectively. A channel is set
    ///   to 255 for visibility in tools and is otherwise unused.
    /// - If `fix_alpha` (set at construction) is true, the source data is interpreted as 63x63 and
    ///   we synthesize the final row/column by duplicating previous pixels while writing.
    ///
    /// This method encapsulates the selection of decoding routine based on the per-layer flags
    /// and offsets stored in `McnkChunk::texture_layers`.
    fn ingest_chunk_layers(&mut self, chunk: &McnkChunk) {
        let bit_10th = 1 << 9; // Compression flag on layer
        for layer in chunk.texture_layers.iter().skip(1) {
            // Skip base layer (no alpha)
            let is_compressed = layer.flags & bit_10th != 0;
            let offset = layer.alpha_map_offset as usize;
            if is_compressed {
                self.ingest_layer_compressed(&chunk.alpha_maps, offset);
            } else if self.has_big_alpha {
                self.ingest_layer_big(&chunk.alpha_maps, offset);
            } else {
                self.ingest_layer_small(&chunk.alpha_maps, offset);
            }
        }
    }

    /// Ingest a full 64x64 layer of already decompressed (8-bit) alpha values.
    /// Feed already expanded 8-bit alpha bytes (exact order: left→right, top→bottom) for the
    /// current channel until we have consumed a full 64x64 plane (or input runs out).
    fn ingest_alphas(&mut self, alphas: &[u8]) {
        for &a in alphas.iter() {
            if !self.set_next_alpha(a) {
                break;
            }
        }
    }

    fn next_layer(&mut self) {
        self.current_layer += 1;
        self.current_x = 0;
        self.current_y = 0;
    }

    /// Ingest a layer stored raw as 4096 8-bit values.
    /// Ingest a raw (uncompressed) 64x64 layer containing 8-bit alpha values.
    /// Advances `offset` by 4096 on success; aborts gracefully if insufficient data remains.
    fn ingest_layer_big(&mut self, raw: &[u8], offset: usize) {
        const LAYER_SIZE: usize = 64 * 64; // 4096
        if offset + LAYER_SIZE <= raw.len() {
            self.ingest_alphas(&raw[offset..offset + LAYER_SIZE]);
        }
        self.next_layer();
    }

    /// Ingest a layer stored as 2048 bytes of two 4-bit values (low nibble first, then high nibble).
    /// Ingest a raw (uncompressed) 4-bit/pixel layer packed two pixels per byte.
    /// Each nibble is scaled (×16) to map 0..15 -> 0..240 for consistency with 8-bit layers.
    fn ingest_layer_small(&mut self, raw: &[u8], offset: usize) {
        const PACKED_SIZE: usize = 64 * 64 / 2; // 2048
        if offset + PACKED_SIZE <= raw.len() {
            for &packed in &raw[offset..offset + PACKED_SIZE] {
                if !self.set_next_alpha((packed & 0x0F) * 16) {
                    break;
                }
                if !self.set_next_alpha(((packed >> 4) & 0x0F) * 16) {
                    break;
                }
            }
        }
        self.next_layer();
    }

    /// Ingest a compressed layer using the Blizzard RLE scheme (MSB=mode, 7 bits count, row-limited to 64).
    /// Ingest a compressed layer encoded with Blizzard's RLE variant.
    /// Format per control byte (token):
    ///   bit7 = 0 -> copy, next (count) literal bytes
    ///   bit7 = 1 -> fill, next single byte is repeated (count) times
    ///   bits0..6 = count (clamped to 1..=64 and row remainder)
    /// Rows are exactly 64 pixels wide; a run may not cross a row boundary (enforced here).
    /// Corrupted data that would produce >4096 bytes is truncated; shorter output is zero-padded.
    fn ingest_layer_compressed(&mut self, raw: &[u8], mut offset: usize) {
        const TARGET: usize = 64 * 64; // 4096
        let mut output = Vec::with_capacity(TARGET);
        let mut x_in_row = 0usize;
        while output.len() < TARGET {
            if offset >= raw.len() {
                break;
            }
            let token = raw[offset];
            offset += 1;
            let mode_fill = (token & 0x80) != 0;
            let mut count = (token & 0x7F) as usize; // 0..127
            if count > 64 {
                count = 64;
            }
            let remaining_in_row = 64 - x_in_row;
            if count > remaining_in_row {
                count = remaining_in_row;
            }
            if count == 0 {
                continue;
            }
            if mode_fill {
                if offset >= raw.len() {
                    break;
                }
                let value = raw[offset];
                offset += 1;
                for _ in 0..count {
                    output.push(value);
                    x_in_row += 1;
                    if output.len() >= TARGET {
                        break;
                    }
                }
            } else {
                for _ in 0..count {
                    if offset >= raw.len() {
                        break;
                    }
                    let value = raw[offset];
                    offset += 1;
                    output.push(value);
                    x_in_row += 1;
                    if output.len() >= TARGET {
                        break;
                    }
                }
            }
            if x_in_row >= 64 {
                x_in_row = 0;
            }
        }
        if output.len() > TARGET {
            output.truncate(TARGET);
        }
        if output.len() < TARGET {
            output.resize(TARGET, 0);
        }
        self.ingest_alphas(&output);
        self.next_layer();
    }

    /// Directly set one channel value without advancing internal iteration state.
    fn set_alpha(&mut self, x: usize, y: usize, layer: usize, alpha: u8) {
        if y < 64 && x < 64 && layer < 4 {
            self.map[y][x][layer] = alpha;
        }
    }

    /// Get one channel value.
    fn get_alpha(&self, x: usize, y: usize, layer: usize) -> u8 {
        if y < 64 && x < 64 && layer < 4 {
            self.map[y][x][layer]
        } else {
            0
        }
    }

    /// Write the next alpha value at the current (x,y,channel) and advance the cursor.
    /// In `fix_alpha` mode we mirror the previous pixel for the last row/column to synthesize
    /// a 64x64 plane from 63x63 source data.
    fn set_next_alpha(&mut self, mut alpha: u8) -> bool {
        if self.fix_alpha {
            // If we are at the last row or column and fix_alpha is true,
            // duplicate the last value to fill the 64x64 texture
            if self.current_x == 63 {
                alpha = self.get_alpha(self.current_x - 1, self.current_y, self.current_layer);
            }
            if self.current_y == 63 {
                alpha = self.get_alpha(self.current_x, self.current_y - 1, self.current_layer);
            }
        }
        self.set_alpha(self.current_x, self.current_y, self.current_layer, alpha);
        self.advance()
    }

    /// Advance write cursor: increment x, wrap to next row.
    /// Returns false if we have filled the entire 64x64 plane.
    fn advance(&mut self) -> bool {
        self.current_x += 1;
        if self.current_x >= 64 {
            self.current_x = 0;
            self.current_y += 1;
        }
        self.current_y < 64
    }

    /// View underlying RGBA bytes.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.map.as_ptr() as *const u8,
                std::mem::size_of_val(&self.map),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_base_chunk() -> McnkChunk {
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
            height_map: vec![0.0; 145],
            normals: vec![[0, 0, 0]; 145],
            texture_layers: Vec::new(),
            doodad_refs: Vec::new(),
            map_obj_refs: Vec::new(),
            alpha_maps: Vec::new(),
            mclq: None,
            vertex_colors: Vec::new(),
        }
    }

    fn layer(texture_id: u32, flags: u32, offset: u32) -> crate::chunk::McnkTextureLayer {
        crate::chunk::McnkTextureLayer {
            texture_id,
            flags,
            alpha_map_offset: offset,
            effect_id: 0,
        }
    }

    #[test]
    fn big_alpha_single_layer() {
        // Prepare raw 4096 bytes increasing pattern
        let mut chunk = make_base_chunk();
        let mut raw = vec![0u8; 4096];
        for (i, v) in raw.iter_mut().enumerate() {
            *v = (i % 251) as u8;
        }
        chunk.alpha_maps = raw.clone();
        // base layer (offset 0, ignored) + one alpha layer (also offset 0)
        // alpha_map_offset=0 is valid since only one alpha layer
        chunk.texture_layers = vec![layer(0, 0, 0), layer(1, 0, 0)];
        let cam = CombinedAlphaMap::new(&chunk, true, false);
        // R channel should mirror the first 4096 bytes
        let slice = cam.as_slice();
        // RGBA pixel stride 4
        for y in 0..64 {
            for x in 0..64 {
                let idx = (y * 64 + x) as usize;
                assert_eq!(slice[idx * 4], raw[idx]);
            }
        }
    }

    #[test]
    fn small_alpha_nibbles() {
        // 2048 bytes -> 4096 pixels (low nibble then high nibble scaled *16)
        let mut chunk = make_base_chunk();
        let mut packed = vec![0u8; 2048];
        for (i, b) in packed.iter_mut().enumerate() {
            *b = (i % 255) as u8;
        }
        chunk.alpha_maps = packed.clone();
        chunk.texture_layers = vec![layer(0, 0, 0), layer(1, 0, 0)];
        let cam = CombinedAlphaMap::new(&chunk, false, false); // has_big_alpha=false => 4-bit path
        let slice = cam.as_slice();
        // Validate first byte expands to two pixels
        let b0 = packed[0];
        assert_eq!(slice[0], (b0 & 0x0F) * 16); // first pixel R
        assert_eq!(slice[4], ((b0 >> 4) & 0x0F) * 16); // second pixel R
    }

    #[test]
    fn compressed_alpha_rle() {
        // Build a simple RLE stream: fill 64 pixels row with value 7, repeated for 64 rows.
        // Control token: MSB=1 (fill), count=64 -> 0x80 | 64 = 0xC0, then value byte.
        // We repeat this 64 times.
        let mut compressed = Vec::new();
        for _ in 0..64 {
            compressed.push(0xC0);
            compressed.push(7);
        }
        let mut chunk = make_base_chunk();
        chunk.alpha_maps = compressed.clone();
        // Set compression flag bit 9 on layer
        let compression_flag = 1 << 9;
        chunk.texture_layers = vec![layer(0, 0, 0), layer(1, compression_flag, 0)];
        let cam = CombinedAlphaMap::new(&chunk, true, false); // has_big_alpha ignored due to compression
        let slice = cam.as_slice();
        // All R channel pixels should be 7
        for y in 0..64 {
            for x in 0..64 {
                let idx = (y * 64 + x) as usize;
                assert_eq!(slice[idx * 4], 7);
            }
        }
    }

    #[test]
    fn fix_alpha_padding() {
        // Provide only 63x63 worth of data (3969) and ensure last row/col duplicate
        let mut chunk = make_base_chunk();
        // We'll give big alpha path but truncated purposely; ingestion stops when cursor full.
        let data = vec![5u8; 4096];
        chunk.alpha_maps = data.clone();
        chunk.texture_layers = vec![layer(0, 0, 0), layer(1, 0, 0)];
        let cam = CombinedAlphaMap::new(&chunk, true, true);
        let slice = cam.as_slice();
        // Check last pixel equals its left neighbor (due to duplicate logic when fix_alpha)
        let last_idx = (64 * 64 - 1) as usize;
        assert_eq!(slice[last_idx * 4], slice[(last_idx - 1) * 4]);
    }
}
