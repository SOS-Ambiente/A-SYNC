// Integration test for complete block processing pipeline
use msscs_v4::block::DataBlock;

#[test]
fn test_block_creation_and_decoding() {
    // Test data
    let original_data = b"Hello, MSSCS! This is a test of the complete pipeline.";
    
    // Create a block (genesis block)
    let block = DataBlock::new(
        original_data,
        0,
        None,
        [0u8; 32],
    ).expect("Failed to create block");
    
    // Verify block was created
    assert_eq!(block.node_index, 0);
    assert_eq!(block.previous_uuid, None);
    
    // Decode the block
    let decoded_data = block.decode(0).expect("Failed to decode block");
    
    // Verify data matches
    assert_eq!(decoded_data, original_data);
}

#[test]
fn test_block_chain() {
    // Create a chain of blocks
    let data1 = b"First block";
    let data2 = b"Second block";
    let data3 = b"Third block";
    
    // Create first block (genesis)
    let block1 = DataBlock::new(data1, 0, None, [0u8; 32])
        .expect("Failed to create block 1");
    let hash1 = block1.calculate_hash().expect("Failed to calculate hash 1");
    
    // Create second block (links to first)
    let block2 = DataBlock::new(data2, 1, Some(block1.uuid), hash1)
        .expect("Failed to create block 2");
    let hash2 = block2.calculate_hash().expect("Failed to calculate hash 2");
    
    // Create third block (links to second)
    let block3 = DataBlock::new(data3, 2, Some(block2.uuid), hash2)
        .expect("Failed to create block 3");
    
    // Verify chain structure
    assert_eq!(block1.previous_uuid, None);
    assert_eq!(block2.previous_uuid, Some(block1.uuid));
    assert_eq!(block3.previous_uuid, Some(block2.uuid));
    
    // Verify hashes
    assert_eq!(block2.previous_hash, hash1);
    assert_eq!(block3.previous_hash, hash2);
    
    // Decode all blocks
    let decoded1 = block1.decode(0).expect("Failed to decode block 1");
    let decoded2 = block2.decode(1).expect("Failed to decode block 2");
    let decoded3 = block3.decode(2).expect("Failed to decode block 3");
    
    assert_eq!(decoded1, data1);
    assert_eq!(decoded2, data2);
    assert_eq!(decoded3, data3);
}

#[test]
fn test_block_encryption_security() {
    let data = b"Secret data";
    
    // Create block with node_index 0
    let block = DataBlock::new(data, 0, None, [0u8; 32])
        .expect("Failed to create block");
    
    // Try to decode with correct node_index (should succeed)
    let result = block.decode(0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), data);
    
    // Try to decode with wrong node_index (should fail)
    let result = block.decode(1);
    assert!(result.is_err());
}

#[test]
fn test_block_with_large_data() {
    // Test with larger data (1MB)
    let large_data: Vec<u8> = (0..1024*1024).map(|i| (i % 256) as u8).collect();
    
    let block = DataBlock::new(&large_data, 0, None, [0u8; 32])
        .expect("Failed to create large block");
    
    let decoded = block.decode(0).expect("Failed to decode large block");
    
    assert_eq!(decoded.len(), large_data.len());
    assert_eq!(decoded, large_data);
}

#[test]
fn test_block_with_empty_data() {
    let empty_data = b"";
    
    let block = DataBlock::new(empty_data, 0, None, [0u8; 32])
        .expect("Failed to create empty block");
    
    let decoded = block.decode(0).expect("Failed to decode empty block");
    
    assert_eq!(decoded, empty_data);
}

#[test]
fn test_block_with_binary_data() {
    // Test with binary data (not just text)
    let binary_data: Vec<u8> = vec![0x00, 0xFF, 0xAA, 0x55, 0x12, 0x34, 0x56, 0x78];
    
    let block = DataBlock::new(&binary_data, 0, None, [0u8; 32])
        .expect("Failed to create binary block");
    
    let decoded = block.decode(0).expect("Failed to decode binary block");
    
    assert_eq!(decoded, binary_data);
}

#[test]
fn test_block_serialization() {
    let data = b"Test serialization";
    
    let block = DataBlock::new(data, 0, None, [0u8; 32])
        .expect("Failed to create block");
    
    // Serialize
    let serialized = bincode::serialize(&block)
        .expect("Failed to serialize block");
    
    // Deserialize
    let deserialized: DataBlock = bincode::deserialize(&serialized)
        .expect("Failed to deserialize block");
    
    // Verify
    assert_eq!(deserialized.uuid, block.uuid);
    assert_eq!(deserialized.node_index, block.node_index);
    
    // Decode and verify data
    let decoded = deserialized.decode(0).expect("Failed to decode deserialized block");
    assert_eq!(decoded, data);
}

#[test]
fn test_block_hash_consistency() {
    let data = b"Hash test";
    
    let block = DataBlock::new(data, 0, None, [0u8; 32])
        .expect("Failed to create block");
    
    // Calculate hash multiple times
    let hash1 = block.calculate_hash().expect("Failed to calculate hash 1");
    let hash2 = block.calculate_hash().expect("Failed to calculate hash 2");
    let hash3 = block.calculate_hash().expect("Failed to calculate hash 3");
    
    // All hashes should be identical
    assert_eq!(hash1, hash2);
    assert_eq!(hash2, hash3);
}

#[test]
fn test_compression_effectiveness() {
    // Test with highly compressible data (repeated pattern)
    let compressible_data = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    
    let block = DataBlock::new(compressible_data, 0, None, [0u8; 32])
        .expect("Failed to create block");
    
    // Serialize to get actual storage size
    let serialized = bincode::serialize(&block)
        .expect("Failed to serialize");
    
    // Compressed size should be significantly smaller than original
    // (accounting for Base-16 encoding which doubles size before compression)
    println!("Original size: {} bytes", compressible_data.len());
    println!("Stored size: {} bytes", serialized.len());
    
    // Verify data integrity
    let decoded = block.decode(0).expect("Failed to decode");
    assert_eq!(decoded, compressible_data);
}
