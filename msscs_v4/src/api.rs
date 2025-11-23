// API module - REST API endpoints
use crate::config::Config;
use crate::error::{MSSCSError, Result};
use crate::metrics::Metrics;
use crate::network::Node;
use crate::vfs::VirtualFileSystem;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub vfs: Arc<RwLock<VirtualFileSystem>>,
    pub node: Arc<Node>,
    pub config: Arc<Config>,
    pub metrics: Arc<Metrics>,
}

/// Request to write a file
#[derive(Debug, Deserialize)]
pub struct WriteFileRequest {
    pub path: String,
    pub content: String, // Base64 encoded
}

/// Response from write file
#[derive(Debug, Serialize)]
pub struct WriteFileResponse {
    pub uuid: String,
    pub blocks: usize,
}

/// Response from read file
#[derive(Debug, Serialize)]
pub struct ReadFileResponse {
    pub content: String, // Base64 encoded
}

/// Response from delete file
#[derive(Debug, Serialize)]
pub struct DeleteFileResponse {
    pub status: String,
}

/// Response from list files
#[derive(Debug, Serialize)]
pub struct ListFilesResponse {
    pub files: Vec<String>,
}

/// Response from health check
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub peers: usize,
}

/// Response from block info
#[derive(Debug, Serialize)]
pub struct BlockInfoResponse {
    pub uuid: String,
    pub node_index: u64,
    pub size: usize,
    pub compressed_size: usize,
    pub is_encrypted: bool,
    pub previous_uuid: Option<String>,
}

/// Request to get file chunks
#[derive(Debug, Deserialize)]
pub struct GetFileChunksRequest {
    pub file_id: String,
}

/// Response with file chunks
#[derive(Debug, Serialize)]
pub struct FileChunksResponse {
    pub file_id: String,
    pub chunks: Vec<FileChunkInfo>,
    pub total_chunks: usize,
}

/// File chunk information
#[derive(Debug, Serialize)]
pub struct FileChunkInfo {
    pub chunk_id: String,
    pub chunk_index: u64,
    pub size: usize,
    pub compressed_size: usize,
    pub checksum: String,
}

/// Request to download a chunk
#[derive(Debug, Deserialize)]
pub struct DownloadChunkRequest {
    pub chunk_id: String,
}

/// Response with chunk data
#[derive(Debug, Serialize)]
pub struct DownloadChunkResponse {
    pub chunk_id: String,
    pub data: String, // Base64 encoded
    pub checksum: String,
}

/// Create API router
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        .route("/files", post(write_file_handler))
        .route("/files", get(list_files_handler))
        .route("/files/:path", get(read_file_handler))
        .route("/files/:path", delete(delete_file_handler))
        .route("/blocks/:uuid", get(get_block_info_handler))
        .route("/health", get(health_check_handler))
        .route("/metrics", get(metrics_handler))
        .layer(cors)
        .with_state(state)
}

/// Write file handler
async fn write_file_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<WriteFileRequest>,
) -> Result<impl IntoResponse> {
    // Check authentication
    check_auth(&state.config, &headers)?;
    
    // Decode base64 content
    use base64::{Engine as _, engine::general_purpose};
    let content = general_purpose::STANDARD.decode(&req.content)
        .map_err(|e| MSSCSError::InvalidData(format!("Invalid base64: {}", e)))?;
    
    // Write file
    let path = PathBuf::from(&req.path);
    let mut vfs = state.vfs.write().await;
    let uuid = vfs.write_file(&path, &content).await?;
    
    let blocks = (content.len() + state.config.chunk_size - 1) / state.config.chunk_size;
    
    // Update metrics
    state.metrics.record_request(true);
    
    Ok((
        StatusCode::CREATED,
        Json(WriteFileResponse {
            uuid: uuid.to_string(),
            blocks,
        }),
    ))
}

/// Read file handler
async fn read_file_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse> {
    // Check authentication
    check_auth(&state.config, &headers)?;
    
    // Read file
    let path = PathBuf::from(&file_path);
    let mut vfs = state.vfs.write().await;
    let content = vfs.read_file(&path).await?;
    
    // Encode to base64
    use base64::{Engine as _, engine::general_purpose};
    let encoded = general_purpose::STANDARD.encode(&content);
    
    // Update metrics
    state.metrics.record_request(true);
    
    Ok(Json(ReadFileResponse { content: encoded }))
}

/// Delete file handler
async fn delete_file_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse> {
    // Check authentication
    check_auth(&state.config, &headers)?;
    
    // Delete file
    let path = PathBuf::from(&file_path);
    let mut vfs = state.vfs.write().await;
    vfs.delete_file(&path).await?;
    
    // Update metrics
    state.metrics.record_request(true);
    
    Ok(Json(DeleteFileResponse {
        status: "deleted".to_string(),
    }))
}

/// List files handler
async fn list_files_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    // Check authentication
    check_auth(&state.config, &headers)?;
    
    // List files
    let vfs = state.vfs.read().await;
    let files = vfs.list_files();
    
    // Update metrics
    state.metrics.record_request(true);
    
    Ok(Json(ListFilesResponse { files }))
}

/// Get block info handler
async fn get_block_info_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(uuid_str): Path<String>,
) -> Result<impl IntoResponse> {
    // Check authentication
    check_auth(&state.config, &headers)?;
    
    // Parse UUID
    let uuid = Uuid::parse_str(&uuid_str)
        .map_err(|e| MSSCSError::InvalidData(format!("Invalid UUID: {}", e)))?;
    
    // Get block from node
    let blocks = state.node.local_blocks.read().await;
    let block = blocks.get(&uuid.to_string())
        .ok_or_else(|| MSSCSError::NotFound(format!("Block {} not found", uuid)))?;
    
    let size = bincode::serialize(block)
        .map(|v| v.len())
        .unwrap_or(0);
    
    // Update metrics
    state.metrics.record_request(true);
    
    Ok(Json(BlockInfoResponse {
        uuid: uuid.to_string(),
        node_index: block.node_index,
        size,
    }))
}

/// Health check handler
async fn health_check_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let peers = state.node.peers.read().await;
    let peer_count = peers.len();
    
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        peers: peer_count,
    }))
}

/// Metrics handler
async fn metrics_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let snapshot = state.metrics.snapshot();
    Ok(Json(snapshot))
}

/// Check API key authentication
fn check_auth(config: &Config, headers: &HeaderMap) -> Result<()> {
    // If no API keys configured, skip authentication
    let Some(api_keys) = &config.api_keys else {
        return Ok(());
    };
    
    // Get API key from header
    let key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| MSSCSError::Config("Missing X-API-Key header".to_string()))?;
    
    // Validate key
    if !api_keys.contains(&key.to_string()) {
        return Err(MSSCSError::Config("Invalid API key".to_string()));
    }
    
    Ok(())
}

/// Convert MSSCSError to HTTP response
impl IntoResponse for MSSCSError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            MSSCSError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            MSSCSError::InvalidData(msg) => (StatusCode::BAD_REQUEST, msg),
            MSSCSError::Config(msg) => (StatusCode::UNAUTHORIZED, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        
        (status, message).into_response()
    }
}
