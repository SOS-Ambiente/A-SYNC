// Huffman compression module
use crate::error::{MSSCSError, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Huffman tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HuffmanNode {
    Leaf { value: u8, freq: usize },
    Internal { freq: usize, left: Box<HuffmanNode>, right: Box<HuffmanNode> },
}

impl HuffmanNode {
    fn freq(&self) -> usize {
        match self {
            HuffmanNode::Leaf { freq, .. } => *freq,
            HuffmanNode::Internal { freq, .. } => *freq,
        }
    }
}

// Wrapper for priority queue (min-heap based on frequency)
struct NodeWrapper(HuffmanNode);

impl PartialEq for NodeWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.freq() == other.0.freq()
    }
}

impl Eq for NodeWrapper {}

impl PartialOrd for NodeWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.0.freq().cmp(&self.0.freq())
    }
}

/// Bit-level writer
pub struct BitWriter {
    data: Vec<u8>,
    current_byte: u8,
    bit_pos: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            current_byte: 0,
            bit_pos: 7,
        }
    }

    pub fn write_bit(&mut self, bit: u8) {
        self.current_byte |= (bit & 1) << self.bit_pos;
        if self.bit_pos == 0 {
            self.data.push(self.current_byte);
            self.current_byte = 0;
            self.bit_pos = 7;
        } else {
            self.bit_pos -= 1;
        }
    }

    pub fn write_bits(&mut self, bits: u32, mut count: u8) {
        while count > 0 {
            self.write_bit((bits >> (count - 1)) as u8);
            count -= 1;
        }
    }

    pub fn flush(&mut self) {
        if self.bit_pos < 7 {
            self.data.push(self.current_byte);
            self.current_byte = 0;
            self.bit_pos = 7;
        }
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        self.flush();
        self.data
    }
}

/// Bit-level reader
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 7,
        }
    }

    pub fn read_bit(&mut self) -> Result<u8> {
        if self.byte_pos >= self.data.len() {
            return Err(MSSCSError::Compression("End of bit stream".to_string()));
        }
        let bit = (self.data[self.byte_pos] >> self.bit_pos) & 1;
        if self.bit_pos == 0 {
            self.byte_pos += 1;
            self.bit_pos = 7;
        } else {
            self.bit_pos -= 1;
        }
        Ok(bit)
    }

    pub fn read_bits(&mut self, count: u8) -> Result<u32> {
        let mut result = 0;
        for _ in 0..count {
            result = (result << 1) | self.read_bit()? as u32;
        }
        Ok(result)
    }

    pub fn has_more(&self) -> bool {
        self.byte_pos < self.data.len()
    }
}


/// Build Huffman tree from data
fn build_huffman_tree(data: &[u8]) -> Result<HuffmanNode> {
    if data.is_empty() {
        return Err(MSSCSError::Compression("Cannot build tree from empty data".to_string()));
    }

    // Count frequency of each symbol
    let mut freq_map: HashMap<u8, usize> = HashMap::new();
    for &symbol in data {
        *freq_map.entry(symbol).or_insert(0) += 1;
    }

    // Create leaf nodes and add to priority queue
    let mut heap = BinaryHeap::new();
    for (value, freq) in freq_map {
        heap.push(NodeWrapper(HuffmanNode::Leaf { value, freq }));
    }

    // Handle single symbol case
    if heap.len() == 1 {
        let node = heap.pop().unwrap().0;
        return Ok(HuffmanNode::Internal {
            freq: node.freq(),
            left: Box::new(node),
            right: Box::new(HuffmanNode::Leaf { value: 0, freq: 0 }),
        });
    }

    // Build tree by combining lowest frequency nodes
    while heap.len() > 1 {
        let left = heap.pop().unwrap().0;
        let right = heap.pop().unwrap().0;
        let combined_freq = left.freq() + right.freq();
        heap.push(NodeWrapper(HuffmanNode::Internal {
            freq: combined_freq,
            left: Box::new(left),
            right: Box::new(right),
        }));
    }

    Ok(heap.pop().unwrap().0)
}

