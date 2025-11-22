// Metrics module
use serde::Serialize;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

/// System metrics
pub struct Metrics {
    pub block_count: AtomicUsize,
    pub storage_bytes: AtomicU64,
    pub peer_count: AtomicUsize,
    pub uptime_start: Instant,
    pub requests_total: AtomicU64,
    pub requests_failed: AtomicU64,
}

impl Metrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Metrics {
            block_count: AtomicUsize::new(0),
            storage_bytes: AtomicU64::new(0),
            peer_count: AtomicUsize::new(0),
            uptime_start: Instant::now(),
            requests_total: AtomicU64::new(0),
            requests_failed: AtomicU64::new(0),
        }
    }
    
    /// Increment block count
    pub fn increment_blocks(&self) {
        self.block_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Add storage bytes
    pub fn add_storage_bytes(&self, bytes: u64) {
        self.storage_bytes.fetch_add(bytes, Ordering::Relaxed);
    }
    
    /// Set peer count
    pub fn set_peer_count(&self, count: usize) {
        self.peer_count.store(count, Ordering::Relaxed);
    }
    
    /// Record request
    pub fn record_request(&self, success: bool) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.requests_failed.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Get metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let block_count = self.block_count.load(Ordering::Relaxed);
        let storage_bytes = self.storage_bytes.load(Ordering::Relaxed);
        let peer_count = self.peer_count.load(Ordering::Relaxed);
        let uptime_seconds = self.uptime_start.elapsed().as_secs();
        let requests_total = self.requests_total.load(Ordering::Relaxed);
        let requests_failed = self.requests_failed.load(Ordering::Relaxed);
        
        let success_rate = if requests_total > 0 {
            ((requests_total - requests_failed) as f64 / requests_total as f64) * 100.0
        } else {
            100.0
        };
        
        MetricsSnapshot {
            block_count,
            storage_bytes,
            peer_count,
            uptime_seconds,
            requests_total,
            requests_failed,
            success_rate,
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics snapshot for serialization
#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub block_count: usize,
    pub storage_bytes: u64,
    pub peer_count: usize,
    pub uptime_seconds: u64,
    pub requests_total: u64,
    pub requests_failed: u64,
    pub success_rate: f64,
}
