// MSSCS NODE - Integrated P2P Storage Node with Full libp2p Stack
// This replaces the basic TCP implementation with production-ready P2P networking

use msscs_v4::{
    identity::QuantumIdentity,
    p2p_network::{P2PNode, P2PConfig},
    p2p_vfs::P2PVirtualFileSystem,
    p2p_api::{P2PAppState, create_p2p_router},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use clap::Parser;
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(name = "msscs-node")]
#[command(about = "MSSCS P2P Storage Node with Internet Connectivity", long_about = None)]
struct Args {
    /// HTTP API port
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    /// P2P listen port (0 = random)
    #[arg(short = 'l', long, default_value = "4001")]
    p2p_port: u16,
    
    /// Bootstrap peers (format: /ip4/addr/tcp/port/p2p/peer_id or ip:port)
    #[arg(short, long)]
    bootstrap: Vec<String>,
    
    /// User passphrase for identity
    #[arg(short = 'P', long)]
    passphrase: Option<String>,
    
    /// Enable mDNS local discovery
    #[arg(short, long, default_value = "true")]
    mdns: bool,
    
    /// Enable relay for NAT traversal
    #[arg(short, long, default_value = "true")]
    relay: bool,
    
    /// Public IP address (for NAT traversal)
    #[arg(long)]
    public_ip: Option<IpAddr>,
    
