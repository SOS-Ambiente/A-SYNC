// GATEWAY NODES - HTTP Gateway for non-P2P clients
// Provides IPFS-compatible HTTP API for browser and mobile access

use crate::error::{MSSCSError, Result};
use crate::p2p_vfs::P2PVirtualFileSystem;
use crate::content_addressing::ContentId;
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use std::collections::HashMap;

/// Gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Requests per minute per IP
    pub rate_limit: usize,
    /// Enable caching
    pub enable_caching: bool,
    /// Cache TTL (seconds)
    pub cache_ttl: u64,
    /// Enable authentication
    pub require_auth: bool,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        GatewayConfig {
            enable_rate_limiting: true,
            rate_limit: 100,
            enable_caching: true,
            cache_ttl: 3600,
            require_auth: false,
        }
    }
}

/// Gateway state
#[derive(Clone)]
pub struct GatewayState {
    pub vfs: Arc<P2PVirtualFileSystem>,
    pub config: GatewayConfig,
    pub cache: Arc<tokio::sync::RwLock<HashMap<String, (Vec<u8>, std::time::Instant)>>>,
}

/// IPFS-compatible gateway response
#[derive(Debug, Serialize)]
pub struct IpfsResponse {
    pub hash: String,
    pub size: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<IpfsLink>>,
}

#[derive(Debug, Serialize)]
pub struct IpfsLink {
    pub name: String,
    pub hash: String,
    pub size: usize,
}

/// Upload response
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub cid: String,
    pub size: usize,
    pub status: String,
}

/// Gateway statistics
#[derive(Debug, Serialize)]
pub struct GatewayStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub bytes_served: u64,
    pub active_connections: usize,
}

/// Create gateway router
pub fn create_gateway_router(state: GatewayState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        // IPFS-compatible endpoints
        .route("/ipfs/:cid", get(ipfs_get_handler))
        .route("/ipfs/:cid/*path", get(ipfs_get_path_handler))
        
        // Upload endpoints
        .route("/api/v0/add", post(ipfs_add_handler))
        .route("/api/v0/cat", get(ipfs_cat_handler))
        .route("/api/v0/ls", get(ipfs_ls_handler))
        
        // Gateway-specific endpoints
        .route("/gateway/upload", post(gateway_upload_handler))
        .route("/gateway/download/:cid", get(gateway_download_handler))
        .route("/gateway/stats", get(gateway_stats_handler))
        .route("/gateway/health", get(gateway_health_handler))
        
        .layer(cors)
        .with_state(state)
}

/// IPFS GET handler - retrieve content by CID
async fn ipfs_get_handler(
    State(state): State<GatewayState>,
    Path(cid): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("📥 IPFS GET: /ipfs/{}", cid);
    
    // Check cache first
    if state.config.enable_caching {
        let cache = state.cache.read().await;
        if let Some((data, cached_at)) = cache.get(&cid) {
            let age = cached_at.elapsed().as_secs();
            if age < state.config.cache_ttl {
                tracing::debug!("   ✅ Cache hit (age: {}s)", age);
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/octet-stream")
                    .header("X-Cache", "HIT")
                    .body(axum::body::Body::from(data.clone()))
                    .unwrap());
            }
        }
    }
    
    // Fetch from P2P network
    // In production, parse CID and retrieve from VFS
    tracing::debug!("   Cache miss, fetching from P2P network");
    
    // For now, return not found
    Err(MSSCSError::NotFound(format!("Content {} not found", cid)))
}

/// IPFS GET with path handler
async fn ipfs_get_path_handler(
    State(_state): State<GatewayState>,
    Path((cid, path)): Path<(String, String)>,
) -> Result<Response> {
    tracing::info!("📥 IPFS GET: /ipfs/{}/{}", cid, path);
    
    // In production, navigate directory structure
    Err(MSSCSError::NotFound(format!("Path {}/{} not found", cid, path)))
}

