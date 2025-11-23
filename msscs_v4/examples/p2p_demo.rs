// P2P Network Demo - Demonstrates global P2P storage network
use msscs_v4::{
    identity::QuantumIdentity,
    quantum_block::QuantumDataBlock,
    p2p_network::{P2PNode, P2PConfig, P2PEvent},
    erasure::ErasureCoding,
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("msscs_v4=info,p2p_demo=info")
        .init();
    
    println!("🌐 MSSCS v4 - Global P2P Storage Network Demo");
    println!("=".repeat(60));
    
    // Create user identity
    println!("\n📝 Step 1: Creating quantum-resistant identity");
    let passphrase = "demo-passphrase-secure-123";
    let identity = QuantumIdentity::new(passphrase)?;
    let unlocked = identity.unlock(passphrase)?;
    println!("   User ID: {}", unlocked.user_id());
    
    // Create P2P node
    println!("\n🌐 Step 2: Initializing P2P node");
    let config = P2PConfig {
        listen_addresses: vec![
            "/ip4/0.0.0.0/tcp/0".to_string(),
        ],
        bootstrap_peers: Vec::new(),
        max_peers: 50,
        enable_mdns: true,
        enable_relay: true,
        replication_factor: 3,
    };
    
    let mut node = P2PNode::new(config).await?;
    println!("   Peer ID: {}", node.peer_id());
    
    let mut event_rx = node.take_event_receiver();
    
    // Start the node
    println!("\n🚀 Step 3: Starting P2P node");
    node.start().await?;
    
    // Spawn event handler
    if let Some(mut event_rx) = event_rx {
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    P2PEvent::PeerDiscovered { peer_id, addresses } => {
                        println!("🔍 Discovered peer: {} ({} addresses)", peer_id, addresses.len());
                    }
                    P2PEvent::PeerConnected { peer_id } => {
                        println!("🤝 Connected to peer: {}", peer_id);
                    }
                    P2PEvent::PeerDisconnected { peer_id } => {
                        println!("👋 Disconnected from peer: {}", peer_id);
                    }
                    P2PEvent::ProvidersFound { block_id, providers } => {
                        println!("📍 Found {} providers for block {}", providers.len(), block_id);
                    }
                    _ => {}
                }
            }
        });
    }
    
    // Create and store a quantum block
    println!("\n💾 Step 4: Creating quantum-encrypted block");
    let data = b"Hello, P2P World! This is quantum-encrypted data distributed globally.";
    let block = QuantumDataBlock::new(
        data,
        0,
        None,
        [0u8; 32],
        &unlocked,
        Some("text/plain".to_string()),
    )?;
    
    println!("   Block UUID: {}", block.uuid);
    println!("   Block hash: {}", hex::encode(block.calculate_hash()));
    
    let stats = block.size_stats();
    stats.print_summary();
    
    // Serialize block
    let block_data = bincode::serialize(&block)?;
    let block_id = block.uuid.to_string();
    
    // Store block on P2P network
    println!("\n📡 Step 5: Storing block on P2P network");
    node.store_block(block_id.clone(), block_data.clone()).await?;
    
    // Apply erasure coding
    println!("\n🔀 Step 6: Applying erasure coding (10+4)");
    let erasure = ErasureCoding::new(10, 4)?;
    let shards = erasure.encode(&block_data)?;
    
    println!("   Created {} shards", shards.len());
    println!("   Can tolerate {} shard failures", erasure.max_failures());
    println!("   Storage overhead: {:.1}%", erasure.overhead_percentage());
    
    // Store shards on network
    for (i, shard) in shards.iter().enumerate() {
        let shard_id = format!("{}-shard-{}", block_id, i);
        let shard_data = bincode::serialize(shard)?;
        node.store_block(shard_id, shard_data).await?;
    }
    
    println!("   ✅ All shards stored on network");
    
    // Find providers
    println!("\n🔍 Step 7: Finding providers for block");
    node.find_providers(block_id.clone()).await?;
    
    // Wait for network activity
    println!("\n⏳ Waiting for network activity (10 seconds)...");
    sleep(Duration::from_secs(10)).await;
    
    // Show network stats
    println!("\n📊 Network Statistics:");
    println!("   Connected peers: {}", node.connected_peers_count().await);
    println!("   Peer ID: {}", node.peer_id());
    
    println!("\n✅ P2P Demo Complete!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   ✓ Quantum-resistant identity");
    println!("   ✓ Seven-layer encryption");
    println!("   ✓ P2P network with Kademlia DHT");
    println!("   ✓ Erasure coding (10+4)");
    println!("   ✓ Content-addressed storage");
    println!("   ✓ Distributed block storage");
    
    Ok(())
}
