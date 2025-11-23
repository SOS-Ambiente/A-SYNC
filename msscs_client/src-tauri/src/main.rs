// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use msscs_v4::{
    config::Config,
    metrics::Metrics,
    network::Node,
    persistence::PersistenceManager,
    vfs::VirtualFileSystem,
    p2p_network::{P2PNode, P2PConfig, P2PNodeCommand},
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
async fn start_node(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<(), String> {
    let mut state_guard = state.write().await;
    
    if state_guard.is_some() {
        return Ok(()); // Already started
    }

    // Get app data directory (platform-specific)
    let app_data_dir = if cfg!(target_os = "windows") {
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
    };

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

    // Initialize advanced P2P node (libp2p) - always initialize for local network discovery
    tracing::info!("Initializing P2P networking with libp2p...");
    
    let p2p_config = P2PConfig {
        listen_port: 0, // Random port
        bootstrap_peers: Vec::new(), // Empty for now - will use mDNS for local discovery
        max_peers: 50,
        replication_factor: config.replication_factor,
        enable_mdns: true,  // Enable local network peer discovery
    };
    
    let p2p_command_tx = match P2PNode::new(p2p_config).await {
        Ok(p2p_node) => {
            // Get command sender before starting
            let cmd_tx = p2p_node.get_command_sender();
            
            let (mut event_rx, _local_blocks) = p2p_node.start().await.map_err(|e| e.to_string())?;

            // Spawn event handler
            tokio::spawn(async move {
                while let Some(event) = event_rx.recv().await {
                    use msscs_v4::p2p_network::P2PEvent;
                    match event {
                        P2PEvent::PeerConnected(peer_id) => {
                            tracing::info!("Connected to peer: {}", peer_id);
                        }
                        P2PEvent::PeerDisconnected(peer_id) => {
                            tracing::info!("Disconnected from peer: {}", peer_id);
                        }
                        P2PEvent::BlockReceived { peer, block } => {
                            tracing::info!("Received block {} from {}", block.uuid, peer);
                        }
                        _ => {}
                    }
                }
            });

            Some(cmd_tx)
        }
        Err(e) => {
            tracing::warn!("Failed to initialize P2P node: {}", e);
            None
        }
    };

    // Associate VFS with node
    vfs.set_node(node.clone());
    let vfs = Arc::new(RwLock::new(vfs));

    // Initialize metrics
    let metrics = Arc::new(Metrics::new());

    *state_guard = Some(AppStateWrapper {
        vfs,
        node,
        p2p_command_tx,
        config,
        metrics,
    });

    Ok(())
}

#[tauri::command]
async fn list_files(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<Vec<FileInfo>, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let vfs = app_state.vfs.read().await;
    let all_metadata = vfs.get_all_metadata();
    
    // Return actual metadata
    let file_infos: Vec<FileInfo> = all_metadata.into_iter().map(|(path, metadata)| {
        FileInfo {
            path,
            size: metadata.size,
            blocks: metadata.blocks,
            uuid: metadata.uuid.to_string(),
            synced: true,
        }
    }).collect();
    
    Ok(file_infos)
}

#[tauri::command]
async fn upload_file(
    file_path: String,
    window: tauri::Window,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<FileUploadResult, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    // Read file
    let data = std::fs::read(&file_path).map_err(|e| e.to_string())?;
    let file_name = PathBuf::from(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file path")?
        .to_string();

    let total_size = data.len();
    let chunk_size = app_state.config.chunk_size;
    let total_blocks = (total_size + chunk_size - 1) / chunk_size;

    // Emit initial progress
    let _ = window.emit("upload-progress", serde_json::json!({
        "file": file_name,
        "progress": 0,
        "current": 0,
        "total": total_blocks
    }));

    let start_time = std::time::Instant::now();

    // Upload to VFS with progress tracking
    let mut vfs = app_state.vfs.write().await;
    let window_clone = window.clone();
    let file_name_clone = file_name.clone();
    
    let uuid = vfs
        .write_file_with_progress(&PathBuf::from(&file_name), &data, |current, total| {
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
                "eta": eta
            }));
        })
        .await
        .map_err(|e| e.to_string())?;

    // Emit completion
    let _ = window.emit("upload-progress", serde_json::json!({
        "file": file_name,
        "progress": 100,
        "current": total_blocks,
        "total": total_blocks,
        "complete": true
    }));

    Ok(FileUploadResult {
        uuid: uuid.to_string(),
        blocks: total_blocks,
    })
}

#[tauri::command]
async fn download_file(
    path: String,
    save_path: String,
    window: tauri::Window,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    // Get file metadata for progress tracking
    let vfs_read = app_state.vfs.read().await;
    let metadata = vfs_read.get_file_metadata(&path);
    let total_blocks = metadata.map(|m| m.blocks).unwrap_or(0);
    drop(vfs_read);

    // Emit initial progress
    let _ = window.emit("download-progress", serde_json::json!({
        "file": path,
        "progress": 0,
        "current": 0,
        "total": total_blocks
    }));

    let start_time = std::time::Instant::now();

    // Read from VFS with progress tracking
    let mut vfs = app_state.vfs.write().await;
    let window_clone = window.clone();
    let path_clone = path.clone();
    
    let data = vfs
        .read_file_with_progress(&PathBuf::from(&path), |current, total| {
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
                "eta": eta
            }));
        })
        .await
        .map_err(|e| e.to_string())?;

    // Write to disk
    std::fs::write(&save_path, data).map_err(|e| e.to_string())?;

    // Emit completion
    let _ = window.emit("download-progress", serde_json::json!({
        "file": path,
        "progress": 100,
        "complete": true
    }));

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
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    let snapshot = app_state.metrics.snapshot();
    
    // Get P2P peer count if available
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
    
    // Combine legacy and P2P peer counts
    let total_peer_count = snapshot.peer_count + p2p_peer_count;
    
    Ok(NodeMetrics {
        block_count: snapshot.block_count,
        storage_bytes: snapshot.storage_bytes,
        peer_count: total_peer_count,
        uptime_seconds: snapshot.uptime_seconds,
        requests_total: snapshot.requests_total,
        requests_failed: snapshot.requests_failed,
        success_rate: snapshot.success_rate,
    })
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

fn main() {
    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();
    
    let app_state: Arc<RwLock<Option<AppStateWrapper>>> = Arc::new(RwLock::new(None));

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_node,
            is_node_running,
            list_files,
            list_peers,
            add_peer,
            upload_file,
            download_file,
            delete_file,
            get_metrics,
            preview_file,
            open_with_native,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
