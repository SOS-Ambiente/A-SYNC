// Main entry point
use clap::Parser;
use msscs_v4::{
    api::{create_router, AppState},
    config::Config,
    error::Result,
    metrics::Metrics,
    network::Node,
    persistence::PersistenceManager,
    vfs::VirtualFileSystem,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// MSSCS v4.0 - Multi-State Chain-based Secure Storage
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,
    
    /// Bootstrap peer address (can be specified multiple times)
    #[arg(short = 'b', long = "peer")]
    peers: Vec<String>,
    
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();
    
    // Load or create configuration
    let mut config = if args.config.exists() {
        Config::load(&args.config)?
    } else {
        let default_config = Config::default();
        default_config.save(&args.config)?;
        println!("Created default configuration at {:?}", args.config);
        default_config
    };
    
    // Override with CLI arguments
    if let Some(port) = args.port {
        config.port = port;
    }
    if !args.peers.is_empty() {
        config.bootstrap_peers.extend(args.peers);
    }
    
    config.validate()?;
    let config = Arc::new(config);
    
    // Initialize tracing
    let log_level = config.log_level.clone();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("msscs_v4={},tower_http=debug", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Starting MSSCS v4.0");
    tracing::info!("Configuration: {:?}", config);
    
    // Create data directories
    std::fs::create_dir_all(&config.data_dir)?;
    
    // Initialize persistence manager
    let persistence = Arc::new(PersistenceManager::new(config.data_dir.clone())?);
    tracing::info!("Persistence manager initialized");
    
    // Initialize VFS
    let mut vfs = VirtualFileSystem::new(config.clone(), persistence.clone())?;
    tracing::info!("VFS initialized");
    
    // Initialize network node
    let node = Arc::new(Node::new(config.clone()));
    tracing::info!("Network node initialized: {}", node.node_id);
    
    // Start DHT
    node.start_dht(config.bootstrap_peers.clone()).await?;
    tracing::info!("DHT started");
    
    // Start P2P listener
    let node_clone = node.clone();
    tokio::spawn(async move {
        if let Err(e) = node_clone.run_p2p_listener().await {
            tracing::error!("P2P listener error: {}", e);
        }
    });
    tracing::info!("P2P listener started on {}", node.addr);
    
    // Associate VFS with node
    vfs.set_node(node.clone());
    let vfs = Arc::new(RwLock::new(vfs));
    
    // Initialize metrics
    let metrics = Arc::new(Metrics::new());
    
    // Update initial metrics
    {
        let vfs_read = vfs.read().await;
        metrics.block_count.store(vfs_read.block_count(), std::sync::atomic::Ordering::Relaxed);
        metrics.storage_bytes.store(vfs_read.storage_bytes(), std::sync::atomic::Ordering::Relaxed);
        metrics.peer_count.store(config.bootstrap_peers.len(), std::sync::atomic::Ordering::Relaxed);
    }
    
    // Create API state
    let state = AppState {
        vfs,
        node,
        config: config.clone(),
        metrics,
    };
    
    // Create API router
    let app = create_router(state);
    
    // Start API server
    let api_addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting API server on {}", api_addr);
    
    let listener = tokio::net::TcpListener::bind(&api_addr).await?;
    tracing::info!("MSSCS v4.0 is ready!");
    tracing::info!("API available at http://{}", api_addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
