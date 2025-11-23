// BLOCK PINNING & GARBAGE COLLECTION
// Manages block lifecycle and prevents deletion of important data

use crate::error::{MSSCSError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Pin type determines how block is retained
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinType {
    /// User's own data - never garbage collected
    User,
    /// Cached data - subject to LRU eviction
    Cache,
    /// Paid pinning - retained until payment expires
    Paid,
    /// Temporary pin - expires after duration
    Temporary,
}

/// Pin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    /// Block identifier
    pub block_id: String,
    /// Pin type
    pub pin_type: PinType,
    /// Owner user ID
    pub owner: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp (None = never expires)
    pub expires_at: Option<u64>,
    /// Reference count
    pub ref_count: usize,
    /// Last access timestamp (for LRU)
    pub last_accessed: u64,
    /// Size in bytes
    pub size: usize,
}

impl Pin {
    /// Create a new pin
    pub fn new(block_id: String, pin_type: PinType, owner: String, size: usize) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Pin {
            block_id,
            pin_type,
            owner,
            created_at: now,
            expires_at: None,
            ref_count: 1,
            last_accessed: now,
            size,
        }
    }
    
    /// Check if pin has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > expires_at
        } else {
            false
        }
    }
    
    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Add reference
    pub fn add_ref(&mut self) {
        self.ref_count += 1;
    }
    
    /// Remove reference
    pub fn remove_ref(&mut self) -> usize {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        self.ref_count
    }
}

/// Pinning manager
pub struct PinningManager {
    /// All pins (block_id -> pin)
    pins: HashMap<String, Pin>,
    /// User pins (user_id -> block_ids)
    user_pins: HashMap<String, HashSet<String>>,
    /// Cache pins (ordered by LRU)
    cache_pins: Vec<String>,
    /// Maximum cache size in bytes
    max_cache_size: usize,
    /// Current cache size in bytes
    current_cache_size: usize,
}

impl PinningManager {
    /// Create a new pinning manager
    pub fn new(max_cache_size: usize) -> Self {
        PinningManager {
            pins: HashMap::new(),
            user_pins: HashMap::new(),
            cache_pins: Vec::new(),
            max_cache_size,
            current_cache_size: 0,
        }
    }
    
    /// Pin a block
    pub fn pin(&mut self, block_id: String, pin_type: PinType, owner: String, size: usize) -> Result<()> {
        if let Some(pin) = self.pins.get_mut(&block_id) {
            // Block already pinned, increment ref count
            pin.add_ref();
            pin.touch();
            return Ok(());
        }
        
        let pin = Pin::new(block_id.clone(), pin_type, owner.clone(), size);
        
        // Add to appropriate tracking structures
        match pin_type {
            PinType::User => {
                self.user_pins
                    .entry(owner.clone())
                    .or_insert_with(HashSet::new)
                    .insert(block_id.clone());
            }
            PinType::Cache => {
                self.cache_pins.push(block_id.clone());
                self.current_cache_size += size;
                
                // Evict if cache is full
                self.evict_cache_if_needed()?;
            }
            _ => {}
        }
        
        self.pins.insert(block_id, pin);
        Ok(())
    }
    
    /// Unpin a block
    pub fn unpin(&mut self, block_id: &str) -> Result<bool> {
        if let Some(pin) = self.pins.get_mut(block_id) {
            let remaining_refs = pin.remove_ref();
            
            if remaining_refs == 0 {
                // Remove pin
                let pin = self.pins.remove(block_id).unwrap();
                
                // Remove from tracking structures
                match pin.pin_type {
                    PinType::User => {
                        if let Some(user_blocks) = self.user_pins.get_mut(&pin.owner) {
                            user_blocks.remove(block_id);
                        }
                    }
                    PinType::Cache => {
                        self.cache_pins.retain(|id| id != block_id);
                        self.current_cache_size = self.current_cache_size.saturating_sub(pin.size);
                    }
                    _ => {}
                }
                
                return Ok(true); // Block can be deleted
            }
        }
        
        Ok(false) // Block still has references
    }
    
    /// Check if block is pinned
    pub fn is_pinned(&self, block_id: &str) -> bool {
        self.pins.contains_key(block_id)
    }
    
    /// Get pin info
    pub fn get_pin(&self, block_id: &str) -> Option<&Pin> {
        self.pins.get(block_id)
    }
    
    /// Touch a block (update LRU)
    pub fn touch(&mut self, block_id: &str) {
        if let Some(pin) = self.pins.get_mut(block_id) {
            pin.touch();
            
            // Move to end of cache list (most recently used)
            if pin.pin_type == PinType::Cache {
                self.cache_pins.retain(|id| id != block_id);
                self.cache_pins.push(block_id.to_string());
            }
        }
    }
    
    /// Evict cache blocks if needed (LRU)
    fn evict_cache_if_needed(&mut self) -> Result<()> {
        while self.current_cache_size > self.max_cache_size && !self.cache_pins.is_empty() {
            // Remove least recently used
            let block_id = self.cache_pins.remove(0);
            
            if let Some(pin) = self.pins.remove(&block_id) {
                self.current_cache_size = self.current_cache_size.saturating_sub(pin.size);
                tracing::info!("🗑️  Evicted cache block {} ({} bytes)", block_id, pin.size);
            }
        }
        
        Ok(())
    }
    