    /// Replication factor
    #[arg(short = 'r', long, default_value = "3")]
    replication: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("msscs_v4=info".parse()?)
                .add_directive("msscs_node=info".parse()?)
        )
        .init();
    
    let args = Args::parse();
    
    println!("\n{}", "=".repeat(70));
    println!("🌐 MSSCS P2P Storage Node - Internet-Ready Edition");
    println!("{}", "=".repeat(70));
    
    // Create or load identity
    let passphrase = args.passphrase.unwrap_or_else(|| {
        println!("⚠️  No passphrase provided, using default (INSECURE for production!)");
        "default-passphrase-change-me".to_string()
    });
    
    println!("\n🔐 Initializing quantum-resistant identity...");
    let identity = QuantumIdentity::new(&passphrase)?;
    let unlocked = Arc::new(identity.unlock(&passphrase)?);
    println!("   ✅ User ID: {}", unlocked.user_id());
    
    // Parse bootstrap peers
    let bootstrap_peers = parse_bootstrap_peers(&args.bootstrap);
    
    if bootstrap_peers.is_empty() && !args.mdns {
        println!("\n⚠️  WARNING: No bootstrap peers and mDNS disabled!");
        println!("   This node will not be able to discover other nodes.");
        println!("   Enable mDNS with --mdns or provide bootstrap peers with --bootstrap");
    }
    
    // Create P2P node configuration
    println!("\n🌐 Configuring P2P network...");
    let mut listen_addresses = vec![
        format!("/ip4/0.0.0.0/tcp/{}", args.p2p_port),
        format!("/ip6/::/tcp/{}", args.p2p_port),
    ];
    
    // Add public IP if provided
    if let Some(public_ip) = args.public_ip {
        match public_ip {
            IpAddr::V4(ip) => {
                listen_addresses.push(format!("/ip4/{}/tcp/{}", ip, args.p2p_port));
                println!("   ✅ Public IPv4: {}", ip);
            }
            IpAddr::V6(ip) => {
                listen_addresses.push(format!("/ip6/{}/tcp/{}", ip, args.p2p_port));
                println!("   ✅ Public IPv6: {}", ip);
            }
        }
    }
    
    let config = P2PConfig {
        listen_addresses,
        bootstrap_peers,
        max_peers: 50,
        enable_mdns: args.mdns,
        enable_relay: args.relay,
        replication_factor: args.replication,
    };
    
    println!("   ✅ Listen addresses: {} configured", config.listen_addresses.len());
    println!("   ✅ Bootstrap peers: {}", config.bootstrap_peers.len());
    println!("   ✅ mDNS: {}", if config.enable_mdns { "enabled" } else { "disabled" });
    println!("   ✅ Relay: {}", if config.enable_relay { "enabled" } else { "disabled" });
    println!("   ✅ Replication factor: {}", config.replication_factor);
    
    // Create P2P node
    println!("\n🚀 Starting P2P node...");
    let mut p2p_node = P2PNode::new(config).await?;
    let peer_id = p2p_node.peer_id();
    println!("   ✅ Peer ID: {}", peer_id);
    
    let mut event_rx = p2p_node.take_event_receiver();
    
    p2p_node.start().await?;
    
    // Bootstrap from known peers
    if !p2p_node.config.bootstrap_peers.is_empty() {
        println!("\n🔗 Bootstrapping from {} peers...", p2p_node.config.bootstrap_peers.len());
        p2p_node.bootstrap().await?;
    }
    
    let p2p_node = Arc::new(RwLock::new(p2p_node));
    
    // Spawn P2P event handler
    let p2p_clone = p2p_node.clone();
    tokio::spawn(async move {
        println!("\n📡 P2P event listener started");
        while let Some(event) = event_rx.recv().await {
            use msscs_v4::p2p_network::P2PEvent;
            match event {
                P2PEvent::PeerDiscovered { peer_id, addresses } => {
                    tracing::info!("🔍 Discovered peer: {} ({} addresses)", peer_id, addresses.len());
                }
                P2PEvent::PeerConnected { peer_id } => {
                    tracing::info!("🤝 Connected to peer: {}", peer_id);
                    // Update connected peers count
                    let node = p2p_clone.read().await;
                    let count = node.connected_peers_count().await;
                    tracing::info!("   Total connected peers: {}", count);
                }
                P2PEvent::PeerDisconnected { peer_id } => {
                    tracing::info!("👋 Disconnected from peer: {}", peer_id);
                }
                P2PEvent::BlockStored { block_id, peer_id: _ } => {
                    tracing::debug!("💾 Block stored: {}", block_id);
                }
                P2PEvent::ProvidersFound { block_id, providers } => {
                    tracing::debug!("📍 Found {} providers for block: {}", providers.len(), block_id);
                }
            }
        }
    });
    
    // Create P2P VFS
    println!("\n💾 Initializing P2P Virtual File System...");
    let vfs = Arc::new(P2PVirtualFileSystem::new(
        unlocked,
        p2p_node.clone(),
        1024 * 1024, // 1MB chunks
    )?);
    println!("   ✅ VFS ready with 1MB chunk size");
    
    // Create HTTP API
    println!("\n🌐 Starting HTTP API server...");
    let state = P2PAppState { vfs };
    let app = create_p2p_router(state);
    
    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    println!("\n{}", "=".repeat(70));
    println!("✅ MSSCS Node is ONLINE and ready for P2P connections!");
    println!("{}", "=".repeat(70));
    println!("\n📡 Network Information:");
    println!("   Peer ID: {}", peer_id);
    println!("   HTTP API: http://0.0.0.0:{}", args.port);
    println!("   P2P Port: {}", args.p2p_port);
    println!("   Connected Peers: 0 (waiting for connections...)");
    
    println!("\n📚 API Endpoints:");
    println!("   POST   /upload          - Upload file");
    println!("   GET    /download/:path  - Download file");
    println!("   DELETE /delete/:path    - Delete file");
    println!("   GET    /files           - List files");
    println!("   GET    /stats           - Node statistics");
    println!("   GET    /health          - Health check");
    
    println!("\n💡 Connection Information:");
    println!("   Share this with other nodes to connect:");
    println!("   --bootstrap \"/ip4/<your-ip>/tcp/{}/p2p/{}\"", args.p2p_port, peer_id);
    
    if args.mdns {
        println!("\n🔍 mDNS Discovery: ENABLED");
        println!("   Nodes on the same local network will discover each other automatically");
    }
    
    if args.relay {
        println!("\n🔄 Relay Support: ENABLED");
        println!("   NAT traversal and hole punching available");
    }
    
    println!("\n{}", "=".repeat(70));
    println!("Press Ctrl+C to stop the node");
    println!("{}", "=".repeat(70));
    
    // Start HTTP server
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Parse bootstrap peers from various formats
fn parse_bootstrap_peers(bootstrap_args: &[String]) -> Vec<(libp2p::PeerId, String)> {
    let mut peers = Vec::new();
    
    for arg in bootstrap_args {
        // Try to parse as multiaddr format: /ip4/addr/tcp/port/p2p/peer_id
        if arg.starts_with("/ip4/") || arg.starts_with("/ip6/") {
            // Extract peer_id from multiaddr
            if let Some(peer_id_str) = arg.split("/p2p/").nth(1) {
                if let Ok(peer_id) = peer_id_str.parse::<libp2p::PeerId>() {
                    peers.push((peer_id, arg.clone()));
                    tracing::info!("   ✅ Added bootstrap peer: {}", peer_id);
                } else {
                    tracing::warn!("   ⚠️  Invalid peer ID in: {}", arg);
                }
            } else {
                tracing::warn!("   ⚠️  No peer ID found in multiaddr: {}", arg);
            }
        } else {
            // Simple format: ip:port (generate random peer_id for now)
            tracing::warn!("   ⚠️  Simple format not supported yet: {}", arg);
            tracing::warn!("      Use multiaddr format: /ip4/addr/tcp/port/p2p/peer_id");
        }
    }
    
    peers
}
