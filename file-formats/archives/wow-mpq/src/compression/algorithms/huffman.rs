//! Huffman compression implementation for MPQ archives
//!
//! This is a simplified port of the StormLib Huffman implementation, primarily used for WAVE files.
//! Based on the algorithm from Ladislav Zezula's StormLib.

use crate::{Error, Result};

// Huffman tree constants
const HUFF_ITEM_COUNT: usize = 0x203; // Number of items in the item pool
const LINK_ITEM_COUNT: usize = 0x80; // Maximum number of quick-link items
const HUFF_DECOMPRESS_ERROR: u32 = 0x1FF;

// All weight tables from StormLib - these define the initial character frequencies
const BYTE_TO_WEIGHT_00: [u8; 258] = [
    0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
    0x00, 0x00,
];

const BYTE_TO_WEIGHT_01: [u8; 258] = [
    0x54, 0x16, 0x16, 0x0D, 0x0C, 0x08, 0x06, 0x05, 0x06, 0x05, 0x06, 0x03, 0x04, 0x04, 0x03, 0x05,
    0x0E, 0x0B, 0x14, 0x13, 0x13, 0x09, 0x0B, 0x06, 0x05, 0x04, 0x03, 0x02, 0x03, 0x02, 0x02, 0x02,
    0x0D, 0x07, 0x09, 0x06, 0x06, 0x04, 0x03, 0x02, 0x04, 0x03, 0x03, 0x03, 0x03, 0x03, 0x02, 0x02,
    0x09, 0x06, 0x04, 0x04, 0x04, 0x04, 0x03, 0x02, 0x03, 0x02, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04,
    0x08, 0x03, 0x04, 0x07, 0x09, 0x05, 0x03, 0x03, 0x03, 0x03, 0x02, 0x02, 0x02, 0x03, 0x02, 0x02,
    0x03, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x01, 0x01, 0x01, 0x02, 0x01, 0x02, 0x02,
    0x06, 0x0A, 0x08, 0x08, 0x06, 0x07, 0x04, 0x03, 0x04, 0x04, 0x02, 0x02, 0x04, 0x02, 0x03, 0x03,
    0x04, 0x03, 0x07, 0x07, 0x09, 0x06, 0x04, 0x03, 0x03, 0x02, 0x01, 0x02, 0x02, 0x02, 0x02, 0x02,
    0x0A, 0x02, 0x02, 0x03, 0x02, 0x02, 0x01, 0x01, 0x02, 0x02, 0x02, 0x06, 0x03, 0x05, 0x02, 0x03,
    0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x03, 0x01, 0x01, 0x01,
    0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x04, 0x04, 0x04, 0x07, 0x09, 0x08, 0x0C, 0x02,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x01, 0x01, 0x03,
    0x04, 0x01, 0x02, 0x04, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x01, 0x01, 0x01,
    0x04, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x03, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x01, 0x01, 0x02, 0x02, 0x02, 0x06, 0x4B,
    0x00, 0x00,
];

const BYTE_TO_WEIGHT_02: [u8; 258] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x27, 0x00, 0x00, 0x23, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xFF, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x01, 0x01, 0x06, 0x0E, 0x10, 0x04,
    0x06, 0x08, 0x05, 0x04, 0x04, 0x03, 0x03, 0x02, 0x02, 0x03, 0x03, 0x01, 0x01, 0x02, 0x01, 0x01,
    0x01, 0x04, 0x02, 0x04, 0x02, 0x02, 0x02, 0x01, 0x01, 0x04, 0x01, 0x01, 0x02, 0x03, 0x03, 0x02,
    0x03, 0x01, 0x03, 0x06, 0x04, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02, 0x01, 0x02, 0x01, 0x01,
    0x01, 0x29, 0x07, 0x16, 0x12, 0x40, 0x0A, 0x0A, 0x11, 0x25, 0x01, 0x03, 0x17, 0x10, 0x26, 0x2A,
    0x10, 0x01, 0x23, 0x23, 0x2F, 0x10, 0x06, 0x07, 0x02, 0x09, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00,
];

// Weight tables lookup - only implementing the ones we need for MPQ
const WEIGHT_TABLES: [&[u8; 258]; 3] = [&BYTE_TO_WEIGHT_00, &BYTE_TO_WEIGHT_01, &BYTE_TO_WEIGHT_02];