    /// Garbage collect expired and unreferenced blocks
    pub fn garbage_collect(&mut self) -> Vec<String> {
        let mut to_remove = Vec::new();
        
        for (block_id, pin) in &self.pins {
            // Remove expired pins
            if pin.is_expired() {
                to_remove.push(block_id.clone());
                continue;
            }
            
            // Remove unreferenced non-user pins
            if pin.ref_count == 0 && pin.pin_type != PinType::User {
                to_remove.push(block_id.clone());
            }
        }
        
        // Remove collected blocks
        for block_id in &to_remove {
            self.unpin(block_id).ok();
        }
        
        tracing::info!("🗑️  Garbage collected {} blocks", to_remove.len());
        to_remove
    }
    
    /// List all pins for a user
    pub fn list_user_pins(&self, user_id: &str) -> Vec<&Pin> {
        if let Some(block_ids) = self.user_pins.get(user_id) {
            block_ids
                .iter()
                .filter_map(|id| self.pins.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get statistics
    pub fn stats(&self) -> PinningStats {
        let mut user_pins = 0;
        let mut cache_pins = 0;
        let mut paid_pins = 0;
        let mut temporary_pins = 0;
        let mut total_size = 0;
        
        for pin in self.pins.values() {
            total_size += pin.size;
            match pin.pin_type {
                PinType::User => user_pins += 1,
                PinType::Cache => cache_pins += 1,
                PinType::Paid => paid_pins += 1,
                PinType::Temporary => temporary_pins += 1,
            }
        }
        
        PinningStats {
            total_pins: self.pins.len(),
            user_pins,
            cache_pins,
            paid_pins,
            temporary_pins,
            total_size,
            cache_size: self.current_cache_size,
            max_cache_size: self.max_cache_size,
        }
    }
}

/// Pinning statistics
#[derive(Debug, Clone)]
pub struct PinningStats {
    pub total_pins: usize,
    pub user_pins: usize,
    pub cache_pins: usize,
    pub paid_pins: usize,
    pub temporary_pins: usize,
    pub total_size: usize,
    pub cache_size: usize,
    pub max_cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pinning_manager() {
        let mut pm = PinningManager::new(1024 * 1024); // 1MB cache
        
        // Pin a user block
        pm.pin("block1".to_string(), PinType::User, "user1".to_string(), 1000).unwrap();
        assert!(pm.is_pinned("block1"));
        
        // Pin a cache block
        pm.pin("block2".to_string(), PinType::Cache, "user1".to_string(), 2000).unwrap();
        assert!(pm.is_pinned("block2"));
        
        let stats = pm.stats();
        assert_eq!(stats.user_pins, 1);
        assert_eq!(stats.cache_pins, 1);
        assert_eq!(stats.total_size, 3000);
    }
    
    #[test]
    fn test_lru_eviction() {
        let mut pm = PinningManager::new(5000); // 5KB cache
        
        // Fill cache
        pm.pin("cache1".to_string(), PinType::Cache, "user1".to_string(), 2000).unwrap();
        pm.pin("cache2".to_string(), PinType::Cache, "user1".to_string(), 2000).unwrap();
        pm.pin("cache3".to_string(), PinType::Cache, "user1".to_string(), 2000).unwrap();
        
        // Should evict cache1 (least recently used)
        assert!(!pm.is_pinned("cache1"));
        assert!(pm.is_pinned("cache2"));
        assert!(pm.is_pinned("cache3"));
    }
    
    #[test]
    fn test_garbage_collection() {
        let mut pm = PinningManager::new(1024 * 1024);
        
        // Pin a cache block
        pm.pin("block1".to_string(), PinType::Cache, "user1".to_string(), 1000).unwrap();
        
        // Manually set ref_count to 0 to test garbage collection
        if let Some(pin) = pm.pins.get_mut("block1") {
            pin.ref_count = 0;
        }
        
        // Garbage collect
        let removed = pm.garbage_collect();
        assert_eq!(removed.len(), 1);
        assert!(!pm.is_pinned("block1"));
    }
    
    #[test]
    fn test_reference_counting() {
        let mut pm = PinningManager::new(1024 * 1024);
        
        // Pin twice
        pm.pin("block1".to_string(), PinType::User, "user1".to_string(), 1000).unwrap();
        pm.pin("block1".to_string(), PinType::User, "user1".to_string(), 1000).unwrap();
        
        let pin = pm.get_pin("block1").unwrap();
        assert_eq!(pin.ref_count, 2);
        
        // Unpin once - should still be pinned
        let can_delete = pm.unpin("block1").unwrap();
        assert!(!can_delete);
        assert!(pm.is_pinned("block1"));
        
        // Unpin again - now can delete
        let can_delete = pm.unpin("block1").unwrap();
        assert!(can_delete);
        assert!(!pm.is_pinned("block1"));
    }
}
