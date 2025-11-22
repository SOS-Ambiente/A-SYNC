// MSSCS Mobile - Library entry point for Android/iOS
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use msscs_v4::{
    config::Config,
    metrics::Metrics,
    network::Node,
    persistence::PersistenceManager,
    vfs::VirtualFileSystem,
    block::DataBlock,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use tauri::State;
use tokio::sync::RwLock;

mod network_discovery;
mod file_viewer;

use network_discovery::{NetworkDiscovery, DiscoveredNode as NetworkDiscoveredNode};

// Application state
struct AppStateWrapper {
    vfs: Arc<RwLock<VirtualFileSystem>>,
    node: Arc<Node>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    discovery: Arc<RwLock<NetworkDiscovery>>,
    cancel_tokens: Arc<RwLock<HashMap<String, Arc<AtomicBool>>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct DiscoveredNode {
    name: String,
    address: String,
    port: u16,
    node_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileUploadResult {
    uuid: String,
    blocks: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileInfo {
    path: String,
    size: u64,
    blocks: usize,
    uuid: String,
    synced: bool,
    mime_type: String,
    extension: String,
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
struct SecurityInfo {
    encryption: String,
    hashing: String,
    compression: String,
    decentralized: bool,
    local_storage: bool,
    p2p_network: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileDetails {
    path: String,
    uuid: String,
    size: u64,
    blocks: usize,
    encrypted: bool,
    replicated: bool,
    peers_count: usize,
    mime_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FilePreview {
    mime_type: String,
    data: String, // Base64 encoded
    size: u64,
}

// Tauri commands
#[tauri::command]
async fn start_node(state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>) -> Result<(), String> {
    let mut state_guard = state.write().await;
    
    if state_guard.is_some() {
        return Ok(()); // Already started
    }

    // Get app data directory
    let data_dir = get_app_data_dir();
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    // Load or create configuration
    let config_path = data_dir.join("config.toml");
    let mut config = if config_path.exists() {
        Config::load(&config_path).map_err(|e| e.to_string())?
    } else {
        Config::default()
    };
    
    // Set mobile-specific config
    config.data_dir = data_dir.join("msscs_data");
    config.save(&config_path).map_err(|e| e.to_string())?;
    let config = Arc::new(config);

    // Initialize persistence
    std::fs::create_dir_all(&config.data_dir).map_err(|e| e.to_string())?;
    let persistence = Arc::new(
        PersistenceManager::new(config.data_dir.clone()).map_err(|e| e.to_string())?
    );

    // Initialize VFS
    let mut vfs = VirtualFileSystem::new(config.clone(), persistence.clone())
        .map_err(|e| e.to_string())?;

    // Initialize network node
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

    // Associate VFS with node
    vfs.set_node(node.clone());
    let vfs = Arc::new(RwLock::new(vfs));

    // Initialize metrics
    let metrics = Arc::new(Metrics::new());

    // Initialize network discovery
    let discovery = Arc::new(RwLock::new(NetworkDiscovery::new()));
    let discovery_clone = discovery.clone();
    let node_clone = node.clone();
    
    tokio::spawn(async move {
        discovery_clone.write().await.start_discovery(node_clone).await;
    });

    *state_guard = Some(AppStateWrapper {
        vfs,
        node,
        config,
        metrics,
        discovery,
        cancel_tokens: Arc::new(RwLock::new(HashMap::new())),
    });

    Ok(())
}

#[tauri::command]
async fn discover_nodes(
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<Vec<DiscoveredNode>, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let discovery = app_state.discovery.read().await;
    let network_nodes = discovery.get_discovered_nodes();
    
    // Convert NetworkDiscoveredNode to DiscoveredNode
    let nodes = network_nodes.into_iter().map(|n| DiscoveredNode {
        name: n.name,
        address: n.address,
        port: n.port,
        node_id: n.node_id,
    }).collect();
    
    Ok(nodes)
}

#[tauri::command]
async fn connect_to_node(
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

#[tauri::command]
async fn list_files(
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<Vec<FileInfo>, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let vfs = app_state.vfs.read().await;
    let file_paths = vfs.list_files();
    
    let file_infos: Vec<FileInfo> = file_paths.into_iter().map(|path| {
        let extension = PathBuf::from(&path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        
        let mime_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();
        
        FileInfo {
            path,
            size: 0,
            blocks: 0,
            uuid: String::new(),
            synced: true,
            mime_type,
            extension,
        }
    }).collect();
    
    Ok(file_infos)
}

#[tauri::command]
async fn upload_file(
    file_path: String,
    operation_id: String,
    window: tauri::Window,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<FileUploadResult, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    // Create cancellation token
    let cancel_token = Arc::new(AtomicBool::new(false));
    app_state.cancel_tokens.write().await.insert(operation_id.clone(), cancel_token.clone());

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
        "operationId": operation_id,
        "file": file_name,
        "progress": 0,
        "current": 0,
        "total": total_blocks,
        "speed": 0,
        "eta": 0
    }));

    let start_time = std::time::Instant::now();
    let mut vfs = app_state.vfs.write().await;
    
    // Upload with progress tracking and cancellation
    let window_clone = window.clone();
    let file_name_clone = file_name.clone();
    let operation_id_clone = operation_id.clone();
    
    let result = vfs
        .write_file_with_progress(&PathBuf::from(&file_name), &data, |current, total| {
            // Check for cancellation
            if cancel_token.load(Ordering::Relaxed) {
                return;
            }

            let progress = (current as f64 / total as f64 * 100.0) as u32;
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { (current as f64 / elapsed) as u64 } else { 0 };
            let eta = if speed > 0 { ((total - current) as f64 / speed as f64) as u64 } else { 0 };
            
            let _ = window_clone.emit("upload-progress", serde_json::json!({
                "operationId": operation_id_clone,
                "file": file_name_clone,
                "progress": progress,
                "current": current,
                "total": total,
                "speed": speed,
                "eta": eta
            }));
        })
        .await;

    // Clean up cancel token
    app_state.cancel_tokens.write().await.remove(&operation_id);

    // Check if cancelled
    if cancel_token.load(Ordering::Relaxed) {
        let _ = window.emit("upload-progress", serde_json::json!({
            "operationId": operation_id,
            "file": file_name,
            "cancelled": true
        }));
        return Err("Upload cancelled".to_string());
    }

    let uuid = result.map_err(|e| e.to_string())?;

    // Emit completion
    let _ = window.emit("upload-progress", serde_json::json!({
        "operationId": operation_id,
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
    operation_id: String,
    window: tauri::Window,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    // Create cancellation token
    let cancel_token = Arc::new(AtomicBool::new(false));
    app_state.cancel_tokens.write().await.insert(operation_id.clone(), cancel_token.clone());

    // Emit initial progress
    let _ = window.emit("download-progress", serde_json::json!({
        "operationId": operation_id,
        "file": path,
        "progress": 0,
        "current": 0,
        "total": 100
    }));

    let start_time = std::time::Instant::now();
    let mut vfs = app_state.vfs.write().await;
    
    // Download with progress tracking
    let window_clone = window.clone();
    let path_clone = path.clone();
    let operation_id_clone = operation_id.clone();
    
    let result = vfs
        .read_file_with_progress(&PathBuf::from(&path), |current, total| {
            // Check for cancellation
            if cancel_token.load(Ordering::Relaxed) {
                return;
            }

            let progress = (current as f64 / total as f64 * 100.0) as u32;
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { (current as f64 / elapsed) as u64 } else { 0 };
            let eta = if speed > 0 { ((total - current) as f64 / speed as f64) as u64 } else { 0 };
            
            let _ = window_clone.emit("download-progress", serde_json::json!({
                "operationId": operation_id_clone,
                "file": path_clone,
                "progress": progress,
                "current": current,
                "total": total,
                "speed": speed,
                "eta": eta
            }));
        })
        .await;

    // Clean up cancel token
    app_state.cancel_tokens.write().await.remove(&operation_id);

    // Check if cancelled
    if cancel_token.load(Ordering::Relaxed) {
        let _ = window.emit("download-progress", serde_json::json!({
            "operationId": operation_id,
            "file": path,
            "cancelled": true
        }));
        return Err("Download cancelled".to_string());
    }

    let data = result.map_err(|e| e.to_string())?;

    std::fs::write(&save_path, data).map_err(|e| e.to_string())?;

    // Emit completion
    let _ = window.emit("download-progress", serde_json::json!({
        "operationId": operation_id,
        "file": path,
        "progress": 100,
        "complete": true
    }));

    Ok(())
}

#[tauri::command]
async fn cancel_operation(
    operation_id: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    if let Some(token) = app_state.cancel_tokens.read().await.get(&operation_id) {
        token.store(true, Ordering::Relaxed);
        Ok(())
    } else {
        Err("Operation not found".to_string())
    }
}

#[tauri::command]
async fn preview_file(
    path: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<FilePreview, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    let mut vfs = app_state.vfs.write().await;
    let data = vfs
        .read_file(&PathBuf::from(&path))
        .await
        .map_err(|e| e.to_string())?;

    let mime_type = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();

    use base64::{Engine as _, engine::general_purpose};
    let encoded = general_purpose::STANDARD.encode(&data);

    Ok(FilePreview {
        mime_type,
        data: encoded,
        size: data.len() as u64,
    })
}

#[tauri::command]
async fn open_with_system(
    path: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    // Download file to temp location
    let temp_dir = std::env::temp_dir();
    let path_buf = PathBuf::from(&path);
    let file_name = path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file path")?;
    let temp_path = temp_dir.join(file_name);

    let mut vfs = app_state.vfs.write().await;
    let data = vfs
        .read_file(&PathBuf::from(&path))
        .await
        .map_err(|e| e.to_string())?;

    std::fs::write(&temp_path, data).map_err(|e| e.to_string())?;

    // Open with system default app
    file_viewer::open_with_system(&temp_path)?;

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
async fn get_metrics(
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<NodeMetrics, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    let snapshot = app_state.metrics.snapshot();
    
    Ok(NodeMetrics {
        block_count: snapshot.block_count,
        storage_bytes: snapshot.storage_bytes,
        peer_count: snapshot.peer_count,
        uptime_seconds: snapshot.uptime_seconds,
        requests_total: snapshot.requests_total,
        requests_failed: snapshot.requests_failed,
        success_rate: snapshot.success_rate,
    })
}

#[tauri::command]
async fn get_security_info(
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<SecurityInfo, String> {
    let _state_guard = state.read().await;
    let _app_state = _state_guard.as_ref().ok_or("Node not started")?;
    
    Ok(SecurityInfo {
        encryption: "AES-256-GCM (Quantum-Resistant)".to_string(),
        hashing: "SHA-256 + UUID Position-Based".to_string(),
        compression: "Huffman Coding".to_string(),
        decentralized: true,
        local_storage: true,
        p2p_network: true,
    })
}

#[tauri::command]
async fn get_file_details(
    path: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<FileDetails, String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;
    
    let vfs = app_state.vfs.read().await;
    let file_manifest = vfs.list_files();
    
    if !file_manifest.contains(&path) {
        return Err("File not found".to_string());
    }
    
    let peers = app_state.node.peers.read().await;
    let peers_count = peers.len();
    
    let mime_type = mime_guess::from_path(&path)
        .first_or_octet_stream()
        .to_string();
    
    // Get file size from blocks
    let block_count = vfs.block_count();
    let storage_bytes = vfs.storage_bytes();
    
    Ok(FileDetails {
        path: path.clone(),
        uuid: "encrypted".to_string(),
        size: storage_bytes,
        blocks: block_count,
        encrypted: true,
        replicated: peers_count > 0,
        peers_count,
        mime_type,
    })
}

fn get_app_data_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/data/data/com.msscs.mobile/files")
    }
    
    #[cfg(not(target_os = "android"))]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("msscs-mobile")
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state: Arc<RwLock<Option<AppStateWrapper>>> = Arc::new(RwLock::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_node,
            discover_nodes,
            connect_to_node,
            list_files,
            upload_file,
            download_file,
            cancel_operation,
            preview_file,
            open_with_system,
            delete_file,
            get_metrics,
            get_security_info,
            get_file_details,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