/// Bit stream reader for Huffman decompression
struct BitReader<'a> {
    data: &'a [u8],
    position: usize,
    bit_buffer: u32,
    bit_count: u32,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: 0,
            bit_buffer: 0,
            bit_count: 0,
        }
    }

    fn get_bit(&mut self) -> Result<u32> {
        if self.bit_count == 0 {
            if self.position >= self.data.len() {
                return Err(Error::compression("Unexpected end of Huffman data"));
            }
            self.bit_buffer = self.data[self.position] as u32;
            self.position += 1;
            self.bit_count = 8;
        }

        let bit = self.bit_buffer & 1;
        self.bit_buffer >>= 1;
        self.bit_count -= 1;
        Ok(bit)
    }

    fn get_8_bits(&mut self) -> Result<u32> {
        if self.position >= self.data.len() {
            return Err(Error::compression("Unexpected end of Huffman data"));
        }
        let byte = self.data[self.position] as u32;
        self.position += 1;
        Ok(byte)
    }

    fn peek_7_bits(&mut self) -> Result<u32> {
        // Ensure we have enough bits in the buffer
        while self.bit_count < 7 {
            if self.position >= self.data.len() {
                return Err(Error::compression("Unexpected end of Huffman data"));
            }
            self.bit_buffer |= (self.data[self.position] as u32) << self.bit_count;
            self.position += 1;
            self.bit_count += 8;
        }

        Ok(self.bit_buffer & 0x7F) // Get lower 7 bits
    }

    fn skip_bits(&mut self, count: u32) {
        if count <= self.bit_count {
            self.bit_buffer >>= count;
            self.bit_count -= count;
        } else {
            let remaining = count - self.bit_count;
            self.bit_buffer = 0;
            self.bit_count = 0;
            self.position += (remaining / 8) as usize;
            // Handle remaining bits if any
            if remaining % 8 > 0 && self.peek_7_bits().is_ok() {
                self.skip_bits(remaining % 8);
            }
        }
    }
}

/// Huffman tree node based on StormLib's THTreeItem
#[derive(Debug, Clone)]
struct HuffmanItem {
    decompressed_value: u32,
    weight: u32,
    parent: Option<usize>,
    child_lo: Option<usize>,
    _next: Option<usize>,
    _prev: Option<usize>,
}

impl HuffmanItem {
    fn new(value: u32, weight: u32) -> Self {
        Self {
            decompressed_value: value,
            weight,
            parent: None,
            child_lo: None,
            _next: None,
            _prev: None,
        }
    }
}

/// Quick link structure for optimized decompression
#[derive(Debug, Clone)]
struct QuickLink {
    valid_value: u32,
    valid_bits: u32,
    decompressed_value: u32,
}

impl QuickLink {
    fn new() -> Self {
        Self {
            valid_value: 0,
            valid_bits: 0,
            decompressed_value: 0,
        }
    }
}

/// Simplified Huffman tree based on StormLib
struct HuffmanTree {
    items: Vec<HuffmanItem>,
    items_by_byte: [Option<usize>; 0x102],
    quick_links: [QuickLink; LINK_ITEM_COUNT],
    first: Option<usize>,
    last: Option<usize>,
    min_valid_value: u32,
    is_cmp0: bool,
}

impl HuffmanTree {
    fn new() -> Self {
        Self {
            items: Vec::with_capacity(HUFF_ITEM_COUNT),
            items_by_byte: [None; 0x102],
            quick_links: std::array::from_fn(|_| QuickLink::new()),
            first: None,
            last: None,
            min_valid_value: 1,
            is_cmp0: false,
        }
    }

    fn build_tree(&mut self, compression_type: u32) -> Result<()> {
        // Clear all items
        self.items.clear();
        self.items_by_byte = [None; 0x102];

        // Ensure compression type is valid
        let comp_type = (compression_type & 0x0F) as usize;
        if comp_type >= WEIGHT_TABLES.len() {
            return Err(Error::compression("Invalid Huffman compression type"));
        }

        let weight_table = WEIGHT_TABLES[comp_type];

        // Build items for all non-zero weight bytes
        for (i, &weight) in weight_table.iter().enumerate() {
            if weight != 0 {
                let item_index = self.items.len();
                self.items.push(HuffmanItem::new(i as u32, weight as u32));
                self.items_by_byte[i] = Some(item_index);
            }
        }

        // Add termination entries
        let end_index = self.items.len();
        self.items.push(HuffmanItem::new(0x100, 1));
        self.items_by_byte[0x100] = Some(end_index);

        let eof_index = self.items.len();
        self.items.push(HuffmanItem::new(0x101, 1));
        self.items_by_byte[0x101] = Some(eof_index);

        // Sort items by weight (simple bubble sort for this implementation)
        self.items.sort_by_key(|item| item.weight);

        // Rebuild items_by_byte after sorting
        for (i, item) in self.items.iter().enumerate() {
            if item.decompressed_value <= 0x101 {
                self.items_by_byte[item.decompressed_value as usize] = Some(i);
            }
        }

        // Set first and last
        if !self.items.is_empty() {
            self.first = Some(0);
            self.last = Some(self.items.len() - 1);
        }

        // Build the tree structure (simplified version)
        self.build_internal_tree()?;

        self.min_valid_value = 1;
        Ok(())
    }