/// IPFS add handler - upload content
async fn ipfs_add_handler(
    State(_state): State<GatewayState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse> {
    tracing::info!("📤 IPFS ADD: {} bytes", body.len());
    
    // Calculate CID
    let cid = ContentId::from_data(&body);
    
    // Store in P2P network
    // In production, upload to VFS
    
    let response = IpfsResponse {
        hash: cid.to_hex(),
        size: body.len(),
        links: None,
    };
    
    Ok(Json(response))
}

/// IPFS cat handler - retrieve content
async fn ipfs_cat_handler(
    State(_state): State<GatewayState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response> {
    let cid = params.get("arg")
        .ok_or_else(|| MSSCSError::Validation("Missing 'arg' parameter".to_string()))?;
    
    tracing::info!("📥 IPFS CAT: {}", cid);
    
    // Fetch content
    Err(MSSCSError::NotFound(format!("Content {} not found", cid)))
}

/// IPFS ls handler - list directory
async fn ipfs_ls_handler(
    State(_state): State<GatewayState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse> {
    let cid = params.get("arg")
        .ok_or_else(|| MSSCSError::Validation("Missing 'arg' parameter".to_string()))?;
    
    tracing::info!("📋 IPFS LS: {}", cid);
    
    let response = IpfsResponse {
        hash: cid.clone(),
        size: 0,
        links: Some(vec![]),
    };
    
    Ok(Json(response))
}

/// Gateway upload handler
async fn gateway_upload_handler(
    State(_state): State<GatewayState>,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse> {
    tracing::info!("📤 Gateway upload: {} bytes", body.len());
    
    let cid = ContentId::from_data(&body);
    
    let response = UploadResponse {
        cid: cid.to_hex(),
        size: body.len(),
        status: "uploaded".to_string(),
    };
    
    Ok((StatusCode::CREATED, Json(response)))
}

/// Gateway download handler
async fn gateway_download_handler(
    State(state): State<GatewayState>,
    Path(cid): Path<String>,
) -> Result<impl IntoResponse> {
    tracing::info!("📥 Gateway download: {}", cid);
    
    // Check cache
    if state.config.enable_caching {
        let cache = state.cache.read().await;
        if let Some((data, _)) = cache.get(&cid) {
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/octet-stream")
                .header("X-Cache", "HIT")
                .body(axum::body::Body::from(data.clone()))
                .unwrap());
        }
    }
    
    Err(MSSCSError::NotFound(format!("Content {} not found", cid)))
}

/// Gateway statistics handler
async fn gateway_stats_handler(
    State(_state): State<GatewayState>,
) -> Result<impl IntoResponse> {
    let stats = GatewayStats {
        total_requests: 0,
        cache_hits: 0,
        cache_misses: 0,
        bytes_served: 0,
        active_connections: 0,
    };
    
    Ok(Json(stats))
}

/// Gateway health check
async fn gateway_health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "4.0.0",
        "gateway": "msscs-gateway"
    }))
}

/// Rate limiter
pub struct RateLimiter {
    /// IP -> (request_count, window_start)
    limits: tokio::sync::RwLock<HashMap<String, (usize, std::time::Instant)>>,
    /// Requests per window
    max_requests: usize,
    /// Window duration
    window: std::time::Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        RateLimiter {
            limits: tokio::sync::RwLock::new(HashMap::new()),
            max_requests,
            window: std::time::Duration::from_secs(window_secs),
        }
    }
    
    pub async fn check_rate_limit(&self, ip: &str) -> bool {
        let mut limits = self.limits.write().await;
        let now = std::time::Instant::now();
        
        if let Some((count, window_start)) = limits.get_mut(ip) {
            if now.duration_since(*window_start) > self.window {
                // Reset window
                *count = 1;
                *window_start = now;
                true
            } else if *count < self.max_requests {
                *count += 1;
                true
            } else {
                false // Rate limit exceeded
            }
        } else {
            limits.insert(ip.to_string(), (1, now));
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gateway_config() {
        let config = GatewayConfig::default();
        assert!(config.enable_rate_limiting);
        assert_eq!(config.rate_limit, 100);
    }
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, 60);
        
        // First 3 requests should succeed
        assert!(limiter.check_rate_limit("127.0.0.1").await);
        assert!(limiter.check_rate_limit("127.0.0.1").await);
        assert!(limiter.check_rate_limit("127.0.0.1").await);
        
        // 4th request should fail
        assert!(!limiter.check_rate_limit("127.0.0.1").await);
        
        // Different IP should succeed
        assert!(limiter.check_rate_limit("192.168.1.1").await);
    }
}
