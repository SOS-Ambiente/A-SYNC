// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod firewall;

use msscs_v4::{
    config::Config,
    metrics::Metrics,
    network::Node,
    persistence::PersistenceManager,
    vfs::VirtualFileSystem,
    p2p_network::{P2PNode, P2PConfig, P2PNodeCommand},
    workspace::{WorkspaceManager, Workspace, Permission},
    p2p_storage::P2PStorageManager,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::{RwLock, mpsc};

// Application state
struct AppStateWrapper {
    vfs: Arc<RwLock<VirtualFileSystem>>,
    node: Arc<Node>,
    p2p_command_tx: Option<mpsc::UnboundedSender<P2PNodeCommand>>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    workspace_manager: Arc<WorkspaceManager>,
    storage_manager: Arc<P2PStorageManager>,
    current_user_id: uuid::Uuid,
    current_user_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileUploadResult {
    uuid: String,
    blocks: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileInfo {
    path: String,
    size: u64,
    blocks: usize,
    uuid: String,
    synced: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeMetrics {
    block_count: usize,
    storage_bytes: u64,
    peer_count: usize,
    uptime_seconds: u64,
    requests_total: u64,
    requests_failed: u64,
    success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PeerInfo {
    id: String,
    address: String,
    status: String,
    blocks: usize,
    latency: u64,
}

// Tauri commands
#[tauri::command]
async fn get_storage_limit() -> Result<u64, String> {
    // Get from config or default to 10GB
    let app_data_dir = get_app_data_dir();
    let config_path = app_data_dir.join("storage_limit.txt");
    
    if config_path.exists() {
        let limit_str = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let limit_mb: u64 = limit_str.trim().parse().unwrap_or(10240);
        Ok(limit_mb * 1024 * 1024) // Convert MB to bytes
    } else {
        Ok(10 * 1024 * 1024 * 1024) // Default 10GB
    }
}

#[tauri::command]
async fn set_storage_limit(limit_mb: u64) -> Result<(), String> {
    let app_data_dir = get_app_data_dir();
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    
    let config_path = app_data_dir.join("storage_limit.txt");
    std::fs::write(&config_path, limit_mb.to_string()).map_err(|e| e.to_string())?;
    
    tracing::info!("📊 Storage limit set to {} MB", limit_mb);
    Ok(())
}

fn get_app_data_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("MSSCS")
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("MSSCS")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".msscs")
    }
}

#[tauri::command]
async fn start_node(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<(), String> {
    tracing::info!("🚀 start_node command called");
    
    let mut state_guard = state.write().await;
    
    if state_guard.is_some() {
        tracing::info!("✅ Node already running");
        return Ok(()); // Already started
    }
    
    tracing::info!("⏳ Starting node initialization...");

    // Get app data directory (platform-specific)
    let app_data_dir = get_app_data_dir();

    // Create app data directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;

    // Load or create configuration
    let config_path = app_data_dir.join("config.toml");
    let config = if config_path.exists() {
        Config::load(&config_path).map_err(|e| e.to_string())?
    } else {
        let mut default_config = Config::default();
        // Set data directory to app data location
        default_config.data_dir = app_data_dir.join("data");
        default_config.save(&config_path).map_err(|e| e.to_string())?;
        default_config
    };
    let config = Arc::new(config);

    // Initialize persistence
    std::fs::create_dir_all(&config.data_dir).map_err(|e| e.to_string())?;
    let persistence = Arc::new(
        PersistenceManager::new(config.data_dir.clone()).map_err(|e| e.to_string())?
    );

    // Initialize VFS
    let mut vfs = VirtualFileSystem::new(config.clone(), persistence.clone())
        .map_err(|e| e.to_string())?;

    // Initialize network node (legacy TCP)
    let node = Arc::new(Node::new(config.clone()));
    
    // Start DHT
    node.start_dht(config.bootstrap_peers.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Start P2P listener
    let node_clone = node.clone();
    tokio::spawn(async move {
        if let Err(e) = node_clone.run_p2p_listener().await {
            eprintln!("P2P listener error: {}", e);
        }
    });

    // Get storage limit from config (user-configurable)
    let storage_limit_bytes = get_storage_limit().await.unwrap_or(10 * 1024 * 1024 * 1024); // Default 10GB
    
    tracing::info!("╔════════════════════════════════════════════════════════════════╗");
    tracing::info!("║  💾 Storage Configuration                                     ║");
    tracing::info!("╚════════════════════════════════════════════════════════════════╝");
    tracing::info!("   • User allocation: {} MB ({:.2} GB)", 
        storage_limit_bytes / (1024 * 1024),
        storage_limit_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    tracing::info!("   • This space will be used for:");
    tracing::info!("     - Your encrypted files");
    tracing::info!("     - Cached blocks from other peers");
    tracing::info!("     - Network contribution (helping others)");
    tracing::info!("");
    
    // CRITICAL FIX: Initialize P2P in background to avoid blocking
    tracing::info!("╔════════════════════════════════════════════════════════════════╗");
    tracing::info!("║  🌍 Initializing Global P2P Network (Internet-Wide)          ║");
    tracing::info!("╚════════════════════════════════════════════════════════════════╝");
    tracing::info!("");
    tracing::info!("📊 Storage Configuration:");
    tracing::info!("   • Allocated space: {} MB ({:.2} GB)", 
        storage_limit_bytes / (1024 * 1024),
        storage_limit_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    );
    tracing::info!("");
    
    let p2p_config = P2PConfig {
        listen_port: 0,
        bootstrap_peers: P2PConfig::default_bootstrap_peers(),
        max_peers: 50,
        replication_factor: config.replication_factor,
        enable_mdns: true,
        enable_relay: true,
        enable_autonat: true,
    };
    
    tracing::info!("📡 Network Configuration:");
    tracing::info!("   • Bootstrap peers: {} (IPFS public nodes)", p2p_config.bootstrap_peers.len());
    tracing::info!("   • Max peer connections: {}", p2p_config.max_peers);
    tracing::info!("   • Data replication: {}x (fault tolerance)", p2p_config.replication_factor);
    tracing::info!("");
    
    // CRITICAL FIX: Start P2P initialization in background (non-blocking)
    let p2p_command_tx = {
        tracing::info!("🔄 Starting P2P node initialization (background)...");
        tracing::info!("   This will not block the UI - node will be ready shortly");
        
        // Start P2P in background without blocking
        let (init_tx, mut init_rx) = mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            // Create P2P node (this is fast - just setup)
            match P2PNode::new(p2p_config.clone()).await {
                Ok(p2p_node) => {
                    let cmd_tx = p2p_node.get_command_sender();
                    
                    // Send command sender back immediately
                    let _ = init_tx.send(Some(cmd_tx.clone()));
                    
                    tracing::info!("✅ P2P node created, starting event loop...");
                    
                    // Start node event loop (this handles bootstrap in background)
                    match p2p_node.start(p2p_config.clone()).await {
                        Ok((mut event_rx, local_blocks)) => {
                            tracing::info!("✅ P2P event loop started");
                            tracing::info!("   Bootstrap will complete in 10-30 seconds");
                            
                            let mut peer_count: usize = 0;
                            let mut bootstrap_complete = false;
                            
                            // Handle events
                            while let Some(event) = event_rx.recv().await {
                                use msscs_v4::p2p_network::P2PEvent;
                                match event {
                                    P2PEvent::PeerConnected(peer_id) => {
                                        peer_count += 1;
                                        tracing::info!("✅ Connected to peer: {} (total: {})", peer_id, peer_count);
                                    }
                                    P2PEvent::PeerDisconnected(peer_id) => {
                                        peer_count = peer_count.saturating_sub(1);
                                        tracing::info!("❌ Disconnected from peer: {} (total: {})", peer_id, peer_count);
                                    }
                                    P2PEvent::BlockReceived { peer, block } => {
                                        tracing::info!("📦 Received block {} from {}", block.uuid, peer);
                                        local_blocks.write().await.insert(block.uuid, block);
                                    }
                                    P2PEvent::BlockRequested { peer, block_id } => {
                                        tracing::debug!("📤 Block {} requested by {}", block_id, peer);
                                    }
                                    P2PEvent::BootstrapComplete => {
                                        if !bootstrap_complete {
                                            bootstrap_complete = true;
                                            tracing::info!("🎉 DHT bootstrap complete! Connected to global network");
                                        }
                                    }
                                    P2PEvent::Error(err) => {
                                        tracing::warn!("⚠️  P2P error: {}", err);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("❌ P2P node start failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️  P2P initialization failed: {}", e);
                    tracing::warn!("   Running without libp2p P2P features");
                    let _ = init_tx.send(None);
                }
            }
        });
        
        // Wait briefly for P2P to initialize (but don't block too long)
        match tokio::time::timeout(std::time::Duration::from_secs(2), init_rx.recv()).await {
            Ok(Some(Some(cmd_tx))) => {
                tracing::info!("✅ P2P node initialized (bootstrap continuing in background)");
                Some(cmd_tx)
            }
            Ok(Some(None)) | Ok(None) | Err(_) => {
                tracing::warn!("⚠️  P2P initialization taking longer than expected");
                tracing::warn!("   Running without libp2p P2P features for now");
                None
            }
        }
    };

    // Associate VFS with node
    vfs.set_node(node.clone());
    
    // Log P2P integration status
    if p2p_command_tx.is_some() {
        tracing::info!("✅ VFS configured with P2P network integration (libp2p + PeerJS)");
        tracing::info!("   • libp2p: Global DHT, NAT traversal, relay");
        tracing::info!("   • PeerJS: WebRTC bridge for web clients");
        tracing::info!("   • Legacy TCP: Direct peer connections");
    } else {
        tracing::warn!("⚠️  VFS running without libp2p P2P (PeerJS + Legacy TCP only)");
    }
    
    let vfs = Arc::new(RwLock::new(vfs));

    // Initialize metrics
    let metrics = Arc::new(Metrics::new());
    
    // Initialize workspace and storage managers
    let workspace_manager = Arc::new(WorkspaceManager::new());
    
    // Generate user ID and email (in production, get from identity)
    let current_user_id = uuid::Uuid::new_v4();
    let current_user_email = format!("user-{}@msscs.local", current_user_id);
    
    let peer_id = format!("peer-{}", current_user_id);
    let storage_manager = Arc::new(P2PStorageManager::new(peer_id, current_user_id));

    *state_guard = Some(AppStateWrapper {
        vfs,
        node,
        p2p_command_tx,
        config,
        metrics,
        workspace_manager,
        storage_manager,
        current_user_id,
        current_user_email,
    });

    tracing::info!("╔════════════════════════════════════════════════════════════════╗");
    tracing::info!("║  ✅ MSSCS Tauri Node Successfully Started                     ║");
    tracing::info!("╚════════════════════════════════════════════════════════════════╝");
    tracing::info!("");
    tracing::info!("🎉 Node is ready to use!");
    tracing::info!("   • VFS initialized");
    tracing::info!("   • P2P network active");
    tracing::info!("   • Storage configured");
    tracing::info!("   • Workspace system ready");
    tracing::info!("");

    Ok(())
}

#[tauri::command]
async fn wait_for_node_ready(
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
    window: tauri::Window,
) -> Result<(), String> {
    tracing::info!("🔄 wait_for_node_ready called");
    
    // CRITICAL FIX: Check immediately first (node might already be ready)
    {
        let state_guard = state.read().await;
        if state_guard.is_some() {
            tracing::info!("✅ Node is already ready (immediate check)");
            // Emit event to frontend
            let _ = window.emit("node-ready", ());
            return Ok(());
        }
    }
    
    // Poll until node is ready (with timeout)
    let max_attempts = 30; // 30 seconds max
    for attempt in 0..max_attempts {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let state_guard = state.read().await;
        if state_guard.is_some() {
            tracing::info!("✅ Node is ready after {} attempts ({} ms)", attempt + 1, (attempt + 1) * 500);
            
            // Emit event to frontend
            let _ = window.emit("node-ready", ());
            return Ok(());
        }
        drop(state_guard);
    }
    
    tracing::error!("❌ Node failed to start within timeout (15 seconds)");
    Err("Node initialization timeout".to_string())
}

#[tauri::command]
async fn list_files(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<Vec<FileInfo>, String> {
    tracing::debug!("📋 list_files command called");
    
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or_else(|| {
        tracing::error!("❌ Node not started - cannot list files");
        "Node not started. Please wait for initialization.".to_string()
    })?;
    
    let vfs = app_state.vfs.read().await;
    
    // Get all files from manifest
    let file_infos: Vec<FileInfo> = vfs.file_manifest.iter().map(|(path, uuid)| {
        // Count blocks for this file
        let blocks = vfs.local_blocks.values()
            .filter(|b| b.uuid == *uuid)
            .count();
        
        // Estimate size (blocks are typically 256KB each)
        let size = (blocks * 256 * 1024) as u64;
        
        FileInfo {
            path: path.clone(),
            size,
            blocks,
            uuid: uuid.to_string(),
            synced: true,
        }
    }).collect();
    
    tracing::debug!("✅ Returning {} files", file_infos.len());
    Ok(file_infos)
}

#[tauri::command]
async fn upload_file(
    path: String,
    window: tauri::Window,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<FileUploadResult, String> {
    tracing::info!("📤 upload_file command called: {}", path);
    
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or_else(|| {
        tracing::error!("❌ Node not started - cannot upload file");
        "Node not started. Please wait for initialization.".to_string()
    })?;

    // Read file
    let data = std::fs::read(&path).map_err(|e| {
        tracing::error!("❌ Failed to read file {}: {}", path, e);
        format!("Failed to read file: {}", e)
    })?;
    
    let file_name = PathBuf::from(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            tracing::error!("❌ Invalid file path: {}", path);
            "Invalid file path".to_string()
        })?
        .to_string();

    let total_size = data.len();
    tracing::info!("📊 File size: {} bytes ({} MB)", total_size, total_size / (1024 * 1024));

    // Emit initial progress
    let _ = window.emit("upload-progress", serde_json::json!({
        "file": file_name,
        "progress": 0,
        "current": 0,
        "total": 100,
        "status": "starting"
    }));

    let start_time = std::time::Instant::now();
    let mut last_emit = std::time::Instant::now();

    // Upload to VFS with progress tracking
    let uuid = {
        let mut vfs = app_state.vfs.write().await;
        let window_clone = window.clone();
        let file_name_clone = file_name.clone();
        
        vfs.write_file_with_progress(&PathBuf::from(&file_name), &data, move |current, total| {
            // Throttle emissions to every 100ms for better performance
            if last_emit.elapsed().as_millis() < 100 && current < total {
                return;
            }
            
            let progress = (current as f64 / total as f64 * 100.0) as u32;
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { (current as f64 / elapsed) as u64 } else { 0 };
            let eta = if speed > 0 { ((total - current) as f64 / speed as f64) as u64 } else { 0 };
            
            let _ = window_clone.emit("upload-progress", serde_json::json!({
                "file": file_name_clone,
                "progress": progress,
                "current": current,
                "total": total,
                "speed": speed,
                "eta": eta,
                "status": "uploading"
            }));
            
            last_emit = std::time::Instant::now();
        })
        .await
        .map_err(|e| {
            tracing::error!("❌ Upload failed: {}", e);
            format!("Upload failed: {}", e)
        })?
    };

    // Emit completion
    let _ = window.emit("upload-progress", serde_json::json!({
        "file": file_name,
        "progress": 100,
        "complete": true,
        "status": "complete"
    }));

    tracing::info!("✅ Upload complete: {} (UUID: {})", file_name, uuid);
    
    Ok(FileUploadResult {
        uuid: uuid.to_string(),
        blocks: 0, // Will be calculated by VFS
    })
}

#[tauri::command]
async fn download_file(
    path: String,
    output_path: String,
    window: tauri::Window,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    tracing::info!("📥 download_file command called: {} -> {}", path, output_path);
    
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or_else(|| {
        tracing::error!("❌ Node not started - cannot download file");
        "Node not started. Please wait for initialization.".to_string()
    })?;

    // Emit initial progress
    let _ = window.emit("download-progress", serde_json::json!({
        "file": path,
        "progress": 0,
        "current": 0,
        "total": 100,
        "status": "starting"
    }));

    let start_time = std::time::Instant::now();
    let mut last_emit = std::time::Instant::now();

    // Read from VFS with progress tracking
    let data = {
        let mut vfs = app_state.vfs.write().await;
        let window_clone = window.clone();
        let path_clone = path.clone();
        
        vfs.read_file_with_progress(&PathBuf::from(&path), move |current, total| {
                // Throttle emissions to every 100ms for better performance
                if last_emit.elapsed().as_millis() < 100 && current < total {
                    return;
                }
                
                let progress = (current as f64 / total as f64 * 100.0) as u32;
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 { (current as f64 / elapsed) as u64 } else { 0 };
                let eta = if speed > 0 { ((total - current) as f64 / speed as f64) as u64 } else { 0 };
                
                let _ = window_clone.emit("download-progress", serde_json::json!({
                    "file": path_clone,
                    "progress": progress,
                    "current": current,
                    "total": total,
                    "speed": speed,
                    "eta": eta,
                    "status": "downloading"
                }));
                
                last_emit = std::time::Instant::now();
            })
            .await
            .map_err(|e| {
                tracing::error!("❌ Download failed: {}", e);
                format!("Download failed: {}", e)
            })?
    };

    // Write to disk
    std::fs::write(&output_path, data).map_err(|e| {
        tracing::error!("❌ Failed to write file {}: {}", output_path, e);
        format!("Failed to write file: {}", e)
    })?;

    // Emit completion
    let _ = window.emit("download-progress", serde_json::json!({
        "file": path,
        "progress": 100,
        "complete": true,
        "status": "complete"
    }));

    tracing::info!("✅ Download complete: {}", path);
    Ok(())
}

#[tauri::command]
async fn delete_file(
    path: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    let mut vfs = app_state.vfs.write().await;
    vfs.delete_file(&PathBuf::from(&path))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_metrics(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<NodeMetrics, String> {
    // CRITICAL FIX: Use a shorter timeout for the read lock to avoid blocking
    let state_guard = match tokio::time::timeout(
        std::time::Duration::from_millis(200),
        state.read()
    ).await {
        Ok(guard) => guard,
        Err(_) => {
            return Err("Node busy".to_string());
        }
    };
    
    let app_state = state_guard.as_ref().ok_or_else(|| {
        "Node not started".to_string()
    })?;

    let snapshot = app_state.metrics.snapshot();
    
    // Get P2P peer count if available (with shorter timeout to avoid blocking)
    let p2p_peer_count = if let Some(ref cmd_tx) = app_state.p2p_command_tx {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        if cmd_tx.send(P2PNodeCommand::GetConnectedPeers(reply_tx)).is_ok() {
            match tokio::time::timeout(std::time::Duration::from_millis(100), reply_rx).await {
                Ok(Ok(peers)) => peers.len(),
                _ => 0
            }
        } else {
            0
        }
    } else {
        0
    };
    
    // Combine legacy and P2P peer counts
    let total_peer_count = snapshot.peer_count + p2p_peer_count;
    
    // Get storage stats (with timeout to prevent hanging)
    let storage_used: u64 = match tokio::time::timeout(
        std::time::Duration::from_millis(100),
        async {
            let vfs = app_state.vfs.read().await;
            let all_metadata = vfs.get_all_metadata();
            all_metadata.values().map(|m| m.size).sum()
        }
    ).await {
        Ok(size) => size,
        Err(_) => 0
    };
    
    Ok(NodeMetrics {
        block_count: snapshot.block_count,
        storage_bytes: storage_used,
        peer_count: total_peer_count,
        uptime_seconds: snapshot.uptime_seconds,
        requests_total: snapshot.requests_total,
        requests_failed: snapshot.requests_failed,
        success_rate: snapshot.success_rate,
    })
}

#[tauri::command]
async fn get_storage_stats(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<StorageStats, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    // Get VFS stats first
    let vfs = app_state.vfs.read().await;
    let total_files = vfs.file_manifest.len();
    let cached_blocks = vfs.local_blocks.len();
    drop(vfs);
    
    // Get P2P peer count (libp2p)
    let p2p_peer_count = if let Some(ref cmd_tx) = app_state.p2p_command_tx {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        if cmd_tx.send(P2PNodeCommand::GetConnectedPeers(reply_tx)).is_ok() {
            reply_rx.await.map(|peers| peers.len()).unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    
    // Get legacy TCP peer count
    let legacy_peer_count = app_state.node.peers.read().await.len();
    
    // Total peer count (libp2p + legacy)
    let total_peers = p2p_peer_count + legacy_peer_count;
    
    // Calculate storage stats
    let storage_limit: usize = 100 * 1024 * 1024; // 100MB default
    let storage_used: usize = cached_blocks * 256 * 1024; // Estimate 256KB per block
    let storage_available: usize = storage_limit.saturating_sub(storage_used);
    
    // Calculate global network storage (estimate based on peer contributions)
    let estimated_global_storage = if total_peers > 0 {
        storage_limit * (total_peers + 1) // +1 for self
    } else {
        storage_limit
    };
    
    tracing::debug!("📊 Storage Stats: {} files, {} used / {} limit, {} peers (libp2p: {}, legacy: {}), ~{} MB global", 
        total_files,
        storage_used,
        storage_limit,
        total_peers,
        p2p_peer_count,
        legacy_peer_count,
        estimated_global_storage / (1024 * 1024)
    );
    
    Ok(StorageStats {
        total_files,
        storage_used,
        storage_limit,
        storage_available,
        connected_peers: total_peers,
        global_storage_estimate: estimated_global_storage,
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageStats {
    total_files: usize,
    storage_used: usize,
    storage_limit: usize,
    storage_available: usize,
    connected_peers: usize,
    global_storage_estimate: usize,
}

#[tauri::command]
async fn preview_file(
    path: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<String, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    // Read from VFS
    let mut vfs = app_state.vfs.write().await;
    let data = vfs
        .read_file(&PathBuf::from(&path))
        .await
        .map_err(|e| e.to_string())?;

    // Convert to base64 for images/videos or UTF-8 for text
    let ext = PathBuf::from(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ["jpg", "jpeg", "png", "gif", "webp", "mp4", "webm"].contains(&ext.as_str()) {
        // Return as base64 data URL
        use base64::{Engine as _, engine::general_purpose};
        let b64 = general_purpose::STANDARD.encode(&data);
        let mime = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            _ => "application/octet-stream",
        };
        Ok(format!("data:{};base64,{}", mime, b64))
    } else {
        // Return as UTF-8 text
        String::from_utf8(data).map_err(|e| format!("Not valid UTF-8: {}", e))
    }
}

#[tauri::command]
async fn open_with_native(
    path: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    // Download to temp file first
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(&path);
    
    // Create parent directories
    if let Some(parent) = temp_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Read from VFS
    let mut vfs = app_state.vfs.write().await;
    let data = vfs
        .read_file(&PathBuf::from(&path))
        .await
        .map_err(|e| e.to_string())?;

    // Write to temp file
    std::fs::write(&temp_path, data).map_err(|e| e.to_string())?;

    // Open with default app
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", temp_path.to_str().unwrap()])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&temp_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&temp_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn is_node_running(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<bool, String> {
    let state_guard = state.read().await;
    Ok(state_guard.is_some())
}

#[tauri::command]
async fn list_peers(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<Vec<PeerInfo>, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let mut peer_infos = Vec::new();
    
    // Get legacy TCP peers
    let peers = app_state.node.peers.read().await;
    for peer_addr in peers.iter() {
        peer_infos.push(PeerInfo {
            id: peer_addr.clone(),
            address: peer_addr.clone(),
            status: "online".to_string(),
            blocks: 0,
            latency: 0,
        });
    }
    
    // Get P2P peers if available
    if let Some(ref cmd_tx) = app_state.p2p_command_tx {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        if cmd_tx.send(P2PNodeCommand::GetConnectedPeers(reply_tx)).is_ok() {
            if let Ok(connected_peers) = reply_rx.await {
                for peer_id in connected_peers {
                    peer_infos.push(PeerInfo {
                        id: peer_id.to_string(),
                        address: peer_id.to_string(),
                        status: "online".to_string(),
                        blocks: 0,
                        latency: 0,
                    });
                }
            }
        }
    }
    
    Ok(peer_infos)
}

#[tauri::command]
async fn add_peer(
    address: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let mut peers = app_state.node.peers.write().await;
    if !peers.contains(&address) {
        peers.push(address);
    }
    
    Ok(())
}

// ============ WORKSPACE COMMANDS ============

#[tauri::command]
async fn list_workspaces(
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<Vec<Workspace>, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let workspaces = app_state.workspace_manager.list_user_workspaces(&app_state.current_user_id).await;
    Ok(workspaces)
}

#[tauri::command]
async fn create_workspace(
    name: String,
    description: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<String, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let workspace_id = app_state.workspace_manager
        .create_workspace(
            name,
            description,
            app_state.current_user_id,
            app_state.current_user_email.clone()
        )
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(workspace_id.to_string())
}

#[tauri::command]
async fn invite_workspace_member(
    workspace_id: String,
    email: String,
    permission: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<String, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let ws_id = uuid::Uuid::parse_str(&workspace_id).map_err(|e| e.to_string())?;
    let perm = match permission.as_str() {
        "Viewer" => Permission::Viewer,
        "Editor" => Permission::Editor,
        "Admin" => Permission::Admin,
        _ => return Err("Invalid permission".to_string()),
    };
    
    let invite_id = app_state.workspace_manager
        .invite_member(ws_id, email, perm, app_state.current_user_id)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(invite_id.to_string())
}

#[tauri::command]
async fn accept_workspace_invite(
    workspace_id: String,
    invite_id: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let ws_id = uuid::Uuid::parse_str(&workspace_id).map_err(|e| e.to_string())?;
    let inv_id = uuid::Uuid::parse_str(&invite_id).map_err(|e| e.to_string())?;
    
    app_state.workspace_manager
        .accept_invite(ws_id, inv_id, app_state.current_user_id, app_state.current_user_email.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn create_shared_folder(
    workspace_id: String,
    name: String,
    path: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<String, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let ws_id = uuid::Uuid::parse_str(&workspace_id).map_err(|e| e.to_string())?;
    
    let mut workspace = app_state.workspace_manager.get_workspace(&ws_id).await
        .ok_or("Workspace not found")?;
    
    let folder_id = workspace.create_shared_folder(name, path, app_state.current_user_id)
        .map_err(|e| e.to_string())?;
    
    Ok(folder_id.to_string())
}

#[tauri::command]
async fn share_folder_with_member(
    workspace_id: String,
    folder_id: String,
    member_id: String,
    permission: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let ws_id = uuid::Uuid::parse_str(&workspace_id).map_err(|e| e.to_string())?;
    let fld_id = uuid::Uuid::parse_str(&folder_id).map_err(|e| e.to_string())?;
    let mem_id = uuid::Uuid::parse_str(&member_id).map_err(|e| e.to_string())?;
    
    let perm = match permission.as_str() {
        "Viewer" => Permission::Viewer,
        "Editor" => Permission::Editor,
        "Admin" => Permission::Admin,
        _ => return Err("Invalid permission".to_string()),
    };
    
    let mut workspace = app_state.workspace_manager.get_workspace(&ws_id).await
        .ok_or("Workspace not found")?;
    
    workspace.share_folder_with_member(fld_id, mem_id, perm, app_state.current_user_id)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

// ============ FIREWALL COMMANDS ============

#[tauri::command]
async fn check_firewall_access() -> Result<bool, String> {
    firewall::check_firewall_access("MSSCS")
}

#[tauri::command]
async fn request_firewall_access(_app_handle: tauri::AppHandle) -> Result<(), String> {
    // Get the application executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let exe_path_str = exe_path.to_str()
        .ok_or("Invalid executable path")?;
    
    tracing::info!("🛡️  Requesting firewall access for: {}", exe_path_str);
    
    // Try to add firewall rule with elevation
    firewall::request_firewall_access_with_elevation("MSSCS", exe_path_str)
}

#[tauri::command]
async fn open_firewall_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // Try multiple methods to open firewall settings
        
        // Method 1: Try ms-settings URI
        let result1 = Command::new("cmd")
            .args(&["/C", "start", "ms-settings:network-firewall"])
            .output();
        
        if result1.is_ok() && result1.unwrap().status.success() {
            return Ok(());
        }
        
        // Method 2: Try control panel directly
        let result2 = Command::new("control")
            .args(&["firewall.cpl"])
            .spawn();
        
        if result2.is_ok() {
            return Ok(());
        }
        
        // Method 3: Try PowerShell
        let result3 = Command::new("powershell")
            .args(&["-Command", "Start-Process", "firewall.cpl"])
            .spawn();
        
        if result3.is_ok() {
            return Ok(());
        }
        
        Err("Failed to open firewall settings".to_string())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

fn main() {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();
    
    let app_state: Arc<RwLock<Option<AppStateWrapper>>> = Arc::new(RwLock::new(None));

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_node,
            is_node_running,
            wait_for_node_ready,
            list_files,
            list_peers,
            add_peer,
            upload_file,
            download_file,
            delete_file,
            get_metrics,
            get_storage_stats,
            preview_file,
            open_with_native,
            get_storage_limit,
            set_storage_limit,
            list_workspaces,
            create_workspace,
            invite_workspace_member,
            accept_workspace_invite,
            create_shared_folder,
            share_folder_with_member,
            check_firewall_access,
            request_firewall_access,
            open_firewall_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