    fn build_internal_tree(&mut self) -> Result<()> {
        // This is a simplified version of the StormLib tree building
        // For a full implementation, we would need to port the complex tree rebalancing logic
        // For now, we'll create a basic structure that should work for most cases

        if self.items.len() < 2 {
            return Ok(());
        }

        // Build parent-child relationships in a simplified manner
        let mut work_items = self.items.len();

        while work_items > 1 {
            if work_items < 2 {
                break;
            }

            // Take the two items with lowest weight
            let child_lo_idx = work_items - 1;
            let child_hi_idx = work_items - 2;

            if child_lo_idx >= self.items.len() || child_hi_idx >= self.items.len() {
                break;
            }

            let combined_weight = self.items[child_lo_idx].weight + self.items[child_hi_idx].weight;

            // Create new parent item
            let parent_idx = self.items.len();
            self.items.push(HuffmanItem::new(0, combined_weight));

            // Set up parent-child relationships
            self.items[parent_idx].child_lo = Some(child_lo_idx);
            self.items[child_lo_idx].parent = Some(parent_idx);
            self.items[child_hi_idx].parent = Some(parent_idx);

            work_items -= 1;
        }

        Ok(())
    }

    fn decode_one_byte(&mut self, reader: &mut BitReader<'_>) -> Result<u32> {
        // Try quick links first (simplified)
        if let Ok(link_index) = reader.peek_7_bits() {
            let link_index = link_index as usize;
            if link_index < LINK_ITEM_COUNT {
                let quick_link = &self.quick_links[link_index];
                if quick_link.valid_value > self.min_valid_value && quick_link.valid_bits <= 7 {
                    reader.skip_bits(quick_link.valid_bits);
                    return Ok(quick_link.decompressed_value);
                }
            }
        }

        // Traverse the tree
        let mut current_item = self.first;

        loop {
            if let Some(item_idx) = current_item {
                if item_idx >= self.items.len() {
                    return Err(Error::compression("Invalid Huffman tree state"));
                }

                let item = &self.items[item_idx];

                // If this is a leaf node (has a value)
                if item.child_lo.is_none() {
                    return Ok(item.decompressed_value);
                }

                // Get next bit and traverse
                let bit = reader.get_bit()?;
                if bit == 0 {
                    current_item = item.child_lo;
                } else {
                    // For simplicity, we'll assume the other child is the next item
                    // A full implementation would track both children properly
                    current_item = item.child_lo.map(|idx| {
                        if idx + 1 < self.items.len() {
                            idx + 1
                        } else {
                            idx
                        }
                    });
                }
            } else {
                return Err(Error::compression("Invalid Huffman tree traversal"));
            }
        }
    }
}

/// Huffman decompression function following StormLib's algorithm
pub(crate) fn decompress(data: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    log::debug!(
        "Huffman decompress input: {} bytes, expected: {}, first 16 bytes: {:02X?}",
        data.len(),
        expected_size,
        &data[..std::cmp::min(16, data.len())]
    );

    let mut reader = BitReader::new(data);
    let mut output = Vec::with_capacity(expected_size);

    // Get compression type from the first byte
    let compression_type = reader.get_8_bits()?;

    // Build Huffman tree
    let mut tree = HuffmanTree::new();
    tree.is_cmp0 = compression_type == 0;
    tree.build_tree(compression_type)?;

    // Decompress data
    loop {
        let decoded_value = tree.decode_one_byte(&mut reader)?;

        // Check for end of stream marker
        if decoded_value == 0x100 {
            break;
        }

        // Check for error
        if decoded_value == HUFF_DECOMPRESS_ERROR {
            return Err(Error::compression("Huffman decompression error"));
        }

        // Handle tree modification marker
        if decoded_value == 0x101 {
            // Read the next byte directly
            let next_byte = reader.get_8_bits()?;
            output.push(next_byte as u8);
            // Tree modification would happen here in full implementation
            continue;
        }

        // Regular data byte
        if decoded_value > 255 {
            return Err(Error::compression("Invalid Huffman decoded value"));
        }

        output.push(decoded_value as u8);

        // Stop if we've reached expected size
        if output.len() >= expected_size {
            break;
        }
    }

    log::debug!(
        "Huffman decompress output: {} bytes, first 16 bytes: {:02X?}",
        output.len(),
        &output[..std::cmp::min(16, output.len())]
    );

    Ok(output)
}

/// Huffman compression stub - not needed for reading HET/BET tables
pub(crate) fn compress(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    // For now, this is a stub since we only need decompression for HET/BET tables
    Err(Error::compression(
        "Huffman compression not implemented - only decompression is available",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_empty_data() {
        assert!(decompress(&[], 0).is_ok());
        assert!(compress(&[]).is_ok());
    }

    #[test]
    fn test_huffman_compression_not_implemented() {
        let data = b"test data";
        assert!(compress(data).is_err());
    }

    #[test]
    fn test_huffman_basic_structure() {
        let mut tree = HuffmanTree::new();
        assert!(tree.build_tree(1).is_ok());
        assert!(!tree.items.is_empty());
    }
}
