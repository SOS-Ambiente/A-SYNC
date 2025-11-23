// CONTENT-ADDRESSED STORAGE (CAS)
// Uses BLAKE3 hashing for content addressing and automatic deduplication

use blake3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Content identifier (CID) - BLAKE3 hash of content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId([u8; 32]);

impl ContentId {
    /// Create a new content ID from data
    pub fn from_data(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        ContentId(*hash.as_bytes())
    }

    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        ContentId(bytes)
    }

    /// Get raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(ContentId(array))
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Content-addressed block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    /// Content identifier (hash of data)
    pub cid: ContentId,
    /// Block data
    pub data: Vec<u8>,
    /// Size in bytes
    pub size: usize,
    /// Reference count (for garbage collection)
    pub ref_count: usize,
}

impl ContentBlock {
    /// Create a new content block
    pub fn new(data: Vec<u8>) -> Self {
        let cid = ContentId::from_data(&data);
        let size = data.len();

        ContentBlock {
            cid,
            data,
            size,
            ref_count: 1,
        }
    }

    /// Verify content integrity
    pub fn verify(&self) -> bool {
        let computed_cid = ContentId::from_data(&self.data);
        computed_cid == self.cid
    }

    /// Increment reference count
    pub fn add_ref(&mut self) {
        self.ref_count += 1;
    }

    /// Decrement reference count
    pub fn remove_ref(&mut self) -> usize {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        self.ref_count
    }
}

/// Content-addressed storage manager
pub struct ContentAddressedStorage {
    /// Block storage (CID -> block)
    blocks: HashMap<ContentId, ContentBlock>,
    /// Total stored bytes
    total_bytes: usize,
    /// Deduplication savings
    dedup_savings: usize,
}

impl ContentAddressedStorage {
    /// Create a new CAS
    pub fn new() -> Self {
        ContentAddressedStorage {
            blocks: HashMap::new(),
            total_bytes: 0,
            dedup_savings: 0,
        }
    }

    /// Store data (returns CID, deduplicates automatically)
    pub fn store(&mut self, data: Vec<u8>) -> ContentId {
        let cid = ContentId::from_data(&data);

        if let Some(block) = self.blocks.get_mut(&cid) {
            // Block already exists - deduplicate!
            block.add_ref();
            self.dedup_savings += data.len();
            tracing::debug!("✨ Deduplicated block {} (saved {} bytes)", cid, data.len());
        } else {
            // New block
            let size = data.len();
            let block = ContentBlock::new(data);
            self.blocks.insert(cid, block);
            self.total_bytes += size;
            tracing::debug!("💾 Stored new block {} ({} bytes)", cid, size);
        }

        cid
    }

    /// Retrieve data by CID
    pub fn get(&self, cid: &ContentId) -> Option<&[u8]> {
        self.blocks.get(cid).map(|block| block.data.as_slice())
    }

    /// Check if block exists
    pub fn contains(&self, cid: &ContentId) -> bool {
        self.blocks.contains_key(cid)
    }

    /// Remove reference to a block (garbage collection)
    pub fn remove_ref(&mut self, cid: &ContentId) -> bool {
        if let Some(block) = self.blocks.get_mut(cid) {
            let remaining_refs = block.remove_ref();
            if remaining_refs == 0 {
                // No more references, remove block
                if let Some(removed) = self.blocks.remove(cid) {
                    self.total_bytes -= removed.size;
                    tracing::debug!("🗑️  Garbage collected block {} ({} bytes)", cid, removed.size);
                    return true;
                }
            }
        }
        false
    }

