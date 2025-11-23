// P2P API - REST API for decentralized storage
use crate::error::{MSSCSError, Result};
use crate::p2p_vfs::{P2PVirtualFileSystem, StorageStats};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Application state
#[derive(Clone)]
pub struct P2PAppState {
    pub vfs: Arc<P2PVirtualFileSystem>,
}

/// Upload file request
#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub path: String,
    pub content: String, // Base64
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub uuid: String,
    pub status: String,
}

/// Download response
#[derive(Debug, Serialize)]
pub struct DownloadResponse {
    pub content: String, // Base64
}

/// List files response
#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub files: Vec<String>,
}

/// Stats response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_files: usize,
    pub cached_blocks: usize,
    pub connected_peers: usize,
}

/// Create P2P API router
pub fn create_p2p_router(state: P2PAppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        .route("/upload", post(upload_handler))
        .route("/download/:path", get(download_handler))
        .route("/delete/:path", delete(delete_handler))
        .route("/files", get(list_handler))
        .route("/stats", get(stats_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .with_state(state)
}

/// Upload file handler
async fn upload_handler(
    State(state): State<P2PAppState>,
    Json(req): Json<UploadRequest>,
) -> Result<impl IntoResponse> {
    use base64::{Engine as _, engine::general_purpose};
    
    let content = general_purpose::STANDARD.decode(&req.content)
        .map_err(|e| MSSCSError::InvalidData(format!("Invalid base64: {}", e)))?;
    
    let path = PathBuf::from(&req.path);
    let uuid = state.vfs.upload_file(&path, &content).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            uuid: uuid.to_string(),
            status: "uploaded".to_string(),
        }),
    ))
}

/// Download file handler
async fn download_handler(
    State(state): State<P2PAppState>,
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse> {
    use base64::{Engine as _, engine::general_purpose};
    
    let path = PathBuf::from(&file_path);
    let content = state.vfs.download_file(&path).await?;
    let encoded = general_purpose::STANDARD.encode(&content);
    
    Ok(Json(DownloadResponse { content: encoded }))
}

/// Delete file handler
async fn delete_handler(
    State(state): State<P2PAppState>,
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse> {
    let path = PathBuf::from(&file_path);
    state.vfs.delete_file(&path).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// List files handler
async fn list_handler(
    State(state): State<P2PAppState>,
) -> Result<impl IntoResponse> {
    let files = state.vfs.list_files().await;
    Ok(Json(ListResponse { files }))
}

/// Stats handler
async fn stats_handler(
    State(state): State<P2PAppState>,
) -> Result<impl IntoResponse> {
    let stats = state.vfs.get_stats().await;
    Ok(Json(StatsResponse {
        total_files: stats.total_files,
        cached_blocks: stats.cached_blocks,
        connected_peers: stats.connected_peers,
    }))
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "4.0.0"
    }))
}

// IntoResponse implementation is in api.rs to avoid duplication
