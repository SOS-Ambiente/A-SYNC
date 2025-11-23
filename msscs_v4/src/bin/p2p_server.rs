// P2P SERVER - Decentralized Storage Node
use msscs_v4::{
    identity::QuantumIdentity,
    p2p_network::{P2PNode, P2PConfig},
    p2p_vfs::P2PVirtualFileSystem,
    p2p_api::{P2PAppState, create_p2p_router},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "p2p-server")]
#[command(about = "MSSCS P2P Storage Node", long_about = None)]
struct Args {
    /// HTTP API port
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    /// P2P listen port
    #[arg(short = 'l', long, default_value = "0")]
    p2p_port: u16,
    
    /// Bootstrap peers (format: peer_id@/ip4/addr/tcp/port)
    #[arg(short, long)]
    bootstrap: Vec<String>,
    
    /// User passphrase
    #[arg(short = 'P', long)]
    passphrase: Option<String>,
    
    /// Enable mDNS local discovery
    #[arg(short, long, default_value = "true")]
    mdns: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("msscs_v4=info,p2p_server=info")
        .init();
    
    let args = Args::parse();
    
    println!("🌐 MSSCS P2P Storage Node");
    println!("{}", "=".repeat(60));
    
    // Create or load identity
    let passphrase = args.passphrase.unwrap_or_else(|| {
        println!("⚠️  No passphrase provided, using default (INSECURE!)");
        "default-passphrase-change-me".to_string()
    });
    
    println!("\n🔐 Creating quantum-resistant identity...");
    let identity = QuantumIdentity::new(&passphrase)?;
    let unlocked = Arc::new(identity.unlock(&passphrase)?);
    println!("   User ID: {}", unlocked.user_id());
    
    // Parse bootstrap peers
    let bootstrap_peers = Vec::new(); // Simplified for now
    
    // Create P2P node
    println!("\n🌐 Initializing P2P network...");
    let config = P2PConfig {
        listen_addresses: vec![
            format!("/ip4/0.0.0.0/tcp/{}", args.p2p_port),
        ],
        bootstrap_peers,
        max_peers: 50,
        enable_mdns: args.mdns,
        enable_relay: true,
        replication_factor: 3,
    };
    
    let mut p2p_node = P2PNode::new(config).await?;
    println!("   Peer ID: {}", p2p_node.peer_id());
    
    let mut event_rx = p2p_node.take_event_receiver();
    
    p2p_node.start().await?;
    
    let p2p_node = Arc::new(RwLock::new(p2p_node));
    
    // Spawn P2P event handler
    let p2p_clone = p2p_node.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            use msscs_v4::p2p_network::P2PEvent;
            match event {
                P2PEvent::PeerDiscovered { peer_id, .. } => {
                    tracing::info!("🔍 Discovered peer: {}", peer_id);
                }
                P2PEvent::PeerConnected { peer_id } => {
                    tracing::info!("🤝 Connected to peer: {}", peer_id);
                }
                P2PEvent::PeerDisconnected { peer_id } => {
                    tracing::info!("👋 Disconnected from peer: {}", peer_id);
                }
                _ => {}
            }
        }
    });
    
    // Spawn P2P node runner
    let p2p_runner = p2p_clone.clone();
    tokio::spawn(async move {
        let node = p2p_runner.write().await;
        // Note: This will consume the node, so we need to handle this differently
        // For now, we'll skip the runner and rely on event handling
    });
    
    // Create P2P VFS
    println!("\n💾 Initializing P2P Virtual File System...");
    let vfs = Arc::new(P2PVirtualFileSystem::new(
        unlocked,
        p2p_clone,
        1024 * 1024, // 1MB chunks
    )?);
    
    // Create API
    println!("\n🚀 Starting HTTP API server...");
    let state = P2PAppState { vfs };
    let app = create_p2p_router(state);
    
    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("   HTTP API: http://{}", addr);
    println!("\n✅ P2P node running!");
    println!("\n📡 API Endpoints:");
    println!("   POST   /upload          - Upload file");
    println!("   GET    /download/:path  - Download file");
    println!("   DELETE /delete/:path    - Delete file");
    println!("   GET    /files           - List files");
    println!("   GET    /stats           - Node statistics");
    println!("   GET    /health          - Health check");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
