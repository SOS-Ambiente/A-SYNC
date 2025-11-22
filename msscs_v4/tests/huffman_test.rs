// Unit tests for Huffman compression module
use msscs_v4::huffman::{compress, decompress, serialize_tree, deserialize_tree, HuffmanNode};

#[test]
fn test_compression_decompression_roundtrip() {
    let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    
    let compressed = compress(&data).expect("Compression failed");
    let decompressed = decompress(&compressed).expect("Decompression failed");
    
    assert_eq!(data, decompressed);
}

#[test]
fn test_compression_with_repeated_data() {
    // Data with high redundancy should compress well
    let data = vec![0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2];
    
    let compressed = compress(&data).expect("Compression failed");
    let decompressed = decompress(&compressed).expect("Decompression failed");
    
    assert_eq!(data, decompressed);
}

#[test]
fn test_compression_with_single_symbol() {
    let data = vec![5, 5, 5, 5, 5];
    
    let compressed = compress(&data).expect("Compression failed");
    let decompressed = decompress(&compressed).expect("Decompression failed");
    
    assert_eq!(data, decompressed);
}

#[test]
fn test_compression_with_all_symbols() {
    // Use all Base-16 symbols
    let data: Vec<u8> = (0..16).cycle().take(100).collect();
    
    let compressed = compress(&data).expect("Compression failed");
    let decompressed = decompress(&compressed).expect("Decompression failed");
    
    assert_eq!(data, decompressed);
}

#[test]
fn test_empty_data_compression() {
    let data = vec![];
    
    let compressed = compress(&data).expect("Compression failed");
    let decompressed = decompress(&compressed).expect("Decompression failed");
    
    assert_eq!(data, decompressed);
}

#[test]
fn test_tree_serialization_deserialization() {
    // Create a simple tree
    let leaf1 = HuffmanNode::Leaf { value: 5, freq: 10 };
    let leaf2 = HuffmanNode::Leaf { value: 3, freq: 5 };
    let tree = HuffmanNode::Internal {
        freq: 15,
        left: Box::new(leaf1),
        right: Box::new(leaf2),
    };
    
    let serialized = serialize_tree(&tree);
    let deserialized = deserialize_tree(&serialized).expect("Deserialization failed");
    
    // Verify structure
    match deserialized {
        HuffmanNode::Internal { left, right, .. } => {
            match (*left, *right) {
                (HuffmanNode::Leaf { value: v1, .. }, HuffmanNode::Leaf { value: v2, .. }) => {
                    assert_eq!(v1, 5);
                    assert_eq!(v2, 3);
                }
                _ => panic!("Expected leaf nodes"),
            }
        }
        _ => panic!("Expected internal node"),
    }
}

#[test]
fn test_compression_ratio() {
    // Test with highly compressible data
    let data = vec![0; 1000]; // 1000 zeros
    
    let compressed = compress(&data).expect("Compression failed");
    
    // Compressed size should be significantly smaller
    println!("Original size: {} bytes", data.len());
    println!("Compressed size: {} bytes", compressed.len());
    println!("Compression ratio: {:.2}%", (compressed.len() as f64 / data.len() as f64) * 100.0);
    
    // Verify decompression
    let decompressed = decompress(&compressed).expect("Decompression failed");
    assert_eq!(data, decompressed);
}