/// Generate codes from Huffman tree
fn generate_codes(node: &HuffmanNode, code: u32, depth: u8, codes: &mut HashMap<u8, (u32, u8)>) {
    match node {
        HuffmanNode::Leaf { value, .. } => {
            codes.insert(*value, (code, depth));
        }
        HuffmanNode::Internal { left, right, .. } => {
            generate_codes(left, code << 1, depth + 1, codes);
            generate_codes(right, (code << 1) | 1, depth + 1, codes);
        }
    }
}

/// Serialize Huffman tree using pre-order traversal
pub fn serialize_tree(node: &HuffmanNode) -> Vec<u8> {
    let mut writer = BitWriter::new();
    serialize_tree_recursive(node, &mut writer);
    writer.into_bytes()
}

fn serialize_tree_recursive(node: &HuffmanNode, writer: &mut BitWriter) {
    match node {
        HuffmanNode::Leaf { value, .. } => {
            writer.write_bit(1); // Leaf marker
            writer.write_bits(*value as u32, 8); // Symbol value
        }
        HuffmanNode::Internal { left, right, .. } => {
            writer.write_bit(0); // Internal marker
            serialize_tree_recursive(left, writer);
            serialize_tree_recursive(right, writer);
        }
    }
}

/// Deserialize Huffman tree
pub fn deserialize_tree(data: &[u8]) -> Result<HuffmanNode> {
    let mut reader = BitReader::new(data);
    deserialize_tree_recursive(&mut reader)
}

fn deserialize_tree_recursive(reader: &mut BitReader) -> Result<HuffmanNode> {
    let bit = reader.read_bit()?;
    if bit == 1 {
        // Leaf node
        let value = reader.read_bits(8)? as u8;
        Ok(HuffmanNode::Leaf { value, freq: 0 })
    } else {
        // Internal node
        let left = Box::new(deserialize_tree_recursive(reader)?);
        let right = Box::new(deserialize_tree_recursive(reader)?);
        Ok(HuffmanNode::Internal { freq: 0, left, right })
    }
}

/// Compress data using Huffman coding
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    // Build Huffman tree
    let tree = build_huffman_tree(data)?;

    // Generate codes
    let mut codes = HashMap::new();
    generate_codes(&tree, 0, 0, &mut codes);

    // Serialize tree
    let tree_bytes = serialize_tree(&tree);

    // Write tree size (4 bytes) + tree + data length (4 bytes) + compressed data
    let mut result = Vec::new();
    result.extend_from_slice(&(tree_bytes.len() as u32).to_be_bytes());
    result.extend_from_slice(&tree_bytes);
    result.extend_from_slice(&(data.len() as u32).to_be_bytes());

    // Compress data
    let mut writer = BitWriter::new();
    for &symbol in data {
        if let Some(&(code, bit_length)) = codes.get(&symbol) {
            writer.write_bits(code, bit_length);
        } else {
            return Err(MSSCSError::Compression(format!("Symbol {} not in code table", symbol)));
        }
    }
    result.extend_from_slice(&writer.into_bytes());

    Ok(result)
}

/// Decompress data using Huffman coding
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    if data.len() < 8 {
        return Err(MSSCSError::Compression("Invalid compressed data: too short".to_string()));
    }

    // Read tree size
    let tree_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 8 + tree_size {
        return Err(MSSCSError::Compression("Invalid compressed data: tree size mismatch".to_string()));
    }

    // Deserialize tree
    let tree_bytes = &data[4..4 + tree_size];
    let tree = deserialize_tree(tree_bytes)?;

    // Read original data length
    let data_length = u32::from_be_bytes([
        data[4 + tree_size],
        data[4 + tree_size + 1],
        data[4 + tree_size + 2],
        data[4 + tree_size + 3],
    ]) as usize;

    // Decompress data
    let compressed_data = &data[8 + tree_size..];
    let mut reader = BitReader::new(compressed_data);
    let mut result = Vec::new();

    while result.len() < data_length {
        let mut node = &tree;
        loop {
            match node {
                HuffmanNode::Leaf { value, .. } => {
                    result.push(*value);
                    break;
                }
                HuffmanNode::Internal { left, right, .. } => {
                    let bit = match reader.read_bit() {
                        Ok(b) => b,
                        Err(_) => return Ok(result), // End of stream
                    };
                    node = if bit == 0 { left } else { right };
                }
            }
        }
    }

    Ok(result)
}
