// P2P PERFORMANCE BENCHMARK
use msscs_v4::{
    identity::QuantumIdentity,
    quantum_block::QuantumDataBlock,
    p2p_network::{P2PNode, P2PConfig},
    p2p_vfs::P2PVirtualFileSystem,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("msscs_v4=warn")
        .init();
    
    println!("⚡ P2P PERFORMANCE BENCHMARK");
    println!("=".repeat(80));
    
    // Setup
    let passphrase = "benchmark-passphrase";
    let identity = QuantumIdentity::new(passphrase)?;
    let unlocked = Arc::new(identity.unlock(passphrase)?);
    
    let config = P2PConfig::default();
    let mut node = P2PNode::new(config).await?;
    let _event_rx = node.take_event_receiver();
    node.start().await?;
    let node = Arc::new(RwLock::new(node));
    
    let vfs = P2PVirtualFileSystem::new(unlocked.clone(), node, 1024 * 64)?;
    
    // Benchmark 1: Small files (1KB)
    println!("\n📊 Benchmark 1: Small Files (1KB)");
    println!("-".repeat(80));
    let small_data = vec![42u8; 1024];
    
    let start = Instant::now();
    for i in 0..10 {
        let path = PathBuf::from(format!("small_{}.dat", i));
        vfs.upload_file(&path, &small_data).await?;
    }
    let duration = start.elapsed();
    
    println!("   Uploaded 10 files in {:?}", duration);
    println!("   Average: {:?} per file", duration / 10);
    println!("   Throughput: {:.2} KB/s", (10.0 * 1024.0) / duration.as_secs_f64() / 1024.0);
    
    // Benchmark 2: Medium files (100KB)
    println!("\n📊 Benchmark 2: Medium Files (100KB)");
    println!("-".repeat(80));
    let medium_data = vec![42u8; 100 * 1024];
    
    let start = Instant::now();
    for i in 0..5 {
        let path = PathBuf::from(format!("medium_{}.dat", i));
        vfs.upload_file(&path, &medium_data).await?;
    }
    let duration = start.elapsed();
    
    println!("   Uploaded 5 files in {:?}", duration);
    println!("   Average: {:?} per file", duration / 5);
    println!("   Throughput: {:.2} KB/s", (5.0 * 100.0 * 1024.0) / duration.as_secs_f64() / 1024.0);
    
    // Benchmark 3: Large files (1MB)
    println!("\n📊 Benchmark 3: Large Files (1MB)");
    println!("-".repeat(80));
    let large_data = vec![42u8; 1024 * 1024];
    
    let start = Instant::now();
    for i in 0..3 {
        let path = PathBuf::from(format!("large_{}.dat", i));
        vfs.upload_file(&path, &large_data).await?;
    }
    let duration = start.elapsed();
    
    println!("   Uploaded 3 files in {:?}", duration);
    println!("   Average: {:?} per file", duration / 3);
    println!("   Throughput: {:.2} MB/s", (3.0 * 1024.0 * 1024.0) / duration.as_secs_f64() / 1024.0 / 1024.0);
    
    // Benchmark 4: Download performance
    println!("\n📊 Benchmark 4: Download Performance");
    println!("-".repeat(80));
    
    let start = Instant::now();
    for i in 0..10 {
        let path = PathBuf::from(format!("small_{}.dat", i));
        let _ = vfs.download_file(&path).await?;
    }
    let duration = start.elapsed();
    
    println!("   Downloaded 10 files in {:?}", duration);
    println!("   Average: {:?} per file", duration / 10);
    
    // Benchmark 5: Encryption overhead
    println!("\n📊 Benchmark 5: Encryption Overhead");
    println!("-".repeat(80));
    let test_data = vec![42u8; 1024 * 1024];
    
    let start = Instant::now();
    for _ in 0..10 {
        let _ = QuantumDataBlock::new(
            &test_data,
            0,
            None,
            [0u8; 32],
            &unlocked,
            None,
        )?;
    }
    let duration = start.elapsed();
    
    println!("   Encrypted 10 blocks in {:?}", duration);
    println!("   Average: {:?} per block", duration / 10);
    println!("   Throughput: {:.2} MB/s", (10.0 * 1024.0 * 1024.0) / duration.as_secs_f64() / 1024.0 / 1024.0);
    
    // Summary
    println!("\n🎯 BENCHMARK SUMMARY");
    println!("=".repeat(80));
    println!("✅ All benchmarks completed successfully!");
    
    Ok(())
}
