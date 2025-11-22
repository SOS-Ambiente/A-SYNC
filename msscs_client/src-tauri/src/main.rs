// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use msscs_v4::{
    config::Config,
    metrics::Metrics,
    network::Node,
    persistence::PersistenceManager,
    vfs::VirtualFileSystem,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

// Application state
struct AppStateWrapper {
    vfs: Arc<RwLock<VirtualFileSystem>>,
    node: Arc<Node>,
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

    *state_guard = Some(AppStateWrapper {
        vfs,
        node,
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
    let file_paths = vfs.list_files();
    
    // Return basic info without reading files (performance)
    let file_infos: Vec<FileInfo> = file_paths.into_iter().map(|path| {
        FileInfo {
            path,
            size: 0, // Will be updated on demand
            blocks: 0,
            uuid: String::new(),
            synced: true,
        }
    }).collect();
    
    Ok(file_infos)
}

#[tauri::command]
async fn upload_file(
    file_path: String,
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

    // Upload to VFS
    let mut vfs = app_state.vfs.write().await;
    let uuid = vfs
        .write_file(&PathBuf::from(&file_name), &data)
        .await
        .map_err(|e| e.to_string())?;

    let blocks = (data.len() + app_state.config.chunk_size - 1) / app_state.config.chunk_size;

    Ok(FileUploadResult {
        uuid: uuid.to_string(),
        blocks,
    })
}

#[tauri::command]
async fn download_file(
    path: String,
    save_path: String,
    state: State<'_, Arc<RwLock<Option<AppStateWrapper>>>>,
) -> Result<(), String> {
    let state_guard = state.read().await;
    let app_state = state_guard.as_ref().ok_or("Node not started")?;

    // Read from VFS
    let mut vfs = app_state.vfs.write().await;
    let data = vfs
        .read_file(&PathBuf::from(&path))
        .await
        .map_err(|e| e.to_string())?;

    // Write to disk
    std::fs::write(&save_path, data).map_err(|e| e.to_string())?;

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

fn main() {
    let app_state: Arc<RwLock<Option<AppStateWrapper>>> = Arc::new(RwLock::new(None));

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_node,
            list_files,
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
