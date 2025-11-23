// SIMPLE P2P TEST - Without API complexity
use msscs_v4::{
    identity::QuantumIdentity,
    quantum_block::QuantumDataBlock,
    p2p_vfs::P2PVirtualFileSystem,
    erasure::ErasureCoding,
    singularity::SingularityFragmentation,
};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("msscs_v4=info")
        .init();
    
    println!("🧪 SIMPLE P2P SYSTEM TEST");
    println!("=".repeat(80));
    
    // Create identity
    println!("\n📝 Creating quantum identity...");
    let passphrase = "test-passphrase";
    let identity = QuantumIdentity::new(passphrase)?;
    let unlocked = Arc::new(identity.unlock(passphrase)?);
    println!("✅ Identity: {}", unlocked.user_id());
    
    // Create VFS
    println!("\n💾 Creating P2P VFS...");
    let vfs = P2PVirtualFileSystem::new(unlocked.clone(), 64 * 1024)?;
    println!("✅ VFS created");
    
    // Upload file
    println!("\n📤 Uploading file...");
    let test_data = b"Hello from decentralized quantum storage!";
    let path = PathBuf::from("test.txt");
    let uuid = vfs.upload_file(&path, test_data).await?;
    println!("✅ File uploaded: {}", uuid);
    
    // Download file
    println!("\n📥 Downloading file...");
    let downloaded = vfs.download_file(&path).await?;
    assert_eq!(test_data.as_slice(), downloaded.as_slice());
    println!("✅ File downloaded and verified");
    
    // Stats
    let stats = vfs.get_stats().await;
    println!("\n📊 Statistics:");
    println!("   Files: {}", stats.total_files);
    println!("   Cached blocks: {}", stats.cached_blocks);
    
    // Test erasure coding
    println!("\n🔀 Testing erasure coding...");
    let erasure = ErasureCoding::new(10, 4)?;
    let data = vec![42u8; 1024];
    let shards = erasure.encode(&data)?;
    println!("✅ Created {} shards", shards.len());
    
    let reconstructed = erasure.decode(&shards[0..10])?;
    assert_eq!(data, reconstructed);
    println!("✅ Reconstruction successful");
    
    // Test singularity
    println!("\n🌀 Testing singularity fragmentation...");
    let singularity = SingularityFragmentation::new(3, 5)?;
    let fragments = singularity.fragment(&data)?;
    println!("✅ Created {} fragments", fragments.len());
    
    let reconstructed = singularity.reconstruct(&fragments[0..3])?;
    assert_eq!(data, reconstructed);
    println!("✅ Reconstruction successful");
    
    println!("\n🎉 ALL TESTS PASSED!");
    
    Ok(())
}