    /// Get storage statistics
    pub fn stats(&self) -> CASStats {
        CASStats {
            total_blocks: self.blocks.len(),
            total_bytes: self.total_bytes,
            dedup_savings: self.dedup_savings,
            dedup_ratio: if self.total_bytes > 0 {
                (self.dedup_savings as f64 / (self.total_bytes + self.dedup_savings) as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Verify all blocks
    pub fn verify_all(&self) -> Vec<ContentId> {
        let mut corrupted = Vec::new();
        for (cid, block) in &self.blocks {
            if !block.verify() {
                corrupted.push(*cid);
            }
        }
        corrupted
    }

    /// Garbage collect unreferenced blocks
    pub fn garbage_collect(&mut self) -> usize {
        let mut removed = 0;
        let cids: Vec<ContentId> = self.blocks.keys().copied().collect();

        for cid in cids {
            if let Some(block) = self.blocks.get(&cid) {
                if block.ref_count == 0 {
                    self.remove_ref(&cid);
                    removed += 1;
                }
            }
        }

        removed
    }
}

impl Default for ContentAddressedStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// CAS statistics
#[derive(Debug, Clone)]
pub struct CASStats {
    pub total_blocks: usize,
    pub total_bytes: usize,
    pub dedup_savings: usize,
    pub dedup_ratio: f64,
}

/// Merkle DAG node for directory structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Node type
    pub node_type: NodeType,
    /// Content ID of this node
    pub cid: ContentId,
    /// Child nodes (for directories)
    pub children: HashMap<String, ContentId>,
    /// Size in bytes
    pub size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    File,
    Directory,
    Symlink,
}

impl MerkleNode {
    /// Create a file node
    pub fn file(cid: ContentId, size: usize) -> Self {
        MerkleNode {
            node_type: NodeType::File,
            cid,
            children: HashMap::new(),
            size,
        }
    }

    /// Create a directory node
    pub fn directory(children: HashMap<String, ContentId>) -> Self {
        // Calculate directory CID from children
        let mut hasher = blake3::Hasher::new();
        let mut sorted_children: Vec<_> = children.iter().collect();
        sorted_children.sort_by_key(|(name, _)| *name);

        for (name, cid) in sorted_children {
            hasher.update(name.as_bytes());
            hasher.update(cid.as_bytes());
        }

        let cid = ContentId::from_bytes(*hasher.finalize().as_bytes());

        MerkleNode {
            node_type: NodeType::Directory,
            cid,
            children,
            size: 0, // Directories have no direct size
        }
    }

    /// Add child to directory
    pub fn add_child(&mut self, name: String, cid: ContentId) {
        if self.node_type == NodeType::Directory {
            self.children.insert(name, cid);
            // Recalculate CID
            *self = Self::directory(self.children.clone());
        }
    }

    /// Get child CID
    pub fn get_child(&self, name: &str) -> Option<&ContentId> {
        self.children.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_id() {
        let data = b"Hello, World!";
        let cid = ContentId::from_data(data);

        let hex = cid.to_hex();
        let parsed = ContentId::from_hex(&hex).unwrap();

        assert_eq!(cid, parsed);
    }

    #[test]
    fn test_content_block() {
        let data = b"Test data".to_vec();
        let block = ContentBlock::new(data.clone());

        assert_eq!(block.size, data.len());
        assert_eq!(block.ref_count, 1);
        assert!(block.verify());
    }

    #[test]
    fn test_deduplication() {
        let mut cas = ContentAddressedStorage::new();

        let data = b"Duplicate data".to_vec();

        // Store first time
        let cid1 = cas.store(data.clone());
        assert_eq!(cas.blocks.len(), 1);

        // Store again - should deduplicate
        let cid2 = cas.store(data.clone());
        assert_eq!(cid1, cid2);
        assert_eq!(cas.blocks.len(), 1);

        let stats = cas.stats();
        assert_eq!(stats.dedup_savings, data.len());
        assert!(stats.dedup_ratio > 0.0);
    }

    #[test]
    fn test_garbage_collection() {
        let mut cas = ContentAddressedStorage::new();

        let data = b"Test data".to_vec();
        let cid = cas.store(data);

        // Remove reference
        cas.remove_ref(&cid);

        // Block should be removed
        assert!(!cas.contains(&cid));
    }

    #[test]
    fn test_merkle_node() {
        let file_cid = ContentId::from_data(b"file content");
        let file_node = MerkleNode::file(file_cid, 12);

        assert_eq!(file_node.node_type, NodeType::File);
        assert_eq!(file_node.size, 12);

        let mut children = HashMap::new();
        children.insert("file.txt".to_string(), file_cid);

        let dir_node = MerkleNode::directory(children);
        assert_eq!(dir_node.node_type, NodeType::Directory);
        assert!(dir_node.get_child("file.txt").is_some());
    }

    #[test]
    fn test_merkle_dag() {
        let mut cas = ContentAddressedStorage::new();

        // Create file
        let file_data = b"Hello, World!".to_vec();
        let file_cid = cas.store(file_data.clone());

        // Create directory with file
        let mut children = HashMap::new();
        children.insert("hello.txt".to_string(), file_cid);
        let dir_node = MerkleNode::directory(children);

        // Directory CID is deterministic
        let dir_data = bincode::serialize(&dir_node).unwrap();
        let dir_cid = cas.store(dir_data);

        assert!(cas.contains(&file_cid));
        assert!(cas.contains(&dir_cid));
    }
}
