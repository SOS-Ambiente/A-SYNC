// COMPRESSION + ENCRYPTION INTEGRATION TEST
// Tests that demonstrate compression effectiveness and encryption overhead

use msscs_v4::{QuantumIdentity, QuantumDataBlock};
use rand::RngCore;

#[test]
fn test_small_data_encryption() {
    println!("\n🧪 Testing Small Data (< 100 bytes)");
    println!("{}", "=".repeat(60));
    
    let passphrase = "test passphrase";
    let identity = QuantumIdentity::new(passphrase).unwrap();
    let unlocked = identity.unlock(passphrase).unwrap();
    
    let data = b"Small test data";
    println!("Original size: {} bytes", data.len());
    
    let block = QuantumDataBlock::new(
        data,
        0,
        None,
        [0u8; 32],
        &unlocked,
        Some("text/plain".to_string()),
    ).unwrap();
    
    let stats = block.size_stats();
    stats.print_summary();
    
    // Decrypt and verify
    let decrypted = block.decode(&unlocked).unwrap();
    assert_eq!(data.as_slice(), decrypted.as_slice());
    
    println!("✅ Small data encryption test PASSED!");
}

#[test]
fn test_medium_data_encryption() {
    println!("\n🧪 Testing Medium Data (1KB)");
    println!("{}", "=".repeat(60));
    
    let passphrase = "test passphrase";
    let identity = QuantumIdentity::new(passphrase).unwrap();
    let unlocked = identity.unlock(passphrase).unwrap();
    
    // Create 1KB of repetitive data (compresses well)
    let data = "Hello World! ".repeat(85).into_bytes();
    println!("Original size: {} bytes", data.len());
    
    let block = QuantumDataBlock::new(
        &data,
        0,
        None,
        [0u8; 32],
        &unlocked,
        Some("text/plain".to_string()),
    ).unwrap();
    
    let stats = block.size_stats();
    stats.print_summary();
    
    // Decrypt and verify
    let decrypted = block.decode(&unlocked).unwrap();
    assert_eq!(data, decrypted);
    
    println!("✅ Medium data encryption test PASSED!");
    
    // Check compression effectiveness
    if stats.compressed_size < stats.original_size {
        println!("🎉 Compression saved {} bytes ({:.1}% reduction)",
            stats.original_size - stats.compressed_size,
            (1.0 - stats.compression_ratio()) * 100.0
        );
    }
}

#[test]
fn test_large_data_encryption() {
    println!("\n🧪 Testing Large Data (100KB)");
    println!("{}", "=".repeat(60));
    
    let passphrase = "test passphrase";
    let identity = QuantumIdentity::new(passphrase).unwrap();
    let unlocked = identity.unlock(passphrase).unwrap();
    
    // Create 100KB of repetitive data
    let data = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".repeat(2849).into_bytes();
    println!("Original size: {} bytes ({:.1} KB)", data.len(), data.len() as f64 / 1024.0);
    
    let start = std::time::Instant::now();
    let block = QuantumDataBlock::new(
        &data,
        0,
        None,
        [0u8; 32],
        &unlocked,
        Some("text/plain".to_string()),
    ).unwrap();
    let encrypt_time = start.elapsed();
    
    let stats = block.size_stats();
    stats.print_summary();
    
    println!("\n⏱️  Encryption time: {:?}", encrypt_time);
    
    let start = std::time::Instant::now();
    let decrypted = block.decode(&unlocked).unwrap();
    let decrypt_time = start.elapsed();
    
    println!("⏱️  Decryption time: {:?}", decrypt_time);
    
    assert_eq!(data, decrypted);
    
    println!("✅ Large data encryption test PASSED!");
}

#[test]
fn test_random_data_encryption() {
    println!("\n🧪 Testing Random Data (incompressible)");
    println!("{}", "=".repeat(60));
    
    let passphrase = "test passphrase";
    let identity = QuantumIdentity::new(passphrase).unwrap();
    let unlocked = identity.unlock(passphrase).unwrap();
    
    // Create 10KB of random data (won't compress)
    let mut data = vec![0u8; 10240];
    rand::rngs::OsRng.fill_bytes(&mut data);
    println!("Original size: {} bytes (random data)", data.len());
    
    let block = QuantumDataBlock::new(
        &data,
        0,
        None,
        [0u8; 32],
        &unlocked,
        Some("application/octet-stream".to_string()),
    ).unwrap();
    
    let stats = block.size_stats();
    stats.print_summary();
    
    // Decrypt and verify
    let decrypted = block.decode(&unlocked).unwrap();
    assert_eq!(data, decrypted);
    
    println!("✅ Random data encryption test PASSED!");
    
    // Random data should not compress
    if stats.compressed_size >= stats.original_size * 95 / 100 {
        println!("✅ Random data correctly identified as incompressible");
    }
}

#[test]
fn test_encryption_overhead() {
    println!("\n🧪 Analyzing Encryption Overhead");
    println!("{}", "=".repeat(60));
    
    let passphrase = "test passphrase";
    let identity = QuantumIdentity::new(passphrase).unwrap();
    let unlocked = identity.unlock(passphrase).unwrap();
    
    let sizes = vec![10, 100, 1000, 10000];
    
    println!("\n{:<15} {:<15} {:<15} {:<15}", "Original", "Compressed", "Encrypted", "Overhead");
    println!("{}", "-".repeat(60));
    
    for size in sizes {
        let data = vec![b'A'; size];
        
        let block = QuantumDataBlock::new(
            &data,
            0,
            None,
            [0u8; 32],
            &unlocked,
            None,
        ).unwrap();
        
        let stats = block.size_stats();
        let overhead = stats.encrypted_size as i64 - stats.original_size as i64;
        
        println!("{:<15} {:<15} {:<15} {:+<15}",
            stats.original_size,
            stats.compressed_size,
            stats.encrypted_size,
            overhead
        );
        
        // Verify decryption
        let decrypted = block.decode(&unlocked).unwrap();
        assert_eq!(data, decrypted);
    }
    
    println!("\n✅ Encryption overhead analysis complete!");
    println!("📝 Note: Encryption overhead includes:");
    println!("   - AES-GCM authentication tag (16 bytes)");
    println!("   - ChaCha20-Poly1305 authentication tag (16 bytes)");
    println!("   - Nonces (24 bytes total)");
    println!("   - Kyber-1024 ciphertext (~1568 bytes)");
    println!("   - Dilithium5 signature (~4595 bytes)");
    println!("   - Metadata and structure overhead");
}
